//!
//! @file lib.rs
//! @author Andrew Spaulding (Kasplat)
//! @brief Top level library file for vector deflation.
//! @bug No known bugs.
//!

mod serial;
mod circ;
mod lz;
mod bits;
mod huff;

/// Compresses data using lz77 + huffman.
pub fn compress(
    data: &[u8]
) -> Vec<u8> {
    huff::compress(lz::compress(data).as_slice())
}

/// Decompresses data using lz77 + huffman.
pub fn decompress(
    data: &[u8]
) -> Vec<u8> {
    lz::decompress(huff::decompress(data).as_slice())
}
