/// An optimized structure trading off memory for speed.
/// It is a slice that has a cursor that navigates around, which only supports push and clear.
#[derive(Debug, Clone, Copy)]
pub struct CursorSlice<T: Copy, const N: usize> {
    data: [T; N],
    cursor: *mut T,
}

impl<T: Copy, const N: usize> CursorSlice<T, N> {
    /// Contract: you must call `init` in the function that owns this struct.
    pub unsafe fn new() -> Self {
        Self {
            data: [unsafe { std::mem::zeroed() }; N],
            cursor: unsafe { std::mem::zeroed() },
        }
    }

    
    pub fn init(&mut self) {
        self.cursor = self.data.as_mut_ptr();
    }

    /// This is fully unsafe! We are assuming that the cursor is always in bounds in release mode.
    pub unsafe fn push_unchecked(&mut self, value: T) {
        debug_assert!(self.len() < N, "CursorSlice::push_unchecked: slice is full!");
        *self.cursor = value;
        self.cursor = self.cursor.offset(1);
    }

    pub fn as_head_ptr(&self) -> *const T {
        self.data.as_ptr()
    }

    pub fn len(&self) -> usize {
        (unsafe { self.cursor.offset_from(self.data.as_ptr() as *mut T) }) as usize
    }

    fn slice(&self) -> &[T] {
        unsafe { std::slice::from_raw_parts(self.as_head_ptr(), self.len()) }
    }

    pub fn clear(&mut self) {
        self.cursor = self.data.as_mut_ptr();
    }

    pub fn iter(&self) -> impl Iterator<Item = &T> {
        self.slice().iter()
    }

    pub fn is_empty(&self) -> bool {
        self.cursor == self.data.as_ptr() as *mut T
    }
}
