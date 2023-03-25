//!
//! @file bits.rs
//! @author Andrew Spaulding (Kasplat)
//! @brief Structures for managing a stream of bits.
//!

const VEC_BITS: usize = u8::BITS as usize;

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
        Self { bits: Vec::new(), len: 0 }
    }

    /// Creates a bit stream from a vector, using all the bits in the vector.
    pub fn from_vec(
        bits: Vec<u8>
    ) -> Self {
        let len = bits.len() * VEC_BITS;
        Self { bits, len }
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
        let r = self.len % VEC_BITS;
        if r == 0 {
            self.bits.push(bit as u8);
        } else {
            let i = self.bits.len() - 1;
            self.bits[i] |= (bit as u8) << r;
        }
        self.len += 1;
    }

    /// Pops a bit from the end of the vector.
    pub fn pop(
        &mut self
    ) {
        self.len -= 1;
        let r = self.len % VEC_BITS;
        if r == 0 {
            self.bits.pop();
        } else {
            let i = self.bits.len() - 1;
            self.bits[i] &= ((1 << r) - 1) as u8;
        }
    }

    /// Appends one bit vector to the end of another.
    pub fn append(
        &mut self,
        b: &Self
    ) {
        self.extend_from_slice(&b.bits, b.len);
    }

    /// Places up to a full byte into the stream.
    pub fn putb(
        &mut self,
        b: u8,
        len: usize
    ) {
        assert!(len <= VEC_BITS);
        self.extend_from_slice(&[b], len);
    }

    // Places the given slice with len many bits into the stream.
    fn extend_from_slice(
        &mut self,
        bits: &[u8],
        len: usize
    ) {
        let lshift = self.len % VEC_BITS;
        if lshift == 0 {
            // If lshift is zero, then we can just do a direct vector append. The bytes in the
            // destination are full.
            self.bits.extend_from_slice(bits);
            self.len += len;
            return;
        }

        // Lshift is non-zero, so our other left/right shifts wont overflow.
        let rshift = VEC_BITS - lshift;

        if len > 0 {
            let i = self.bits.len() - 1;
            self.bits[i] |= bits[0] << lshift;
        }

        for i in 0..(bits.len()-1) {
            self.bits.push((bits[i] >> rshift) | (bits[i + 1] << lshift));
        }

        if len > rshift + ((bits.len() - 1) * VEC_BITS) {
            self.bits.push(bits[bits.len() - 1] >> rshift);
        }

        self.len += len;
    }
}

impl<'a> BitStream<'a> {
    /// Creates a bit stream to iterate over the bits in a slice.
    pub fn from_slice(
        bits: &'a [u8]
    ) -> Self {
        Self {
            bits,
            len: bits.len() * VEC_BITS,
            index: 0
        }
    }

    /// Reads up to u8::BITS from the bit stream.
    pub fn getb(
        &mut self,
        len: usize
    ) -> u8 {
        assert!(self.len - self.index > len);
        assert!(len <= VEC_BITS);

        let ret = if self.index % VEC_BITS == 0 {
            self.bits[self.index / VEC_BITS]
        } else {
            let i = self.index / VEC_BITS;
            let shift = self.index % VEC_BITS;
            (self.bits[i] >> shift) | if VEC_BITS - shift < len {
                self.bits[i + 1] << shift
            } else {
                0
            }
        };

        self.index += len;
        return ret & ((1 << len) - 1);
    }
}

impl<'a> Iterator for BitStream<'a> {
    type Item = Bit;
    fn next(
        &mut self
    ) -> Option<Self::Item> {
        if self.index < self.len {
            let ret = (self.bits[self.index / VEC_BITS] >> (self.index % VEC_BITS)) & 1;
            self.index += 1;
            assert!(ret <= 1);
            Some(unsafe { std::mem::transmute::<u8, Bit>(ret) })
        } else {
            None
        }
    }
}
