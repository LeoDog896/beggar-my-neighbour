use std::ptr;

#[derive(Debug, Clone, Copy)]
pub struct CircularBuffer<T, const N: usize> {
    head: usize,
    len: usize,
    data: [T; N],
}

impl<T: Copy, const N: usize> CircularBuffer<T, N> {
    pub fn new() -> Self {
        Self {
            head: 0,
            len: 0,
            data: [unsafe { std::mem::zeroed() }; N],
        }
    }

    /// Get a `SliceFifo` from a pointer to a slice of length M, where M <= N.
    ///
    /// It does not make any assumptions in production about the length of the slice.
    pub unsafe fn from_memory(source: *const T, len: usize) -> Self {
        debug_assert!(
            len <= N,
            "SliceFifo::from_slice: slice is too long!"
        );
        let mut data = [std::mem::zeroed(); N];
        ptr::copy_nonoverlapping(source, data.as_mut_ptr(), len);
        Self {
            head: 0,
            len,
            data,
        }
    }

    pub unsafe fn push(&mut self, item: T) {
        // We use a bitwise operator where N is a power of 2
        let tail = (self.head + self.len) & (N - 1);

        // This is safe because we know that the length of the slice is less than N (because of % N)
        *self.data.get_unchecked_mut(tail) = item;

        // But this is not safe, because we don't know if the slice is full or not
        self.len += 1;
    }

    pub unsafe fn push_slice(&mut self, slice: &[T]) {
        debug_assert!(
            self.len + slice.len() <= N,
            "SliceFifo::push_slice: slice is too long!"
        );

        debug_assert!(
            !slice.is_empty(),
            "SliceFifo::push_slice: slice is empty!"
        );

        // We use a bitwise operator where N is a power of 2
        let tail = (self.head + self.len) & (N - 1);
        if slice.len() > N - tail {
            // We need to split the slice into two parts (unsafe mode)
            ptr::copy_nonoverlapping(slice.as_ptr(), self.data.as_mut_ptr().add(tail), N - tail);
            ptr::copy_nonoverlapping(slice.as_ptr().add(N - tail), self.data.as_mut_ptr(), slice.len() - (N - tail));
        } else {
            // We can just copy the slice into the buffer
            ptr::copy_nonoverlapping(slice.as_ptr(), self.data.as_mut_ptr().add(tail), slice.len());
        }
        self.len += slice.len();
    }

    /// Skips bounds checking. If the buffer is empty, this will be UB.
    pub unsafe fn pop_unchecked(&mut self) -> T {
        let item = self.data.get_unchecked(self.head);
        if self.head == N - 1 {
            self.head = 0;
        } else {
            self.head += 1;
        }
        self.len -= 1;
        *item
    }

    pub fn slice(&self) -> &[T] {
        let tail = (self.head + self.len) % N;
        if self.head <= tail {
            &self.data[self.head..tail]
        } else {
            &self.data[self.head..N]
        }
    }

    pub fn iter(&self) -> impl Iterator<Item = &T> {
        self.slice().iter()
    }

    pub const fn len(&self) -> usize {
        self.len
    }
}

impl<T: Copy, const N: usize> FromIterator<T> for CircularBuffer<T, N> {
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
