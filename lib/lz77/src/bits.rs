//!
//! @file bits.rs
//! @author Andrew Spaulding (Kasplat)
//! @brief Structures for managing a stream of bits.
//!

/// Type-safe enum of a bit. Can be cast to u8 safely.
#[repr(u8)]
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum Bit {
    Zero = 0,
    One = 1
}

/// A vector of bits, ordered lsb first in each byte.
#[derive(Clone)]
pub struct BitVec {
    bits: Vec<u8>,
    len: usize
}

/// An iterator over a stream of bits, lsb-first.
pub struct BitStream<'a> {
    bits: &'a [u8],
    len: usize,
    index: usize
}

impl BitVec {
    pub fn new() -> Self {
        Self {
            bits: Vec::new(),
            len: 0
        }
    }

    /// Creates a bit stream from a vector, using all the bits in the vector.
    pub fn from_vec(
        bits: Vec<u8>
    ) -> Self {
        Self {
            bits,
            len: bits.len() * u8::BITS
        }
    }

    /// Gets the underlying vector of the bit vector.
    pub fn into_vec(
        self
    ) -> Vec<u8> {
        self.bits
    }

    /// Pushes a bit to the end of the bit vector.
    pub fn push(
        &mut self,
        bit: Bit
    ) {
        let r = self.len % u8::BITS;
        if r == 0 {
            self.bits.push(bit as u8);
        } else {
            self.bits[self.bits.len() - 1] |= (bit as u8) << r;
        }
        self.len += 1;
    }

    /// Pops a bit from the end of the vector.
    pub fn pop(
        &mut self
    ) {
        self.len -= 1;
        if self.len % u8::BITS == 0 {
            self.bits.pop()
        }
    }

    /// Appends one bit vector to the end of another.
    pub fn append(
        &mut self,
        b: &Self
    ) {
        let r = self.len % u8::BITS;

        if b.len > 0 {
            self.bits[self.bits.len() - 1] |= b.bits[0] << r;
        }

        for i in 0..(b.bits.len()-1) {
            self.bits.push((b.bits[i] >> (u8::BITS - r)) | (b.bits[i + 1] << r));
        }

        if b.len - (u8::BITS - r) > 0 {
            self.bits.push(b.bits[b.bits.len() - 1] >> (u8::BITS - r));
        }

        self.len += b.len;
    }

    /// Creates an iterator over this bit stream.
    pub fn iter(
        &self
    ) -> BitStream<'_> {
        BitStream {
            bits: self.bits.slice(),
            len: self.len,
            index: 0
        }
    }
}

impl<'a> BitStream<'a> {
    /// Creates a bit stream to iterate over the bits in a slice.
    pub fn from_slice(
        bits: &'a [u8]
    ) -> Self {
        Self {
            bits,
            len: bits.len() * u8::BITS,
            index: 0
        }
    }
}

impl<'a> Iterator for BitStream<'a> {
    type Item = Bit;
    fn next(
        &mut self
    ) -> Option<Self::Item> {
        if self.index < self.len {
            let ret = self.bits[self.index / u8::BITS] >> (self.index % u8::BITS);
            self.index += 1;
            assert!(ret <= 1);
            Some(unsafe { std::mem::transmute::<u8, Bit>(ret) })
        } else {
            None
        }
    }
}
