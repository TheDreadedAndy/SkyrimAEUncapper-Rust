//!
//! @file lz.rs
//! @author Andrew Spaulding (Kasplat)
//! @brief Simple LZ77 compression library for vectors of data.
//! @bug No known bugs.
//!

use alloc::vec::Vec;

/// The minimum length for a match to be compressed.
const MIN_MATCH_SIZE: usize = 4;

/// The maximum length of a match before it must be terminated. The huffman implementation expects
/// this to be the maximum.
const MAX_MATCH_SIZE: usize = 1 << 15;

/// The window size of the item being compressed to look backward in.
const WINDOW_SIZE: usize = 1 << 15;

/// A token in lz77 compressed output.
#[derive(Copy, Clone, PartialEq, Eq)]
pub enum Token {
    Phrase(u8),
    Offset(u16),
    Stop
}

/// The input window used as a scratch space during compression.
struct Window {
    front: usize,
    back: usize,
    size: usize,
    buf: [u8; WINDOW_SIZE]
}

/// Manages a group of matches found within the window.
struct MatchGroup {
    offsets: Vec<u16>,
    stream: Vec<u8>
}

/// Compresses the given byte stream.
pub fn compress(
    data: &[u8]
) -> Vec<Token> {
    let mut win = Window { front: 0, back: 0, size: 0, buf: [0; WINDOW_SIZE] };
    let mut current_match: Option<MatchGroup> = None;
    let mut ret = Vec::new();

    for b in data.iter() {
        'compress_byte: {
            if let Some(mut group) = current_match.take() {
                let mut new_matches = Vec::new();
                for offset in group.offsets.iter() {
                    if win.buf[(win.front + (win.size - (*offset as usize))) % WINDOW_SIZE] == *b {
                        new_matches.push(*offset);
                    }
                }

                // If anything matched and we can continue matching, move on to the next byte.
                if new_matches.len() > 0 && group.stream.len() < MAX_MATCH_SIZE {
                    group.stream.push(*b);
                    current_match = Some(MatchGroup { offsets: new_matches, stream: group.stream });
                    break 'compress_byte;
                }

                // Otherwise, add the bytes to the compressed output. Only streams which reach the
                // minimum size will be emited as compressed data.
                if group.stream.len() < MIN_MATCH_SIZE {
                    for phrase in group.stream.iter() {
                        ret.push(Token::Phrase(*phrase));
                    }
                } else {
                    ret.push(Token::Offset(group.offsets[0]));
                    ret.push(Token::Offset(group.stream.len().try_into().unwrap()));
                }
            }

            // Searches the window buffer for a copy of the current byte. We only reach here if
            // this is the first byte or matching couldn't find this byte in the current run.
            //
            // Any offsets found are pushed into a vector which is transformed into a new match
            // group object. If none are found, the byte is added as raw output.
            let mut offsets: Vec<u16> = Vec::new();
            for i in 0..win.size {
                if win.buf[(win.front + i) % WINDOW_SIZE] == *b {
                    offsets.push((win.size - i).try_into().unwrap());
                }
            }

            if offsets.len() > 0 {
                current_match = Some(MatchGroup { offsets, stream: alloc::vec![*b] });
            } else {
                ret.push(Token::Phrase(*b));
            }
        }

        // Update the window with the newly processed input byte.
        win.buf[win.back] = *b;
        win.back = (win.back + 1) % WINDOW_SIZE;
        win.size += 1;

        if win.size > WINDOW_SIZE {
            assert!(win.size == WINDOW_SIZE + 1);
            win.size = WINDOW_SIZE;
            win.front = (win.front + 1) % WINDOW_SIZE;
            assert!(win.front == win.back);
        }
    }

    if let Some(group) = current_match {
        ret.push(Token::Offset(group.offsets[0]));
        ret.push(Token::Offset(group.stream.len().try_into().unwrap()));
    }

    ret.push(Token::Stop);
    return ret;
}

/// Decompresses the given byte stream.
pub fn decompress(
    data: &[Token]
) -> Vec<u8> {
    let mut ret = Vec::new();
    let mut iter = data.iter();

    while let Some(token) = iter.next() {
        match token {
            Token::Phrase(b) => ret.push(*b),
            Token::Offset(offset) => {
                let len = match iter.next() {
                    Some(Token::Offset(len)) => len,
                    _ => panic!("Malformed decompression data!")
                };

                for _ in 0..*len {
                    ret.push(ret[ret.len() - (*offset as usize)]);
                }
            },
            Token::Stop => {
                assert!(iter.next().is_none());
                break;
            }
        }
    }

    return ret;
}
