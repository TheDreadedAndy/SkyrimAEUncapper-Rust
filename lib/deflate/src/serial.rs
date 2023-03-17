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
    n: isize,
    out: &mut Vec<u8>
) {
    // Always encode at least one sign extension bit.
    let leading = (if n < 0 { !n } else { n }).leading_zeros();
    let bits = std::cmp::min((isize::BITS - leading) + 1, isize::BITS);

    let mut shift = 0;
    while shift < bits {
        let cont = if shift + BYTE_DATA_BITS < bits { BYTE_CONT_FLAG } else { 0 };
        out.push((((n >> shift) as u8) & BYTE_DATA_MASK) | cont);
        shift += BYTE_DATA_BITS;
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
        assert!(shift < isize::BITS);

        let b = inb[i];
        i += 1;

        n |= ((b & BYTE_DATA_MASK) as isize) << shift;
        shift += BYTE_DATA_BITS;

        if b & BYTE_CONT_FLAG == 0 { break; }
    }

    let bits = std::cmp::min(shift, isize::BITS);
    let ext_shift = isize::BITS - bits;

    (i, (n << ext_shift) >> ext_shift)
}
