//!
//! @file huff.rs
//! @author Andrew Spaulding (Kasplat)
//! @brief Implementation of Huffman compression.
//! @bug No known bugs.
//!

use std::collections::VecDeque;

use crate::bits::*;
use crate::serial;

/// The number of phrases which can be placed in a frequency table. One for each byte value.
const NUM_PHRASES: usize = u8::MAX as usize + 1;

/// A leaf in the huffman tree.
struct HTreeLeaf {
    weight: usize,
    phrase: u8
}

/// A node in a huffman encoding tree.
struct HTreeNode {
    weight: usize,
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
    /// Creates a data node from a stream of bits. See HTree::from_stream() for details.
    fn from_stream(
        bits: &mut BitStream<'_>
    ) -> Self {
        match bits.next().unwrap() {
            Bit::Zero => {
                let mut phrase = 0;
                for i in 0..u8::BITS {
                    phrase |= (bits.next().unwrap() as u8) << i;
                }
                Self::Leaf(HTreeLeaf { weight: 0, phrase })
            },
            Bit::One => {
                let left = Self::from_stream(bits);
                let right = Self::from_stream(bits);
                Self::Link(Box::new(HTreeNode { weight: 0, left, right }))
            }
        }
    }

    /// Serializes the data into a stream of bits.
    fn into_bits(
        &self,
        bits: &mut BitVec
    ) {
        match self {
            Self::Link(node) => {
                bits.push(Bit::One);
                node.left.into_bits(bits);
                node.right.into_bits(bits);
            },
            Self::Leaf(leaf) => {
                bits.push(Bit::Zero);
                for i in 0..u8::BITS {
                    bits.push(if (leaf.phrase >> i) & 1 == 1 { Bit::One } else { Bit::Zero });
                }
            }
        }
    }

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
                enc[node.phrase as usize].replace(vec.clone()).unwrap();
            }
        }
    }

    /// Gets the weight of the underlying data.
    fn weight(
        &self
    ) -> usize {
        match self {
            Self::Link(node) => node.weight,
            Self::Leaf(leaf) => leaf.weight
        }
    }
}

impl HTree {
    /// Creates a huffman tree for the phrases in the given data.
    fn new(
        data: &[u8]
    ) -> Self {
        // Gets a minimum weighted element from the two queues.
        let qmin = |l: &mut VecDeque<HTreeData>, r: &mut VecDeque<HTreeData>| -> HTreeData {
            if l.front().is_some() && r.front().is_some() {
                if l.front().unwrap().weight() <= r.front().unwrap().weight() {
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
            work_q.push_back(HTreeData::Link(Box::new(HTreeNode {
                weight: left.weight() + right.weight(),
                left,
                right
            })));
        }

        Self(qmin(&mut base_q, &mut work_q))
    }

    ///
    /// Creates an HTree from a bit stream.
    ///
    /// HTrees are encoded starting from the root, with a leaf being prefixed by a zero bit and
    /// a node being prefixed by a 1 bit. A leaf will contain its phrase, while a node will be
    /// followed by its left child (and all of its left childs decendents) and then its right
    /// child.
    ///
    fn from_stream(
        bits: &mut BitStream<'_>
    ) -> Self {
        Self(HTreeData::from_stream(bits))
    }

    /// Serializes this instance into a bit vector.
    fn into_bits(
        &self
    ) -> BitVec {
        let mut ret = BitVec::new();
        self.0.into_bits(&mut ret);
        return ret;
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

    /// Creates a queue of huffman nodes from a base data stream.
    fn create_base_queue(
        data: &[u8]
    ) -> VecDeque<HTreeData> {
        assert!(data.len() > 0);
        let mut freq = [0; NUM_PHRASES];
        for b in data.iter() {
            freq[*b as usize] += 1;
        }

        let mut base_q = Vec::new();
        for (b, f) in freq.iter().enumerate() {
            if *f > 0 {
                base_q.push(HTreeData::Leaf(HTreeLeaf { weight: *f, phrase: b as u8 }));
            }
        }

        assert!(base_q.len() > 0);
        base_q.sort_by(|lhs, rhs| lhs.weight().cmp(&rhs.weight()));
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
    ret.append(&tree.into_bits());

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
    let mut stream = BitStream::from_slice(&data[n..]);

    let tree = HTree::from_stream(&mut stream);
    let mut ret = Vec::new();
    for _ in 0..size {
        ret.push(tree.decode(&mut stream));
    }

    return ret;
}
