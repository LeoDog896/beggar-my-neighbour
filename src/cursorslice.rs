use std::mem::MaybeUninit;

/// An optimized structure trading off memory for speed.
/// It is a slice that has a cursor that navigates around, which only supports push and clear.
#[derive(Debug, Clone, Copy)]
pub struct CursorSlice<T: Copy, const N: usize> {
    data: [T; N],
    cursor: usize,
}

impl<T: Copy, const N: usize> CursorSlice<T, N> {
    pub fn new() -> Self {
        Self {
            data: unsafe { MaybeUninit::uninit().assume_init() },
            cursor: 0,
        }
    }

    /// This is fully unsafe! We are assuming that the cursor is always in bounds in release mode.
    pub unsafe fn push_unchecked(&mut self, value: T) {
        debug_assert!(self.cursor < N, "CursorSlice is full!");
        *self.data.get_unchecked_mut(self.cursor) = value;
        self.cursor += 1;
    }

    pub fn slice(&self) -> &[T] {
        &self.data[..self.cursor]
    }

    pub fn clear(&mut self) {
        self.cursor = 0;
    }

    pub fn iter(&self) -> impl Iterator<Item = &T> {
        self.slice().iter()
    }

    pub const fn is_empty(&self) -> bool {
        self.cursor == 0
    }
}
