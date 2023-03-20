//!
//! @file lz.rs
//! @author Andrew Spaulding (Kasplat)
//! @brief Simple LZ77 compression library for vectors of data.
//! @bug No known bugs.
//!

use crate::circ::*;
use crate::serial;

/// The minimum length for a match to be compressed.
const MIN_MATCH_SIZE: usize = 4;

/// The input string size considered by the input window.
const WINDOW_INPUT: usize = 16;

/// The window size of the item being compressed to look backward in.
const WINDOW_BUF: usize = 1 << 14;

/// A token in lz77 compressed output.
pub enum Token {
    Phrase(u8),
    Offset(u16),
    Stop
}

/// A non-compressed literal byte string stored immediately after this struct in memory.
#[repr(C)]
struct Literal {
    len: isize
}

/// A compressed byte string found by looking backward in the input buffer.
#[repr(C)]
struct Lookup {
    offset: isize,
    len: usize
}

/// The input window used as a scratch space during compression.
struct Window {
    input: CircQueue<WINDOW_INPUT>,
    buf: CircQueue<WINDOW_BUF>
}

/// Manages a group of matches found within the window.
struct MatchGroup {
    matches: Vec<usize>,
    len: usize
}

impl Window {
    const fn new() -> Self {
        Self {
            input: CircQueue::new(),
            buf: CircQueue::new()
        }
    }

    /// Checks if the input to the window has been fully processed yet.
    fn has_input(
        &self
    ) -> bool {
        !self.input.is_empty()
    }

    ///
    /// Enqueues a byte to the input buffer, pushing a byte to the window buffer if
    /// the input buffer is full.
    ///
    /// If a byte was moved out of the input buffer, returns that byte.
    ///
    fn enq(
        &mut self,
        input: u8
    ) -> Option<u8> {
        if let Some(deq) = self.input.enq(input) {
            self.buf.enq(deq);
            Some(deq)
        } else {
            None
        }
    }

    /// Finds the longest match between the input and window buffers.
    fn find_match(
        &mut self,
        stream_index: usize
    ) -> Option<(usize, MatchGroup)> {
        // Can't possibly find a long enough match.
        if self.input.len() < MIN_MATCH_SIZE {
            return None;
        }

        // As an optimization, we don't emit matches unless they are
        // actually long enough to save space.
        let stream_index = stream_index - self.input.len();
        let base = stream_index - self.buf.len();
        let mut matches = MatchGroup::new(MIN_MATCH_SIZE);
        let mut i = 0;

        while i < self.buf.len() {
            let mut j = 0;
            while (j < self.input.len()) && ((i + j) < self.buf.len()) {
                if self.buf[i + j] == self.input[j] {
                    j += 1;
                } else {
                    break;
                }
            }

            if j == matches.len() {
                matches.add(base + i);
            } else if j > matches.len() {
                matches = MatchGroup::new(j);
                matches.add(base + i);
            }

            i += 1;
        }

        if matches.matches() > 0 {
            for _ in 0..matches.len() { self.buf.enq(self.input.deq().unwrap()); }
            Some((stream_index, matches))
        } else {
            None
        }
    }

    /// Peeks the next byte to be removed from the input buffer.
    fn peek_input(
        &self
    ) -> u8 {
        self.input[0]
    }

    /// Moves a byte from the input buffer to the window buffer.
    fn drain_one(
        &mut self
    ) -> Option<u8> {
        if let Some(drain) = self.input.deq() {
            self.buf.enq(drain);
            Some(drain)
        } else {
            None
        }
    }
}

impl MatchGroup {
    fn new(
        len: usize
    ) -> Self {
        Self {
            matches: Vec::new(),
            len
        }
    }

    /// Gets the length of the matches in the group.
    fn len(
        &self
    ) -> usize {
        self.len
    }

    /// Gets the number of matches currently being considered by this group.
    fn matches(
        &self
    ) -> usize {
        self.matches.len()
    }

    /// Gets the current (index, len) in the match group.
    fn get(
        &self
    ) -> (usize, usize) {
        (self.matches[0], self.len)
    }

    /// Adds a new index to the given match group.
    fn add(
        &mut self,
        index: usize
    ) {
        self.matches.push(index);
    }

    ///
    /// Updates the match of each index based on the given byte and data stream.
    ///
    /// If no matches are found, the index and len of one of the groups is returned.
    ///
    fn next(
        &mut self,
        b: u8,
        data: &[u8]
    ) -> Result<(), (usize, usize)> {
        assert!(self.matches.len() > 0);
        let mut matches = Vec::new();

        for i in self.matches.iter() {
            if b == data[i + self.len] {
                matches.push(*i);
            }
        }

        if matches.len() > 0 {
            self.matches = matches;
            self.len += 1;
            Ok(())
        } else {
            Err((self.matches[0], self.len))
        }
    }
}

/// Compresses the given byte stream.
pub fn compress(
    data: &[u8]
) -> Vec<Token> {
    enum State { Literal, Match { base: usize, group: MatchGroup } }

    let mut state = State::Literal;
    let mut win = Window::new();
    let mut out = Vec::new();
    let mut i = 0;

    while (i < data.len()) || win.has_input() {
        let (drain, deq) = if i < data.len() {
            i += 1;
            (false, win.enq(data[i - 1]))
        } else {
            (true, Some(win.drain_one().unwrap()))
        };

        match &mut state {
            State::Literal => {
                if let Some(b) = deq {
                    out.push(Token::Phrase(b));
                }

                if let Some((base, group)) = win.find_match(i) {
                    state = State::Match { base, group };
                }
            },
            State::Match { base, group } => {
                assert!((!drain && deq.is_none()) || (drain && deq.is_some()));
                let b = if drain { deq.unwrap() } else { win.peek_input() };

                match group.next(b, data) {
                    Ok(()) => {
                        if !drain {
                            win.drain_one();
                        }
                    }
                    Err((index, len)) => {
                        assert!(index < *base);
                        let offset = TryInto::<u16>::try_into(*base - index).unwrap();
                        out.push(Token::Offset(offset));
                        out.push(Token::Offset(len.try_into().unwrap()));
                        if drain {
                            out.push(Token::Phrase(b));
                        }
                        state = State::Literal;
                    }
                }
            }
        }
    }

    // Flush final state.
    match state {
        State::Literal => (),
        State::Match { base, group } => {
            let (index, len) = group.get();
            assert!(index < base);
            let offset = TryInto::<u16>::try_into(base - index).unwrap();
            out.push(Token::Offset(offset));
            out.push(Token::Offset(len.try_into().unwrap()));
        }
    }

    out.push(Token::Stop);
    return out;
}

/// Decompresses the given byte stream.
pub fn decompress(
    data: &[u8]
) -> Vec<u8> {
    let mut out = Vec::new();
    let mut i = 0;

    while i < data.len() {
        let (r, meta) = serial::read(&data[i..]);
        i += r;

        if meta >= 0 {
            assert!(meta != 0);
            for _ in 0..meta {
                out.push(data[i]);
                i += 1;
            }
        } else {
            let (r, len) = serial::read(&data[i..]);
            let offset = (-meta) as usize;
            let len = len as usize;
            i += r;

            assert!(offset <= out.len());
            assert!(len >= MIN_MATCH_SIZE);

            let base = out.len().wrapping_sub(offset);
            let limit = base + len;
            for j in base..limit {
                out.push(out[j]);
            }
        }
    }

    assert!(i == data.len());
    return out;
}
