use std::{mem::MaybeUninit, ptr::copy_nonoverlapping};

/// Capacity for all `CircularBuffers`.
/// This size satisfies the following restrictions:
/// - Capacity MUST be a power of 2 (for fast modulo).
/// - Capacity should be >= the size of the deck (52 cards).
const CAPACITY: usize = 64;

// TODO: optimize by virtual memory
/// A circular buffer.
#[derive(Debug, Clone, Copy)]
pub struct CircularBuffer<T> {
    head: usize,
    len: usize,
    data: [T; CAPACITY],
}

impl<T: Copy> CircularBuffer<T> {
    pub const fn new() -> Self {
        Self {
            head: 0,
            len: 0,
            data: unsafe { MaybeUninit::uninit().assume_init() },
        }
    }

    /// Get a `SliceFifo` from a pointer to a slice of length M, where M <= N.
    ///
    /// It does not make any assumptions in production about the length of the slice.
    pub unsafe fn from_memory(source: *const T, len: usize) -> Self {
        debug_assert!(len <= CAPACITY, "SliceFifo::from_slice: slice is too long!");
        let mut data = [std::mem::zeroed(); CAPACITY];
        copy_nonoverlapping(source, data.as_mut_ptr(), len);
        Self { head: 0, len, data }
    }

    pub unsafe fn push(&mut self, item: T) {
        debug_assert!(
            self.len < CAPACITY,
            "SliceFifo::push: slice is full! (len = {}, CAPACITY = {})",
            self.len,
            CAPACITY
        );

        // We use a bitwise operator where N is a power of 2 instead of modulo
        let tail = (self.head + self.len) & (CAPACITY - 1);

        // This is safe because we know that the length of the slice is less than N (because of % N)
        *self.data.get_unchecked_mut(tail) = item;

        // But this is not safe, because we don't know if the slice is full or not
        self.len += 1;
    }

    pub unsafe fn push_slice(&mut self, slice: &[T]) {
        debug_assert!(
            self.len + slice.len() <= CAPACITY,
            "SliceFifo::push_slice: slice is too long!"
        );

        debug_assert!(!slice.is_empty(), "SliceFifo::push_slice: slice is empty!");

        // We use a bitwise operator where N is a power of 2 instead of modulo
        let tail = (self.head + self.len) & (CAPACITY - 1);
        if slice.len() > CAPACITY - tail {
            // We need to split the slice into two parts
            copy_nonoverlapping(
                slice.as_ptr(),
                self.data.as_mut_ptr().add(tail),
                CAPACITY - tail,
            );
            copy_nonoverlapping(
                slice.as_ptr().add(CAPACITY - tail),
                self.data.as_mut_ptr(),
                slice.len() - (CAPACITY - tail),
            );
        } else {
            // We can just copy the slice into the buffer
            copy_nonoverlapping(
                slice.as_ptr(),
                self.data.as_mut_ptr().add(tail),
                slice.len(),
            );
        }
        self.len += slice.len();
    }

    /// Skips bounds checking. If the buffer is empty, this will be UB.
    pub unsafe fn pop_unchecked(&mut self) -> T {
        debug_assert!(self.len > 0, "SliceFifo::pop_unchecked: slice is empty!");
        let item = self.data.get_unchecked(self.head);
        if self.head == CAPACITY - 1 {
            self.head = 0;
        } else {
            self.head += 1;
        }
        self.len -= 1;
        *item
    }

    pub fn slice(&self) -> &[T] {
        let tail = (self.head + self.len) % CAPACITY;
        if self.head <= tail {
            &self.data[self.head..tail]
        } else {
            &self.data[self.head..CAPACITY]
        }
    }

    pub fn iter(&self) -> impl Iterator<Item = &T> {
        self.slice().iter()
    }

    pub const fn len(&self) -> usize {
        self.len
    }
}

impl<T: Copy> FromIterator<T> for CircularBuffer<T> {
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        let mut fifo = Self::new();
        for item in iter {
            unsafe {
                fifo.push(item);
            }
        }
        fifo
    }
}
