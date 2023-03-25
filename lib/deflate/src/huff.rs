//!
//! @file huff.rs
//! @author Andrew Spaulding (Kasplat)
//! @brief Implementation of Huffman compression.
//! @bug No known bugs.
//!

use std::cmp::{Ordering, max};
use std::collections::VecDeque;

use crate::bits::*;
use crate::lz::Token;

/// The number of phrases which can be placed in a frequency table. One for each byte value plus
/// 15 different bit lengths for offsets/run-lengths + 1 stop code.
const NUM_CODES: usize = (u8::MAX as usize + 1) + 15 + 1;

/// The maximum number of bits an offset can encode. This is a constant within the deflate
/// algorithm.
const OFFSET_MAX_BITS: u32 = u16::BITS - 1;

/// A leaf in the huffman tree.
///
/// Encodes either a raw byte, an offset, or a stop code. Note that the offset is encoded as
/// the number of bits within the offset, where any number greater than 1 has an implicit one,
/// and all numbers are followed by their remaining explicit bits in the stream.
///
/// Bytes are values [0, 256).
/// Offsets are values [256, 271).
/// The stop code is 271.
#[derive(Copy, Clone)]
struct HTreeLeaf(u16);

/// A node in a huffman encoding tree.
struct HTreeNode {
    left: HTreeData,
    right: HTreeData
}

/// The data in a node in the huffman tree.
enum HTreeData {
    Link(Box<HTreeNode>),
    Leaf(HTreeLeaf),
    Stub
}

/// The root node of a huffman tree.
struct HTree(HTreeData);

impl HTreeLeaf {
    /// The inclusive value range for byte codewords.
    const BYTE_CODE_BASE: u16 = 0;
    const BYTE_CODE_LIMIT: u16 = u8::MAX as u16;

    /// The inclusive value range for offset codewords.
    const OFFSET_CODE_BASE: u16 = Self::BYTE_CODE_LIMIT + 1;
    const OFFSET_CODE_LIMIT: u16 = (Self::OFFSET_CODE_BASE + OFFSET_MAX_BITS as u16) - 1;

    /// The phrase used to mark that a bit stream has ended.
    const STOP_CODE: u16 = Self::OFFSET_CODE_LIMIT + 1;

    /// Emits any excess bits associated with the leaf.
    fn emit_bits(
        token: &Token,
        vec: &mut BitVec
    ) {
        if let Token::Offset(offset) = token {
            let bits = max(bit_width(*offset) - 1, 1);
            assert!(*offset <= 1 || *offset & (1 << bits) > 0);
            vec.putw(*offset, bits);
        }
    }

    /// Creates a new HTreeLeaf from an LZ77 token.
    fn from_token(
        token: &Token
    ) -> Self {
        match token {
            Token::Phrase(b) => Self(*b as u16),
            Token::Stop => Self(Self::STOP_CODE),
            Token::Offset(offset) => {
                Self(Self::BYTE_CODE_LIMIT + bit_width(*offset) as u16)
            }
        }
    }

