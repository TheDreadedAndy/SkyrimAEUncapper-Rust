//!
//! @file huff.rs
//! @author Andrew Spaulding (Kasplat)
//! @brief Implementation of Huffman compression.
//! @bug No known bugs.
//!

use core::cmp::{Ordering, max};
use alloc::vec::Vec;
use alloc::boxed::Box;

use crate::bits::*;
use crate::lz::Token;

/// The maximum number of bits an offset can encode. This is a constant within the deflate
/// algorithm.
const OFFSET_MAX_BITS: u32 = u16::BITS - 1;

/// The number of phrases which can be placed in a frequency table. One for each byte value plus
/// 15 different bit lengths for offsets/run-lengths + 1 stop code.
const NUM_CODES: usize = (u8::MAX as usize + 1) + OFFSET_MAX_BITS as usize + 1;

/// A leaf in the huffman tree.
///
/// Encodes either a raw byte, an offset, or a stop code. Note that the offset is encoded as
/// the number of bits within the offset, where any number greater than 1 has an implicit one,
/// and all numbers are followed by their remaining explicit bits in the stream.
///
/// Since the range of valid offsets is 1..u16::MAX-1, we store offsets from tokens as offset-1
/// so that they can be stored in 15 bits instead of 16.
///
/// Bytes are values [0, 256).
/// Offsets are values [256, 271).
/// The stop code is 271.
#[derive(Copy, Clone)]
struct Codeword(u16);

/// A node in a huffman encoding tree.
struct HTreeNode {
    left: HTreeData,
    right: HTreeData
}

/// The data in a node in the huffman tree.
enum HTreeData {
    Link(Box<HTreeNode>),
    Leaf(Codeword)
}

/// A queue used to build the initial huffman tree from the frequency data.
struct HQueue {
    buf: [Option<(usize, HTreeData)>; NUM_CODES],
    front: usize,
    back: usize
}

/// The root node of a huffman tree.
struct HTree(HTreeData);

impl Codeword {
    /// The inclusive value range for byte codewords.
    const BYTE_CODE_BASE: u16 = 0;
    const BYTE_CODE_LIMIT: u16 = u8::MAX as u16;

    /// The inclusive value range for offset codewords.
    const OFFSET_CODE_BASE: u16 = Self::BYTE_CODE_LIMIT + 1;
    const OFFSET_CODE_LIMIT: u16 = (Self::OFFSET_CODE_BASE + OFFSET_MAX_BITS as u16) - 1;

    /// The phrase used to mark that a bit stream has ended.
    const STOP_CODE: u16 = Self::OFFSET_CODE_LIMIT + 1;

    /// Converts a codeword to an Codeword.
    const fn from_raw(
        code: u16
    ) -> Self {
        assert!(Self::BYTE_CODE_BASE <= code && code <= Self::STOP_CODE);
        Self(code)
    }

    /// Creates a new Codeword from an LZ77 token.
    const fn from_token(
        token: &Token
    ) -> Self {
        match token {
            Token::Phrase(b) => Self(*b as u16),
            Token::Stop => Self(Self::STOP_CODE),
            Token::Offset(offset) => {
                assert!(*offset < (1 << OFFSET_MAX_BITS));
                Self(Self::BYTE_CODE_LIMIT + bit_width(*offset - 1) as u16)
            }
        }
    }

    /// Converts a leaf into a huffman codeword.
    const fn as_raw(
        self
    ) -> u16 {
        self.0
    }

    /// Emits any excess bits associated with the leaf.
    fn emit_bits(
        token: &Token,
        vec: &mut BitVec
    ) {
        if let Token::Offset(raw_offset) = token {
            let offset = *raw_offset - 1;
            let bits = max(bit_width(offset) - 1, 1);
            assert!(offset <= 1 || offset == (offset & ((1 << bits) - 1)) | (1 << bits));
            vec.putw(offset, bits);
        }
    }

