use std::ptr;

#[derive(Debug, Clone, Copy)]
pub struct SliceFifo<T, const N: usize> {
    head: usize,
    len: usize,
    data: [T; N],
}

impl<T: Copy, const N: usize> SliceFifo<T, N> {
    pub fn new() -> Self {
        Self {
            head: 0,
            len: 0,
            data: [unsafe { std::mem::zeroed() }; N],
        }
    }

    /// Get a `SliceFifo` from a slice of length M, where M <= N.
    /// 
    /// It does not make any assumptions in production about the length of the slice.
    pub unsafe fn from_slice(slice: &[T]) -> Self {
        debug_assert!(slice.len() <= N, "SliceFifo::from_slice: slice is too long!");
        let mut data = [unsafe { std::mem::zeroed() }; N];
        ptr::copy_nonoverlapping(slice.as_ptr(), data.as_mut_ptr(), slice.len());
        Self {
            head: 0,
            len: slice.len(),
            data,
        }
    }

    pub fn push(&mut self, item: T) {
        let tail = (self.head + self.len) % N;
        unsafe {
            *self.data.get_unchecked_mut(tail) = item;
        }
        if self.len == N {
            self.head = (self.head + 1) % N;
        } else {
            self.len += 1;
        }
    }

    pub fn push_slice(&mut self, slice: &[T]) {
        debug_assert!(self.len + slice.len() <= N, "SliceFifo::push_slice: slice is too long!");
        for item in slice {
            self.push(*item);
        }
    }

    /// Skips bounds checking. Use with caution!
    pub unsafe fn pop_unchecked(&mut self) -> T {
        let item = self.data.get_unchecked(self.head);
        self.head = (self.head + 1) % N;
        self.len -= 1;
        *item
    }

    pub fn as_slice(&self) -> &[T] {
        let tail = (self.head + self.len) % N;
        if self.head <= tail {
            &self.data[self.head..tail]
        } else {
            &self.data[self.head..N]
        }
    }

    pub fn iter(&self) -> impl Iterator<Item = &T> {
        self.as_slice().iter()
    }

    pub const fn is_empty(&self) -> bool {
        self.len == 0
    }
}

impl<T: Copy, const N: usize> FromIterator<T> for SliceFifo<T, N> {
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        let mut fifo = Self::new();
        for item in iter {
            fifo.push(item);
        }
        fifo
    }
}
