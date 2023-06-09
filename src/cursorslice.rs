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
            data: [unsafe { std::mem::zeroed() }; N],
            cursor: 0,
        }
    }

    pub unsafe fn push_unchecked(&mut self, value: T) {
        // This is fully unsafe! We are assuming that the cursor is always in bounds in release mode.
        debug_assert!(self.cursor < N, "CursorSlice is full!");
        unsafe {
            *self.data.get_unchecked_mut(self.cursor) = value;
        };
        self.cursor += 1;
    }

    pub fn slice(&self) -> &[T] {
        unsafe { self.data.get_unchecked(..self.cursor) }
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
