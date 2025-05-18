use std::mem::MaybeUninit;
use std::ptr;

use super::Lookahead;

pub const LOOKAHEAD_CAPACITY: usize = 4; // ← kept at 4
const LOOKAHEAD_MASK: usize = LOOKAHEAD_CAPACITY - 1;

// Compile-time guard: fails if you ever change CAP to a non-power-of-two.
const _: () = {
    assert!(LOOKAHEAD_CAPACITY.is_power_of_two());
};

#[derive(Debug)]
pub struct LookaheadBuffer<'a> {
    data: [MaybeUninit<Lookahead<'a>>; LOOKAHEAD_CAPACITY],
    head: usize,
    len: usize,
}

impl<'a> LookaheadBuffer<'a> {
    #[inline]
    pub fn new() -> Self {
        // SAFETY: `[MaybeUninit<_>; _]` is valid for any bit pattern.
        let data = unsafe {
            MaybeUninit::<[MaybeUninit<Lookahead<'a>>; LOOKAHEAD_CAPACITY]>::uninit().assume_init()
        };
        Self { data, head: 0, len: 0 }
    }

    #[inline]
    pub fn clear(&mut self) {
        for i in 0..self.len {
            // SAFETY: this is safe
            unsafe { ptr::drop_in_place(self.data[(self.head + i) & LOOKAHEAD_MASK].as_mut_ptr()) }
        }
        self.head = 0;
        self.len = 0;
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.len
    }

    #[inline]
    pub fn get(&self, index: usize) -> Option<&Lookahead<'a>> {
        if index < self.len {
            let ptr = self.data[(self.head + index) & LOOKAHEAD_MASK].as_ptr();
            // SAFETY: this is safe
            Some(unsafe { &*ptr })
        } else {
            None
        }
    }

    #[inline]
    pub fn back(&self) -> Option<&Lookahead<'a>> {
        if self.len == 0 {
            None
        } else {
            let ptr = self.data[(self.head + self.len - 1) & LOOKAHEAD_MASK].as_ptr();
            // SAFETY: this is safe
            Some(unsafe { &*ptr })
        }
    }

    #[inline]
    pub fn push_back(&mut self, value: Lookahead<'a>) {
        assert!(self.len < LOOKAHEAD_CAPACITY);
        let idx = (self.head + self.len) & LOOKAHEAD_MASK;
        self.data[idx].write(value);
        self.len += 1;
    }

    #[inline]
    pub fn pop_front(&mut self) -> Option<Lookahead<'a>> {
        if self.len == 0 {
            None
        } else {
            let idx = self.head;
            self.head = (self.head + 1) & LOOKAHEAD_MASK;
            self.len -= 1;
            // SAFETY: this is safe
            Some(unsafe { self.data[idx].assume_init() })
        }
    }
}

impl Drop for LookaheadBuffer<'_> {
    fn drop(&mut self) {
        self.clear();
    }
}
