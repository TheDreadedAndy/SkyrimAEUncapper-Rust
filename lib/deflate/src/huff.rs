//!
//! @file huff.rs
//! @author Andrew Spaulding (Kasplat)
//! @brief Implementation of Huffman compression.
//! @bug No known bugs.
//!

use std::cmp::Ordering;
use std::collections::VecDeque;

use crate::bits::*;
use crate::serial;

/// The number of phrases which can be placed in a frequency table. One for each byte value.
const NUM_PHRASES: usize = u8::MAX as usize + 1;

/// A leaf in the huffman tree.
struct HTreeLeaf {
    phrase: u8
}

/// A node in a huffman encoding tree.
struct HTreeNode {
    left: HTreeData,
    right: HTreeData
}

/// The data in a node in the huffman tree.
enum HTreeData {
    Link(Box<HTreeNode>),
    Leaf(HTreeLeaf)
}

/// The root node of a huffman tree.
struct HTree(HTreeData);

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
                assert!(enc[node.phrase as usize].replace(vec.clone()).is_none());
            }
        }
    }

    /// Converts this data item and its children into a phrase length table.
    fn into_length_table(
        &self,
        depth: usize,
        table: &mut [u8]
    ) {
        match self {
            Self::Link(node) => {
                node.left.into_length_table(depth + 1, table);
                node.right.into_length_table(depth + 1, table);
            },
            Self::Leaf(node) => {
                assert!(depth > 0);
                table[node.phrase as usize] = depth.try_into().unwrap();
            }
        }
    }

    /// Creates a data element from a pair table and depth.
    fn from_pair_table(
        depth: usize,
        index: &mut usize,
        table: &[(u8, u8)]
    ) -> Self {
        if table[*index].1 == depth.try_into().unwrap() {
            let ret = Self::Leaf(HTreeLeaf { phrase: table[*index].0 });
            *index += 1;
            return ret;
        } else {
            let left = Self::from_pair_table(depth + 1, index, table);
            let right = Self::from_pair_table(depth + 1, index, table);
            return Self::Link(Box::new(HTreeNode { left, right }));
        }
    }
}

impl HTree {
    /// The number of used in the max code length prefix of a huffman tree serialization.
    const LEN_PREFIX_BITS: usize = 4;

    /// Creates a huffman tree for the phrases in the given data.
    fn new(
        data: &[u8]
    ) -> Self {
        // Gets a minimum weighted element from the two queues.
        let qmin = |
            l: &mut VecDeque<(usize, HTreeData)>,
            r: &mut VecDeque<(usize, HTreeData)>
        | -> (usize, HTreeData) {
            if l.front().is_some() && r.front().is_some() {
                if l.front().unwrap().0 < r.front().unwrap().0 {
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

    /// Consumes data from a bit stream, decoding it into a phrase.
    fn decode(
        &self,
        bits: &mut BitStream<'_>
    ) -> u8 {
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
                    return leaf.phrase;
                }
            }
        }
    }

    /// Creates an encode table for this tree.
    fn into_encode_table(
        self
    ) -> [Option<BitVec>; NUM_PHRASES] {
        const NODE_INIT: Option<BitVec> = None;
        let mut vec = BitVec::new();
        let mut ret = [NODE_INIT; NUM_PHRASES];
        self.0.into_encode_table(&mut vec, &mut ret);
        return ret;
    }

    /// Creates a encoding length table for this tree.
    fn into_length_table(
        &self
    ) -> [u8; NUM_PHRASES] {
        let mut ret = [0; NUM_PHRASES];
        self.0.into_length_table(0, &mut ret);
        return ret;
    }

    /// Creates a new huffman tree from a length table.
    fn from_length_table(
        table: [u8; NUM_PHRASES]
    ) -> Self {
        let mut groups = [(0, 0); NUM_PHRASES];
        for i in 0..NUM_PHRASES { groups[i] = (i as u8, table[i]); }
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
        table: [u8; NUM_PHRASES],
        vec: &mut BitVec
    ) {
        let width = |n: u8| { if n == 0 { 1u8 } else { (n.ilog2() + 1) as u8 } };

        // Determine how many bits are needed to represent each length.
        let mut max_len: u8 = 0;
        for b in table.iter() {
            max_len = std::cmp::max(max_len, width(*b));
        }
        assert!(max_len as u32 <= u8::BITS);

        vec.putb(max_len, Self::LEN_PREFIX_BITS);

        for b in table.iter() {
            vec.putb(*b, max_len as usize);
        }
    }

    /// Deserializes a length table from a bit stream.
    fn deserialize_length_table(
        stream: &mut BitStream<'_>
    ) -> [u8; NUM_PHRASES] {
        // Get bit width of each length.
        let max_len: usize = stream.getb(Self::LEN_PREFIX_BITS) as usize;

        let mut ret = [0; NUM_PHRASES];
        for i in 0..NUM_PHRASES {
            ret[i] = stream.getb(max_len);
        }
        return ret;
    }

    /// Creates a queue of huffman nodes from a base data stream.
    fn create_base_queue(
        data: &[u8]
    ) -> VecDeque<(usize, HTreeData)> {
        assert!(data.len() > 0);
        let mut freq = [0; NUM_PHRASES];
        for b in data.iter() {
            freq[*b as usize] += 1;
        }

        let mut base_q = Vec::new();
        for (b, f) in freq.iter().enumerate() {
            if *f > 0 {
                base_q.push((*f, HTreeData::Leaf(HTreeLeaf { phrase: b as u8 })));
            }
        }

        assert!(base_q.len() > 0);
        base_q.sort_by(|lhs, rhs| lhs.0.cmp(&rhs.0));
        return VecDeque::from(base_q);
    }
}

/// Compresses a byte slice using the huffman algorithm.
pub fn compress(
    data: &[u8]
) -> Vec<u8> {
    let mut ret = Vec::new();
    serial::write(data.len() as isize, &mut ret);

    let mut ret = BitVec::from_vec(ret);
    let tree = HTree::new(data);
    HTree::serialize_length_table(tree.into_length_table(), &mut ret);

    let table = tree.into_encode_table();
    for b in data.iter() {
        ret.append(table[*b as usize].as_ref().unwrap());
    }

    ret.into_vec()
}

/// Decompresses the huffman-compressed data from the given byte slice.
pub fn decompress(
    data: &[u8]
) -> Vec<u8> {
    let (n, size) = serial::read(data);
    let size = size as usize;
    let mut stream = BitStream::from_slice(&data[n..]);

    let tree = HTree::from_length_table(HTree::deserialize_length_table(&mut stream));
    let mut ret = Vec::new();
    for _ in 0..size {
        ret.push(tree.decode(&mut stream));
    }

    return ret;
}
