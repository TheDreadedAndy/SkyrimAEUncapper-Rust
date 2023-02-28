//!
//! @file serial.rs
//! @author Andrew Spaulding (Kasplat)
//! @brief Compressed serialization of signed integers.
//! @bug No known bugs.
//!

const BYTE_CONT_FLAG: u8 = 0x80;
const BYTE_DATA_BITS: u32 = 7;
const BYTE_DATA_MASK: u8 = (1 << BYTE_DATA_BITS) - 1;

/// Serializes an isize to as few bytes as possible, using the msb of each
/// byte as a continuation bit.
pub fn write(
    mut n: isize,
    out: &mut Vec<u8>
) {
    let is_neg = n < 0;
    let can_stop = |n, next_n| {
        (!is_neg && (next_n == 0) && ((n & (1 << (BYTE_DATA_BITS - 1))) == 0)) ||
        (is_neg && (next_n == -1) && ((n & (1 << (BYTE_DATA_BITS - 1))) > 0))
    };

    loop {
        let next_n = n >> BYTE_DATA_BITS;
        let stop = can_stop(n, next_n);
        let cont = if stop { 0 } else { BYTE_CONT_FLAG };
        out.push(((n as u8) & BYTE_DATA_MASK) | cont);
        n = next_n;

        if stop { break; }
    }
}

/// Deserializes data which was serialized with write().
pub fn read(
    inb: &[u8]
) -> (usize, isize) {
    let mut n: isize = 0;
    let mut shift: u32 = 0;
    let mut i: usize = 0;

    loop {
        let b = inb[i];
        i += 1;

        n |= ((b & BYTE_DATA_MASK) as isize) << shift;

        if b & BYTE_CONT_FLAG > 0 {
            shift += BYTE_DATA_BITS;
            assert!(shift < isize::BITS);
        } else {
            break;
        }
    }

    let bits = std::cmp::min(shift + BYTE_DATA_BITS, isize::BITS);
    let ext_shift = isize::BITS - bits;

    (i, (n << ext_shift) >> ext_shift)
}