    /// Converts a leaf into a token, consuming from the data stream as necessary.
    ///
    /// Note that the data stream will only be consumed from if the leaf is an offset.
    fn to_token(
        self,
        stream: &mut BitStream<'_>
    ) -> Token {
        match self.0 {
            Self::BYTE_CODE_BASE..=Self::BYTE_CODE_LIMIT => Token::Phrase(self.0 as u8),
            Self::STOP_CODE => Token::Stop,
            Self::OFFSET_CODE_BASE..=Self::OFFSET_CODE_LIMIT => {
                let bits: u32 = (self.0 - Self::BYTE_CODE_LIMIT) as u32;
                let extra_bits: u32 = max(bits - 1, 1) as u32;
                assert!((0 < extra_bits) && (extra_bits <= OFFSET_MAX_BITS - 1));
                Token::Offset((stream.getw(extra_bits) | (((bits > 1) as u16) << extra_bits)) + 1)
            },
            _ => unreachable!()
        }
    }
}

impl HTreeData {
    /// Converts this data item and its children into an encoding table.
    fn into_encode_table(
        &self,
        vec: &mut BitVec,
        enc: &mut [Option<BitVec>]
    ) {
        match self {
            Self::Link(node) => {
                vec.push(Bit::Zero);
                node.left.into_encode_table(vec, enc);
                vec.pop();

                vec.push(Bit::One);
                node.right.into_encode_table(vec, enc);
                vec.pop();
            },
            Self::Leaf(node) => {
                assert!(enc[node.as_raw() as usize].replace(vec.clone()).is_none());
            }
        }
    }

    /// Converts this data item and its children into a phrase length table.
    fn into_length_table(
        &self,
        depth: usize,
        table: &mut [u16]
    ) {
        match self {
            Self::Link(node) => {
                node.left.into_length_table(depth + 1, table);
                node.right.into_length_table(depth + 1, table);
            },
            Self::Leaf(node) => {
                assert!(depth > 0);
                assert!(table[node.as_raw() as usize] == 0);
                table[node.as_raw() as usize] = depth.try_into().unwrap();
            }
        }
    }

    /// Creates a data element from a pair table and depth.
    fn from_pair_table(
        depth: usize,
        index: &mut usize,
        table: &[(u16, u16)]
    ) -> Self {
        if table[*index].1 == depth.try_into().unwrap() {
            let ret = Self::Leaf(Codeword::from_raw(table[*index].0));
            *index += 1;
            ret
        } else {
            let left = Self::from_pair_table(depth + 1, index, table);
            let right = Self::from_pair_table(depth + 1, index, table);
            Self::Link(Box::new(HTreeNode { left, right }))
        }
    }
}

impl HQueue {
    const fn new() -> Self {
        const INIT_ELEM: Option<(usize, HTreeData)> = None;
        Self { buf: [INIT_ELEM; NUM_CODES], front: 0, back: 0 }
    }

    const fn len(
        &self
    ) -> usize {
        self.back - self.front
    }

    const fn peek(
        &self
    ) -> Option<&(usize, HTreeData)> {
        self.buf[self.front].as_ref()
    }

    fn enq(
        &mut self,
        data: (usize, HTreeData)
    ) {
        assert!(self.back < NUM_CODES);
        assert!(self.buf[self.back].replace(data).is_none());
        self.back += 1;
    }

    fn deq(
        &mut self
    ) -> Option<(usize, HTreeData)> {
        if self.len() == 0 {
            None
        } else {
            let ret = self.buf[self.front].take();
            self.front += 1;
            return ret;
        }
    }
}

impl HTree {
    /// The number of used in the max code length prefix of a huffman tree serialization.
    const LEN_PREFIX_BITS: u32 = 4;

