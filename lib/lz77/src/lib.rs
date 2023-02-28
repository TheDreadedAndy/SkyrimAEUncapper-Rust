//!
//! @file lib.rs
//! @author Andrew Spaulding (aspauldi)
//! @brief Simple LZ77 compression library for static data.
//! @bug No known bugs.
//!

mod circ;
mod serial;

use circ::*;

/// The minimum length for a match to be compressed.
const MIN_MATCH_SIZE: usize = 4;

/// The minimum length for a literal before trying to match again.
const MIN_LIT_SIZE: usize = 1;

/// The input string size considered by the input window.
const WINDOW_INPUT: usize = 4096;

/// The window size of the item being compressed to look backward in.
const WINDOW_BUF: usize = 4096;

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

impl Literal {
    /// Emits the literal data to the byte stream as lz77 metadata.
    fn emit(
        lit: &[u8],
        out: &mut Vec<u8>
    ) {
        if lit.len() == 0 {
            return;
        }

        serial::write(lit.len().try_into().unwrap(), out);
        out.extend_from_slice(lit);
    }
}

impl Lookup {
    /// Emits the match to the byte stream as lz77 metadata.
    fn emit(
        offset: isize,
        len: usize,
        out: &mut Vec<u8>
    ) {
        assert!(offset < 0);
        serial::write(offset, out);
        serial::write(len as isize, out);
    }
}

impl Window {
    const fn new() -> Self {
        Self {
            input: CircQueue::new(),
            buf: CircQueue::new()
        }
    }

    /// Gets the total number of bytes in the window and input buffers.
    fn len(
        &self
    ) -> usize {
        self.input.len() + self.buf.len()
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
        &mut self
    ) -> Option<(usize, Lookup)> {
        // Can't possibly find a long enough match.
        if self.input.len() < MIN_MATCH_SIZE {
            return None;
        }

        let mut loc = 0;
        let mut len = 0;
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

            if j > len {
                len = j;
                loc = i;
                if len == WINDOW_INPUT {
                    break;
                }
            }

            i += 1;
        }

        // As an optimization, we don't emit matches unless they are
        // actually long enough to save space.
        if len >= MIN_MATCH_SIZE {
            let offset = (self.buf.len() + self.input.len()) - loc;
            let loc = Lookup {
                offset: -TryInto::<isize>::try_into(self.buf.len() - loc).unwrap(),
                len: len
            };

            for _ in 0..len { self.buf.enq(self.input.deq().unwrap()); }

            Some((offset, loc))
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

/// Compresses the given byte stream.
pub fn compress(
    data: &[u8]
) -> Vec<u8> {
    enum State { Literal(Vec<u8>), Match { index: usize, offset: isize, len: usize } }

    let mut state = State::Literal(Vec::new());
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
            State::Literal(v) => {
                if let Some(b) = deq {
                    v.push(b);
                }

                if (v.len() >= MIN_LIT_SIZE) || (v.len() == 0) {
                    if let Some((ioffset, m)) = win.find_match() {
                        Literal::emit(v.as_slice(), &mut out);
                        state = State::Match { index: i - ioffset, offset: m.offset, len: m.len };
                    }
                }
            },
            State::Match { index, offset, len } => {
                assert!((!drain && deq.is_none()) || (drain && deq.is_some()));
                let b = if drain { deq.unwrap() } else { win.peek_input() };

                if b == data[*index + (*len as usize)] {
                    *len += 1;
                    if !drain {
                        win.drain_one();
                    }
                } else {
                    Lookup::emit(*offset, *len, &mut out);
                    state = State::Literal(if drain {
                        vec![b]
                    } else {
                        Vec::new()
                    });
                }
            }
        }
    }

    // Flush final state.
    match state {
        State::Literal(v) => Literal::emit(v.as_slice(), &mut out),
        State::Match { offset, len, .. } => Lookup::emit(offset, len, &mut out)
    }

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
            let len = len as usize;
            i += r;
            assert!((-meta as usize) <= out.len());
            assert!(len >= MIN_MATCH_SIZE);

            let base = out.len().wrapping_sub(-meta as usize);
            let limit = base + len;
            for j in base..limit {
                out.push(out[j]);
            }
        }
    }

    assert!(i == data.len());
    return out;
}
