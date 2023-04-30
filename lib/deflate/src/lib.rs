//!
//! @file lib.rs
//! @author Andrew Spaulding (Kasplat)
//! @brief Top level library file for vector deflation.
//!
//! Note that this is not a complient deflate implementation. It is simply an implementation of
//! both huffman and lz77 compression on top of each other.
//!

mod lz;
mod bits;
mod huff;

/// Compresses data using lz77 + huffman.
pub fn compress(
    data: &[u8]
) -> Vec<u8> {
    huff::compress(&lz::compress(data))
}

/// Decompresses data using lz77 + huffman.
pub fn decompress(
    data: &[u8]
) -> Vec<u8> {
    lz::decompress(&huff::decompress(data))
}
