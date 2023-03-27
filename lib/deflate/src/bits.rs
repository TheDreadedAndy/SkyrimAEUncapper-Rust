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
        self.putb(bit as u8, 1);
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
        len: u32
    ) {
        assert!(len <= u8::BITS);
        self.extend_from_slice(&[b], len as usize);
    }

    /// Places up to a full x86 word into the stream.
    pub fn putw(
        &mut self,
        w: u16,
        len: u32
    ) {
        assert!(len <= u16::BITS);
        self.extend_from_slice(&w.to_le_bytes(), len as usize);
    }

    // Places the given slice with len many bits into the stream.
    //
    // The given bit length may be less than the size of the slice, or less than the msb in the
    // slice.
    fn extend_from_slice(
        &mut self,
        bits: &[u8],
        len: usize
    ) {
        assert!(len <= bits.len() * VEC_BITS);

        let byte_len = (len + (VEC_BITS - 1)) / VEC_BITS;
        assert!(byte_len <= bits.len());

        let lshift = self.len % VEC_BITS;
        if lshift == 0 {
            // If lshift is zero, then we can just do a direct vector append. The bytes in the
            // destination are full.
            self.bits.extend_from_slice(&bits[..byte_len]);
        } else {
            // Lshift is non-zero, so our other left/right shifts wont overflow.
            let rshift = VEC_BITS - lshift;

            if len > 0 {
                let i = self.bits.len() - 1;
                self.bits[i] |= bits[0] << lshift;
            }

            for i in 0..(byte_len - 1) {
                self.bits.push((bits[i] >> rshift) | (bits[i + 1] << lshift));
            }

            if len > rshift + ((byte_len - 1) * VEC_BITS) {
                self.bits.push(bits[byte_len - 1] >> rshift);
            }
        }

        self.len += len;

        if self.len % VEC_BITS > 0 {
            let last = self.bits.len() - 1;
            self.bits[last] &= (1 << (self.len % VEC_BITS)) - 1;
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
            len: bits.len() * VEC_BITS,
            index: 0
        }
    }

    /// Reads up to u16::BITS from the bit stream.
    pub fn getw(
        &mut self,
        len: u32
    ) -> u16 {
        assert!(self.len - self.index >= len as usize);
        assert!(len <= u16::BITS);

        let mut ret = [0; std::mem::size_of::<u16>()];
        let mut b = 0;
        let shift = self.index % VEC_BITS;
        let limit = self.index + len as usize;
        while self.index < limit {
            let byte_len = std::cmp::min(VEC_BITS, limit - self.index);
            let i = self.index / VEC_BITS;

            ret[b] = if shift == 0 {
                self.bits[i]
            } else {
                (self.bits[i] >> shift) | (self.bits[i + 1] << (VEC_BITS - shift))
            };

            self.index += byte_len;
            b += 1;
        }

        assert!(self.index == limit);
        return u16::from_le_bytes(ret) & ((1 << len) - 1);
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
            Some(if ret == 1 { Bit::One } else { Bit::Zero })
        } else {
            None
        }
    }
}
