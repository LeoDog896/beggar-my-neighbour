/// An optimized structure trading off memory for speed.
/// It is a slice that has a cursor that navigates around, which only supports push and clear.

#[derive(Debug, Clone, Copy)]
pub struct ClearVec<T: Copy, const N: usize> {
    data: [T; N],
    cursor: usize,
}

impl<T: Copy, const N: usize> ClearVec<T, N> {
    pub fn new() -> Self {
        Self {
            data: [unsafe { std::mem::zeroed() }; N],
            cursor: 0,
        }
    }

    pub fn push(&mut self, value: T) {
        self.data[self.cursor] = value;
        self.cursor += 1;
    }

    pub fn clear(&mut self) {
        self.cursor = 0;
    }

    pub fn as_slice(&self) -> &[T] {
        &self.data[..self.cursor]
    }

    pub fn iter(&self) -> impl Iterator<Item = &T> {
        self.as_slice().iter()
    }

    pub const fn is_empty(&self) -> bool {
        self.cursor == 0
    }
}