    /// Creates a huffman tree for the phrases in the given data.
    fn new(
        data: &[Token]
    ) -> Self {
        // Gets a minimum weighted element from the two queues. At least one of the queues must 
        // have an element in it.
        //
        // Returns the element and its current frequency/weight.
        let qmin = |lhs: &mut HQueue, rhs: &mut HQueue| -> (usize, HTreeData) {
            if lhs.peek().is_some() && rhs.peek().is_some() {
                if lhs.peek().unwrap().0 <= rhs.peek().unwrap().0 {
                    lhs.deq().unwrap()
                } else {
                    rhs.deq().unwrap()
                }
            } else if lhs.peek().is_some() {
                lhs.deq().unwrap()
            } else {
                rhs.deq().unwrap()
            }
        };

        // Next, we construct a queue of codewords sorted by the frequency of their appearance in
        // the input data stream. This queue will not contain codewords which never appear in the
        // stream.

        // Create a table that converts codeword values into their frequency in the data.
        assert!(data.len() > 0);
        let mut freq = [0; NUM_CODES];
        for token in data.iter() {
            freq[Codeword::from_token(token).as_raw() as usize] += 1;
        }

        // Create a table of code words sorted by their frequency, from least to greatest.
        let mut index_freq = [(0, 0); NUM_CODES];
        for (i, f) in freq.iter().enumerate() { index_freq[i] = (i, *f); }
        index_freq.sort_by(|lhs, rhs| lhs.1.cmp(&rhs.1));

        // Finally, convert that codeword table into a queue.
        let mut base_q = HQueue::new();
        for (b, f) in index_freq.iter() {
            if *f > 0 {
                base_q.enq((*f, HTreeData::Leaf(Codeword::from_raw(*b as u16))));
            }
        }

        // Construct a huffman tree from the bottom up using the queue we just created. We do this
        // by repeatedly creating a node with the lowest possible weight, which will be the bottom
        // of the tree. Note that when we do this we don't actual care about what the encoding is,
        // just what the position in the tree is.
        //
        // We can use induction to show that this process creates a proper huffman tree.
        //
        // Base case:
        //   1) We'll start with an empty work queue and a full base queue.
        //   2) We'll grab the two least elements, and then create a node containing them in sorted
        //      order.
        //   3) That node will then be pushed on to the back of the work queue. Since there were no
        //      elements previously on the work queue, it is still sorted. Since we only removed
        //      elements from the front of the base queue, it is still sorted.
        //
        // Iterative step:
        //   1) Now we have a sorted work queue and a sorted base queue with some number of
        //      elements.
        //   2) We take two elements from between the two queues, taking the lowest one at each
        //      step.
        //   3) We enqueue it to the back of the work queue. The work queue is still sorted because
        //      in previous steps the element pushed to the back of the work queue was the sum of
        //      two values less than or equal to the two values we just summed.
        //
        // And so, the queues remain sorted, and we are able to construct a proper encoding of a
        // huffman tree from the bottom up, ensuring that the frequency requirement of the table is
        // always met.
        let mut work_q = HQueue::new();
        while base_q.len() + work_q.len() > 1 {
            let (left_freq,  left_node)  = qmin(&mut base_q, &mut work_q);
            let (right_freq, right_node) = qmin(&mut base_q, &mut work_q);
            work_q.enq((left_freq + right_freq, HTreeData::Link(Box::new(HTreeNode {
                left: left_node,
                right: right_node
            }))));
        }

        // At this point, only one element remains within the tree. It could technically be in
        // either one, in the case where the base tree only had one element to begin with.
        // We collect that one element, and use it as our root node.
        let tree = Self(qmin(&mut base_q, &mut work_q).1);

        // We use the length table encoding process to ensure the returned tree is canonical.
        Self::from_length_table(tree.into_length_table())
    }

    /// Consumes data from a bit stream, decoding it into a lz77 token.
    fn decode(
        &self,
        bits: &mut BitStream<'_>
    ) -> Token {
        let mut node = &self.0;
        loop {
            match node {
                HTreeData::Link(split) => {
                    if bits.next().unwrap() == Bit::Zero {
                        node = &split.left;
                    } else {
                        node = &split.right;
                    }
                },
                HTreeData::Leaf(leaf) => {
                    return leaf.to_token(bits);
                }
            }
        }
    }

    /// Creates an encode table for this tree.
    fn into_encode_table(
        self
    ) -> [Option<BitVec>; NUM_CODES] {
        const NODE_INIT: Option<BitVec> = None;
        let mut vec = BitVec::new();
        let mut ret = [NODE_INIT; NUM_CODES];
        self.0.into_encode_table(&mut vec, &mut ret);
        return ret;
    }

    /// Creates a encoding length table for this tree.
    fn into_length_table(
        &self
    ) -> [u16; NUM_CODES] {
        let mut ret = [0; NUM_CODES];
        self.0.into_length_table(0, &mut ret);
        return ret;
    }

    /// Creates a new huffman tree from a length table.
    fn from_length_table(
        table: [u16; NUM_CODES]
    ) -> Self {
        let mut groups = [(0, 0); NUM_CODES];
        for i in 0..NUM_CODES { groups[i] = (i as u16, table[i]); }
        groups.sort_by(|lhs, rhs| {
            let len_cmp = lhs.1.cmp(&rhs.1);
            if let Ordering::Equal = len_cmp {
                lhs.0.cmp(&rhs.0)
            } else {
                len_cmp
            }
        });

        let mut i = 0;
        while groups[i].1 == 0 { i += 1; }
        let mut index = 0;
        Self(HTreeData::from_pair_table(0, &mut index, groups.split_at(i).1))
    }

