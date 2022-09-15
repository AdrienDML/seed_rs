use core::slice;
use std::{
    alloc::{self, Layout},
    marker::PhantomData,
    mem,
    ops::{Index, self},
    ptr::{self, NonNull}, slice::SliceIndex,
};

/// The inner type for the Maximum sized Vector.
struct RawVec<T, const N: usize> {
    ptr: NonNull<T>,
    cap: usize,
    _marker: PhantomData<T>,
}

impl<T, const N: usize> RawVec<T, N> {
    const MAX_CAP: usize = (isize::MAX as usize).min(N);
    pub fn new() -> Self {
        assert!(mem::size_of::<T>() != 0, "TODO: implement ZST support");
        RawVec {
            ptr: NonNull::dangling(),
            cap: 0,
            _marker: PhantomData,
        }
    }

    pub fn grow(&mut self) {
        let (new_cap, new_layout) = if self.cap == 0 {
            (1, Layout::array::<T>(1).unwrap())
        } else {
            // This can't overflow because we ensure self.cap <= isize::MAX.
            let new_cap = usize::min(2 * self.cap, N);

            // Layout::array checks that the number of bytes is <= usize::MAX,
            // but this is redundant since old_layout.size() <= isize::MAX,
            // so the `unwrap` should never fail.
            let new_layout = Layout::array::<T>(new_cap).unwrap();
            (new_cap, new_layout)
        };

        let new_ptr = if self.cap == 0 {
            unsafe { alloc::alloc(new_layout) }
        } else {
            let old_layout = Layout::array::<T>(self.cap).unwrap();
            let old_ptr = self.ptr.as_ptr() as *mut u8;
            unsafe { alloc::realloc(old_ptr, old_layout, new_layout.size()) }
        };

        // If allocation fails, `new_ptr` will be null, in which case we abort.
        self.ptr = match NonNull::new(new_ptr as *mut T) {
            Some(p) => p,
            None => alloc::handle_alloc_error(new_layout),
        };
        self.cap = new_cap;
    }

    pub fn extend(&mut self, count: usize) {
        let new_cap = self.cap + count;
        let new_layout = Layout::array::<T>(new_cap).unwrap();
        let new_ptr = {
            let old_layout = Layout::array::<T>(self.cap).unwrap();
            let old_ptr = self.ptr.as_ptr() as *mut u8;
            unsafe {
                alloc::realloc(old_ptr, old_layout, new_layout.size())
            }
        };
        self.ptr = match NonNull::new(new_ptr as *mut T) {
            Some(p) => p,
            None => alloc::handle_alloc_error(new_layout),
        };
        self.cap = new_cap;
    }
}

impl<T, const N: usize> Drop for RawVec<T, N> {
    fn drop(&mut self) {
        if self.cap != 0 {
            let layout = Layout::array::<T>(self.cap).unwrap();
            unsafe {
                alloc::dealloc(self.ptr.as_ptr() as *mut u8, layout);
            }
        }
    }
}

// A vector that is limited in maximum size. Usefull if you know the size of the vector is bounded.
pub struct MVec<T, const N: usize> {
    buffer: RawVec<T, N>,
    len: usize,
}

impl<T, const N: usize> MVec<T, N> {
    pub fn new() -> Self {
        Self {
            buffer: RawVec::new(),
            len: 0,
        }
    }

    pub fn len(&self) -> usize {
        self.len
    }

    fn ptr(&self) -> *mut T {
        self.buffer.ptr.as_ptr()
    }

    pub fn capacity(&self) -> usize {
        self.buffer.cap
    }

    pub fn max_cap() -> usize {
        N
    }

    fn extend(&mut self, count: usize) {
        self.buffer.extend(count)
    }

    pub fn push(&mut self, elem: T) {
        if self.len == self.capacity() {
            self.buffer.grow();
        }

        unsafe {
            ptr::write(self.ptr().add(self.len), elem);
        }

        // Can't fail, we'll OOM first.
        self.len += 1;
    }

    pub fn pop(&mut self) -> Option<T> {
        if self.len == 0 {
            None
        } else {
            self.len -= 1;
            unsafe { Some(ptr::read(self.ptr().add(self.len))) }
        }
    }

    pub fn insert(&mut self, idx: usize, elem: T) {
        assert!(
            idx < N,
            "Insert index exeeds the size of the MVec: {} < {}",
            idx,
            N
        );
        if idx > self.capacity() {
            self.extend(self.capacity() - idx - 1);
        }
        if idx > self.len {
            self.len = idx + 1;
        }
        unsafe { ptr::write(self.ptr().add(idx), elem) }
    }

    pub fn get(&self, idx: usize) -> &T {
        unsafe { &ptr::read(self.ptr().add(idx)) }
    }
    pub fn get_mut(&self, idx: usize) -> &mut T {
        unsafe { &mut ptr::read(self.ptr().add(idx)) }
    }
}

unsafe impl<T: Send, const N: usize> Send for MVec<T, N> {}
unsafe impl<T: Sync, const N: usize> Sync for MVec<T, N> {}

impl<T, const N: usize> ops::Deref for MVec<T, N> {
    type Target = [T];

    fn deref(&self) -> &Self::Target {
        unsafe { slice::from_raw_parts(self.ptr(), self.len) }
    }
}


impl<T, const N: usize> ops::DerefMut for MVec<T, N> {
    fn deref_mut(&mut self) -> &mut [T] {
        unsafe { slice::from_raw_parts_mut(self.ptr(), self.len) }
    }
}