    /// Converts a codeword to an HTreeLeaf.
    fn from_codeword(
        code: u16
    ) -> Self {
        assert!(Self::BYTE_CODE_BASE <= code && code <= Self::STOP_CODE);
        Self(code)
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
                assert!((0 < bits) && (bits <= OFFSET_MAX_BITS));
                Token::Offset(if bits > 1 {
                    stream.getw(bits - 1) | (1 << (bits - 1))
                } else {
                    stream.getw(1)
                })
            },
            _ => unreachable!()
        }
    }

    /// Converts a leaf into a huffman codeword.
    fn to_codeword(
        self
    ) -> u16 {
        self.0
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
                assert!(enc[node.to_codeword() as usize].replace(vec.clone()).is_none());
            },
            Self::Stub => ()
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
                table[node.to_codeword() as usize] = depth.try_into().unwrap();
            },
            Self::Stub => ()
        }
    }

    /// Creates a data element from a pair table and depth.
    fn from_pair_table(
        depth: usize,
        index: &mut usize,
        table: &[(u16, u16)]
    ) -> Self {
        if *index >= table.len() {
            Self::Stub
        } else if table[*index].1 == depth.try_into().unwrap() {
            let ret = Self::Leaf(HTreeLeaf::from_codeword(table[*index].0));
            *index += 1;
            ret
        } else {
            let left = Self::from_pair_table(depth + 1, index, table);
            let right = Self::from_pair_table(depth + 1, index, table);
            Self::Link(Box::new(HTreeNode { left, right }))
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
        // Gets a minimum weighted element from the two queues.
        let qmin = |
            l: &mut VecDeque<(usize, HTreeData)>,
            r: &mut VecDeque<(usize, HTreeData)>
        | -> (usize, HTreeData) {
            if l.front().is_some() && r.front().is_some() {
                if l.front().unwrap().0 <= r.front().unwrap().0 {
                    l.pop_front().unwrap()
                } else {
                    r.pop_front().unwrap()
                }
            } else if l.front().is_some() {
                l.pop_front().unwrap()
            } else {
                r.pop_front().unwrap()
            }
        };

        let mut base_q = Self::create_base_queue(data);
        let mut work_q = VecDeque::new();
        while base_q.len() + work_q.len() > 1 {
            let left = qmin(&mut base_q, &mut work_q);
            let right = qmin(&mut base_q, &mut work_q);
            work_q.push_back((left.0 + right.0, HTreeData::Link(Box::new(HTreeNode {
                left: left.1,
                right: right.1
            }))));
        }

        // Make the tree into a canonical encoding.
        let tree = Self(qmin(&mut base_q, &mut work_q).1);
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
                HTreeData::Stub => panic!("Cannot decode from stub!")
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
    fn serialize_length_table(
        table: [u16; NUM_CODES],
        vec: &mut BitVec
    ) {
        // Determine how many bits are needed to represent each length.
        let mut max_len: u32 = 0;
        for b in table.iter() {
            max_len = std::cmp::max(max_len, bit_width(*b));
        }
        assert!(max_len as usize <= NUM_CODES);

        vec.putb(max_len as u8, Self::LEN_PREFIX_BITS);

        for b in table.iter() {
            vec.putw(*b, max_len);
        }
    }

    /// Deserializes a length table from a bit stream.
    fn deserialize_length_table(
        stream: &mut BitStream<'_>
    ) -> [u16; NUM_CODES] {
        // Get bit width of each length.
        let max_len: u32 = stream.getw(Self::LEN_PREFIX_BITS) as u32;

        let mut ret = [0; NUM_CODES];
        for i in 0..NUM_CODES {
            ret[i] = stream.getw(max_len);
        }
        return ret;
    }

    /// Creates a queue of huffman nodes from a base data stream.
    fn create_base_queue(
        data: &[Token]
    ) -> VecDeque<(usize, HTreeData)> {
        assert!(data.len() > 0);
        let mut freq = [0; NUM_CODES];
        for token in data.iter() {
            freq[HTreeLeaf::from_token(token).to_codeword() as usize] += 1;
        }

        let mut base_q = Vec::new();
        for (b, f) in freq.iter().enumerate() {
            if *f > 0 {
                base_q.push((*f, HTreeData::Leaf(HTreeLeaf::from_codeword(b as u16))));
            }
        }

        assert!(base_q.len() > 0);
        base_q.sort_by(|lhs, rhs| lhs.0.cmp(&rhs.0));
        return VecDeque::from(base_q);
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
        ret.append(table[HTreeLeaf::from_token(b).to_codeword() as usize].as_ref().unwrap());
        HTreeLeaf::emit_bits(b, &mut ret);
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

fn bit_width(
    n: u16
) -> u32 {
    if n == 0 { 1 } else { n.ilog2() + 1 }
}
