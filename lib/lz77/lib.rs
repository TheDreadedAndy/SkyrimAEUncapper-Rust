//!
//! @file lib.rs
//! @author Andrew Spaulding (aspauldi)
//! @brief Simple LZ77 compression library for static data.
//! @bug No known bugs.
//!

use core::ops::Index;

/// The input string size considered by the input window.
const WINDOW_INPUT: usize = 4096;

/// The window size of the item being compressed to look backward in.
const WINDOW_BUF: usize = 4096;

/// The circular buffer used in the input window for compression.
struct CircQueue<const SIZE: usize> {
    front: usize,
    back: usize,
    size: usize,
    buf: [u8; SIZE]
}

/// A non-compressed literal byte string stored immediately after this struct in memory.
#[repr(C)]
struct Literal {
    len: i16
}

/// A compressed byte string found by looking backward in the input buffer.
#[repr(C)]
struct Lookup {
    offset: i16,
    len: u16
}

/// The input window used as a scratch space during compression.
struct Window {
    input: CircQueue<WINDOW_INPUT>,
    buf: CircQueue<WINDOW_BUF>
}

impl<const SIZE: usize> CircQueue<SIZE> {
    const fn new() -> Self {
        Self {
            front: 0,
            back: 0,
            size: 0,
            buf: [0; SIZE]
        }
    }

    /// Checks if the buffer is empty.
    const fn is_empty(
        &self
    ) -> bool {
        self.size == 0
    }

    /// Checks if the buffer is full.
    const fn is_full(
        &self
    ) -> bool {
        self.size == SIZE
    }

    /// Gets the current size of the buffer.
    const fn len(
        &self
    ) -> usize {
        self.size
    }

    /// Enqueues a new byte to the buffer, returning the evicted byte.
    fn enq(
        &mut self,
        input: u8
    ) -> Option<u8> {
        let ret = if self.is_full() { Some(self.buf[self.front]) } else { None };

        self.buf[self.back] = input;
        self.back = (self.back + 1) % SIZE;
        self.size += 1;

        if self.size > SIZE {
            assert!(self.size == SIZE + 1);
            self.size = SIZE;
            self.front = (self.front + 1) % SIZE;
            assert!(self.front == self.back);
        }

        return ret;
    }

    /// Dequeues an element from the buffer, if possible.
    fn deq(
        &mut self
    ) -> Option<u8> {
        if !self.is_empty() {
            let ret = self.buf[self.front];
            self.front = (self.front + 1) % SIZE;
            self.size -= 1;
            Some(ret)
        } else {
            None
        }
    }
}

impl<const SIZE: usize> Index<usize> for CircQueue<SIZE> {
    type Output = u8;
    fn index(
        &self,
        index: usize
    ) -> &Self::Output {
        assert!(index < self.size);
        &self.buf[(self.front + index) % SIZE]
    }
}

impl Literal {
    /// Emits the literal data to the byte stream as lz77 metadata.
    fn emit(
        lit: &[u8],
        out: &mut Vec<u8>
    ) {
        assert!(lit.len() < i16::MAX as usize);

        if lit.len() == 0 {
            return;
        }

        let len: u16 = lit.len().try_into().unwrap();
        out.push((len & 0xff) as u8);
        out.push((len >> 8) as u8);
        out.extend_from_slice(lit);
    }
}

impl Lookup {
    /// Emits the match to the byte stream as lz77 metadata.
    fn emit(
        offset: i16,
        len: u16,
        out: &mut Vec<u8>
    ) {
        assert!(offset < 0);
        let offset = offset as u16;
        out.push((offset & 0xff) as u8);
        out.push((offset >> 8) as u8);
        out.push((len & 0xff) as u8);
        out.push((len >> 8) as u8);
    }
}

impl Window {
    const fn new() -> Self {
        Self {
            input: CircQueue::new(),
            buf: CircQueue::new()
        }
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
        if self.input.len() < std::mem::size_of::<Lookup>() {
            return None;
        }

        let mut loc = 0;
        let mut len = 0;
        let mut i = 0;
        let mut j = 0;

        while i < self.buf.len() {
            let mut match_len = 0;
            while (j < self.input.len()) && ((i + j) < self.buf.len()) {
                if self.buf[i + j] == self.input[j] {
                    match_len += 1;
                } else {
                    break;
                }

                j += 1;
            }

            if match_len > len {
                len = match_len;
                loc = i;
                if len == WINDOW_INPUT {
                    break;
                }
            }

            i += 1;
        }

        // As an optimization, we don't emit matches unless they are
        // actually long enough to save space.
        if len >= std::mem::size_of::<Lookup>() {
            let offset = (self.buf.len() + self.input.len()) - loc;
            let loc = Lookup {
                offset: (-TryInto::<isize>::try_into(offset - (self.input.len() as usize)).unwrap()).try_into().unwrap(),
                len: len.try_into().unwrap()
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
    ) {
        self.buf.enq(self.input.deq().unwrap());
    }
}

/// Compresses the given byte stream.
pub fn compress(
    data: &[u8]
) -> Vec<u8> {
    enum State { Literal(Vec<u8>), Match { index: usize, offset: i16, len: u16 } }

    let mut state = State::Literal(Vec::new());
    let mut win = Window::new();
    let mut out = Vec::new();
    let mut i = 0;

    while i < data.len() {
        let deq = win.enq(data[i]);
        i += 1;

        match &mut state {
            State::Literal(v) => {
                if let Some(b) = deq {
                    v.push(b);
                }

                if let Some((ioffset, m)) = win.find_match() {
                    Literal::emit(v.as_slice(), &mut out);
                    state = State::Match { index: i - ioffset, offset: m.offset, len: m.len };
                }
            },
            State::Match { index, offset, len } => {
                assert!(deq.is_none());
                if win.peek_input() == data[*index + (*len as usize)] {
                    *len += 1;
                    win.drain_one();
                } else {
                    Lookup::emit(*offset, *len, &mut out);
                    state = State::Literal(Vec::new());
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
        let meta: i16 = data[i] as i16 | ((data[i + 1] as i16) << 8);
        i += 2;

        if meta >= 0 {
            assert!(meta != 0);
            for _ in 0..meta {
                out.push(data[i]);
                i += 1;
            }
        } else {
            let len: u16 = data[i] as u16 | ((data[i + 1] as u16) << 8);
            i += 2;
            assert!(len as usize >= core::mem::size_of::<Lookup>());
            let base = ((out.len() as isize) + (meta as isize)) as usize;
            let limit = out.len() + (len as usize);
            for j in base..limit {
                out.push(out[j]);
            }
        }
    }

    assert!(i == data.len());
    return out;
}