    /// Serializes a length table into a bit vector.
    ///
    /// The length table is formatted as:
    /// - LEN: 4-bit bit-length width encoding.
    /// - RUN: 4-bit run-length width encoding.
    /// - An item group which is either:
    ///     * 0 + LEN bits encoding a single item.
    ///     * 1 + LEN bits encoding an item + RUN bits encoding how many times to repeat it.
    fn serialize_length_table(
        table: [u16; NUM_CODES],
        vec: &mut BitVec
    ) {
        // Determine how many bits are needed to represent each length and the maximum match
        // width. Additionally, build up an in-order table of matches.
        let mut max_len: u32 = 0;
        let mut max_run: u32 = 0;
        let mut cur_run: u16 = 0;
        let mut cur_match: u16 = 0;
        let mut num_matches: usize = 0;
        let mut matches = [(0, 0); NUM_CODES];
        for b in table.iter() {
            max_len = core::cmp::max(max_len, bit_width(*b));

            if cur_run == 0 {
                // First iteration.
                cur_match = *b;
                cur_run = 1;
            } else if *b == cur_match {
                cur_run += 1;
            } else {
                max_run = core::cmp::max(max_run, bit_width(cur_run));
                matches[num_matches] = (cur_match, cur_run);
                num_matches += 1;

                cur_run = 1;
                cur_match = *b;
            }
        }

        max_run = core::cmp::max(max_run, bit_width(cur_run));
        matches[num_matches] = (cur_match, cur_run);

        assert!(max_len as usize <= NUM_CODES);
        assert!(bit_width(max_len as u16) <= Self::LEN_PREFIX_BITS);
        assert!(bit_width(max_run as u16) <= Self::LEN_PREFIX_BITS);

        vec.putb(max_len as u8, Self::LEN_PREFIX_BITS);
        vec.putb(max_run as u8, Self::LEN_PREFIX_BITS);

        for (b, run) in matches.iter() {
            if *run == 0 { break; }

            if *run == 1 {
                vec.push(Bit::Zero);
                vec.putw(*b, max_len);
            } else {
                vec.push(Bit::One);
                vec.putw(*b, max_len);
                vec.putw(*run, max_run);
            }
        }
    }

    /// Deserializes a length table from a bit stream.
    fn deserialize_length_table(
        stream: &mut BitStream<'_>
    ) -> [u16; NUM_CODES] {
        // Get bit/run width of each item.
        let max_len: u32 = stream.getw(Self::LEN_PREFIX_BITS) as u32;
        let max_run: u32 = stream.getw(Self::LEN_PREFIX_BITS) as u32;

        let mut ret = [0; NUM_CODES];
        let mut i = 0;
        while i < NUM_CODES {
            if stream.next().unwrap() == Bit::Zero {
                ret[i] = stream.getw(max_len);
                i += 1;
            } else {
                let b = stream.getw(max_len);
                let run = stream.getw(max_run);
                for _ in 0..run {
                    ret[i] = b;
                    i += 1;
                }
            }
        }

        return ret;
    }
}

/// Compresses a byte slice using the huffman algorithm.
pub fn compress(
    data: &[Token]
) -> Vec<u8> {
    let mut ret = BitVec::new();

    let tree = HTree::new(data);
    HTree::serialize_length_table(tree.into_length_table(), &mut ret);

    let table = tree.into_encode_table();
    for b in data.iter() {
        ret.append(table[Codeword::from_token(b).as_raw() as usize].as_ref().unwrap());
        Codeword::emit_bits(b, &mut ret);
    }

    ret.into_vec()
}

/// Decompresses the huffman-compressed data from the given byte slice.
pub fn decompress(
    data: &[u8]
) -> Vec<Token> {
    let mut stream = BitStream::from_slice(data);
    let tree = HTree::from_length_table(HTree::deserialize_length_table(&mut stream));
    let mut ret = Vec::new();
    loop {
        let token = tree.decode(&mut stream);
        ret.push(token);
        if token == Token::Stop { break; }
    }

    return ret;
}

const fn bit_width(
    n: u16
) -> u32 {
    if n == 0 { 1 } else { n.ilog2() + 1 }
}
