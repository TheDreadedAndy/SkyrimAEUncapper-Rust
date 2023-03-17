//!
//! @file circ.rs
//! @author Andrew Spaulding (Kasplat)
//! @brief Circular buffer implementation for the compression library.
//! @bug No known bugs.
//!

use core::ops::Index;

/// The circular buffer used in the input window for compression.
pub struct CircQueue<const SIZE: usize> {
    front: usize,
    back: usize,
    size: usize,
    buf: [u8; SIZE]
}

impl<const SIZE: usize> CircQueue<SIZE> {
    pub const fn new() -> Self {
        Self {
            front: 0,
            back: 0,
            size: 0,
            buf: [0; SIZE]
        }
    }

    /// Checks if the buffer is empty.
    pub const fn is_empty(
        &self
    ) -> bool {
        self.size == 0
    }

    /// Checks if the buffer is full.
    pub const fn is_full(
        &self
    ) -> bool {
        self.size == SIZE
    }

    /// Gets the current size of the buffer.
    pub const fn len(
        &self
    ) -> usize {
        self.size
    }

    /// Enqueues a new byte to the buffer, returning the evicted byte.
    pub fn enq(
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
    pub fn deq(
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
