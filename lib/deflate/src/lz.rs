//!
//! @file lz.rs
//! @author Andrew Spaulding (Kasplat)
//! @brief Simple LZ77 compression library for vectors of data.
//! @bug No known bugs.
//!

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

impl Window {
    const fn new() -> Self {
        Self { front: 0, back: 0, size: 0, buf: [0; WINDOW_SIZE] }
    }

    /// Adds a byte to the window buffer so that it may be matched by the next byte.
    fn enq(
        &mut self,
        input: u8
    ) {
        self.buf[self.back] = input;
        self.back = (self.back + 1) % WINDOW_SIZE;
        self.size += 1;

        if self.size > WINDOW_SIZE {
            assert!(self.size == WINDOW_SIZE + 1);
            self.size = WINDOW_SIZE;
            self.front = (self.front + 1) % WINDOW_SIZE;
            assert!(self.front == self.back);
        }
    }

    /// Matches the current set of offsets to the next byte in the sequence, or returns one
    /// of the sequences if there is no match.
    fn match_next(
        &self,
        next: u8,
        mut group: MatchGroup
    ) -> Result<MatchGroup, (u16, Vec<u8>)> {
        let mut new_matches = Vec::new();
        for offset in group.offsets.iter() {
            if self.buf[(self.front + (self.size - (*offset as usize))) % WINDOW_SIZE] == next {
                new_matches.push(*offset);
            }
        }

        if new_matches.len() > 0 && group.stream.len() < MAX_MATCH_SIZE {
            group.stream.push(next);
            Ok(MatchGroup { offsets: new_matches, stream: group.stream })
        } else {
            Err((group.offsets[0], group.stream))
        }
    }

    /// Searches the window buffer for a copy of next, returning a match group containing
    /// the offsets which index into any copies.
    fn match_first(
        &self,
        next: u8
    ) -> Result<MatchGroup, ()> {
        let mut offsets: Vec<u16> = Vec::new();
        for i in 0..self.size {
            if self.buf[(self.front + i) % WINDOW_SIZE] == next {
                offsets.push((self.size - i).try_into().unwrap());
            }
        }

        if offsets.len() > 0 {
            Ok(MatchGroup { offsets, stream: vec![next] })
        } else {
            Err(())
        }
    }
}

/// Compresses the given byte stream.
pub fn compress(
    data: &[u8]
) -> Vec<Token> {
    let mut win = Window::new();
    let mut current_match: Option<MatchGroup> = None;
    let mut ret = Vec::new();

    for b in data.iter() {
        if let Some(group) = current_match.take() {
            match win.match_next(*b, group) {
                Ok(group) => {
                    current_match = Some(group);
                    win.enq(*b);
                    continue;
                }
                Err((offset, stream)) => {
                    if stream.len() < MIN_MATCH_SIZE {
                        for phrase in stream.iter() {
                            ret.push(Token::Phrase(*phrase));
                        }
                    } else {
                        ret.push(Token::Offset(offset));
                        ret.push(Token::Offset(stream.len().try_into().unwrap()));
                    }
                }
            }
        }

        match win.match_first(*b) {
            Ok(group) => current_match = Some(group),
            Err(_) => ret.push(Token::Phrase(*b))
        }

        win.enq(*b);
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
