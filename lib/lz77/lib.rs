//!
//! @file lib.rs
//! @author Andrew Spaulding (aspauldi)
//! @brief Simple LZ77 compression library for static data.
//! @bug No known bugs.
//!

use core::ops::Index;

/// The input string size considered by the input window.
const WINDOW_INPUT: usize = 16;

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

/// A lz77 metadata item in a compressed input.
#[repr(C)]
union Meta {
    ty: i16,
    raw: Literal,
    cmp: Lookup
}

/// The input window used as a scratch space during compression.
struct Window {
    input: CircQueue<WINDOW_INPUT>,
    buf: CircQueue<WINDOW_BUF>
}

impl<SIZE> CircQueue<SIZE> {
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
    const fn enq(
        &mut self,
        input: u8
    ) -> Option<u8> {
        let ret = if self.is_full() { Some(self.buf[self.front]) } else { None };

        self.buf[self.back] = input;
        self.back = (self.back + 1) % SIZE;
        self.size += 1;

        if self.size > SIZE {
            self.size = SIZE;
            self.front = (self.front + 1) % SIZE;
        }

        return ret;
    }

    /// Dequeues an element from the buffer, if possible.
    const fn deq(
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

impl<SIZE> Index<usize> for CircQueue<SIZE> {
    type Output = u8;
    fn index(
        &self,
        index: Self::Idx
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
        // TODO
    }
}

impl Lookup {
    /// Emits the match to the byte stream as lz77 metadata.
    fn emit(
        offset: i16,
        len: u16,
        out: &mut Vec<u8>
    ) {
        // TODO
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
    const fn enq(
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
    ) -> Option<Lookup> {
        let mut loc = 0;
        let mut len = 0;
        let mut i = 0;
        let mut j = 0;

        while i < self.buf.len() {
            let mut match_len = 0;
            while j < self.input.len() {
                if self.buf[i] == self.input[j] {
                    match_len += 1;
                } else {
                    break;
                }

                j += 1;
            }

            if match_len > len {
                len = match_len;
                loc = i;
            }

            i += 1;
        }

        // As an optimization, we don't emit matches unless they are
        // actually long enough to save space.
        if len > std::mem::size_of::<Meta>() {
            let ret = Lookup { offset: -(self.buf.len() - loc) as i16, len: len };
            for _ in 0..len { self.buf.enq(self.input.deq().unwrap()); }
            Some(ret)
        } else {
            None
        }
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

        match state {
            State::Literal(v) => {
                if let Some(m) = win.find_match() {
                    Literal::emit(v.slice(), &mut out);
                    *state = State::Match { index: i, offset: m.offset, len: m.len };
                } else if let Some(b) = deq {
                    v.push_back(b);
                }
            },
            State::Match { index, offset, len } => {
                let mnext = data[*index + (*len as usize)];
                if mnext == data[i] {
                    *len += 1;
                    assert!(win.flush_input() == 1);
                } else {
                    Lookup::emit(*offset, *len, &mut out);
                    *state = State::Literal(Vec::new());
                }
            }
        }

        i += 1;
    }

    // Flush final state.
    match state {
        State::Literal(v) => Literal::emit(v.slice(), &mut out),
        State::Match { offset, len, .. } => Lookup::emit(*offset, *len, &mut out)
    }

    return out;
}

/// Decompresses the given byte stream.
pub fn decompress(
    data: &[u8]
) -> Vec<u8> {
}
