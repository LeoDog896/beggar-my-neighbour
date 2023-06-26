use std::mem::MaybeUninit;

/// An optimized structure trading off memory for speed.
/// It is a slice that has a cursor that navigates around, which only supports push and clear.
#[derive(Debug, Clone, Copy)]
pub struct CursorSlice<T: Copy, const N: usize> {
    data: [MaybeUninit<T>; N],
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
        let ptr = self.data.get_unchecked_mut(self.cursor) as *mut MaybeUninit<T> as *mut T;
        ptr.write(value);
        self.cursor += 1;
    }

    pub fn slice(&self) -> &[T] {
        let slice = &self.data[..self.cursor];
        unsafe { &*(slice as *const [MaybeUninit<T>] as *const [T]) }
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
