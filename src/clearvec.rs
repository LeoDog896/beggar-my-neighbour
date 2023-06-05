/// An optimized structure trading off memory for speed.
/// It is a slice that has a cursor that navigates around, which only supports push and clear.

use std::{marker::PhantomData, mem, ptr::{NonNull, self}};

struct RawValIter<T> {
    start: *const T,
    end: *const T,
}

impl<T> RawValIter<T> {
    // unsafe to construct because it has no associated lifetimes.
    // This is necessary to store a RawValIter in the same struct as
    // its actual allocation. OK since it's a private implementation
    // detail.
    unsafe fn new(slice: &[T]) -> Self {
        RawValIter {
            start: slice.as_ptr(),
            end: if slice.len() == 0 {
                // if `len = 0`, then this is not actually allocated memory.
                // Need to avoid offsetting because that will give wrong
                // information to LLVM via GEP.
                slice.as_ptr()
            } else {
                slice.as_ptr().add(slice.len())
            }
        }
    }
}

impl<T> Iterator for RawValIter<T> {
    type Item = T;
    fn next(&mut self) -> Option<T> {
        if self.start == self.end {
            None
        } else {
            unsafe {
                if mem::size_of::<T>() == 0 {
                    self.start = (self.start as usize + 1) as *const _;
                    Some(ptr::read(NonNull::<T>::dangling().as_ptr()))
                } else {
                    let old_ptr = self.start;
                    self.start = self.start.offset(1);
                    Some(ptr::read(old_ptr))
                }
            }
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let elem_size = mem::size_of::<T>();
        let len = (self.end as usize - self.start as usize)
                  / if elem_size == 0 { 1 } else { elem_size };
        (len, Some(len))
    }
}

pub struct Drain<'a, T: 'a> {
    vec: PhantomData<&'a mut Vec<T>>,
    iter: RawValIter<T>,
}

impl<'a, T> Iterator for Drain<'a, T> {
    type Item = T;
    fn next(&mut self) -> Option<T> { self.iter.next() }
    fn size_hint(&self) -> (usize, Option<usize>) { self.iter.size_hint() }
}

impl<'a, T> Drop for Drain<'a, T> {
    fn drop(&mut self) {
        for _ in &mut *self {}
    }
}

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
        // This is fully unsafe! We are assuming that the cursor is always in bounds in release mode.
        debug_assert!(self.cursor < N, "ClearVec is full!");
        unsafe { 
            *self.data.get_unchecked_mut(self.cursor) = value 
        };
        self.cursor += 1;
    }

    pub fn drain(&mut self) -> Drain<T> {
        unsafe {
            let iter = RawValIter::new(&self.data[..self.cursor]);

            self.cursor = 0;

            Drain {
                iter,
                vec: PhantomData,
            }
        }
    }

    pub fn iter(&self) -> impl Iterator<Item = &T> {
        self.data[..self.cursor].iter()
    }

    pub const fn is_empty(&self) -> bool {
        self.cursor == 0
    }
}
