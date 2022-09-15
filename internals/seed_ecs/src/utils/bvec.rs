use std::{ptr::NonNull, alloc::Layout};

use super::MVec;

// This is where all the magic happens. Each layer condense the information from the previous one.
// Each bit of the last layer represent the storage of something inside the vector. If the bit is 0
// then nothing is stored at its index.
// Each bit of the layers above represent 4 bits in the layer below. If one of the bits in the
// layer below is at one the it also is at 1 else it is at 0.
pub struct BMask {
    root: u32,
    l1: MVec<u32, 32>,
    l2: MVec<u32, {32*32}>, // 32^2
    l3: MVec<u32, {32*32*32}>, // 32^3
}

#[inline]
pub fn position(idx: usize, row_nb: usize) -> (usize, u32) {
    let index = idx >> 5*row_nb;
    let bit_nb = (idx >> 5*(row_nb-1)) % 32;
    (index, bit_nb as u32)
}

impl BMask {

    pub fn new() -> Self {
        Self {
            root: 0,
            l1: MVec::new(),
            l2: MVec::new(),
            l3: MVec::new(),
        }
    }

    fn add(&mut self, idx: usize) {
        let (l3_idx, l3_offset) = position(idx, 1);
        let (l2_idx, l2_offset) = position(idx , 2);
        let (l1_idx, l1_offset) = position(idx , 3);
        let (_, root_offset) = position(idx , 4);
        self.root |= 1 << root_offset;
        (*self.l1)[l1_idx] |= 1 << l1_offset;
        (*self.l2)[l2_idx] |= 1 << l2_offset;
        (*self.l3)[l3_idx] |= 1 << l3_offset;
    }

    fn first_empty_spot(&self) -> usize {
        let found = false;
        let mut win= self.root;
        let mut win_idx = 0;
        let mut tot_idx = 0;
        for i in 1..=4 {
            while self.root & 1 << win_idx != 1 << win_idx {
                win_idx += 1;
            }
            tot_idx = (tot_idx*32 + win_idx)*32;
            win = match i {
                1 => (*self.l1)[win_idx as usize * 32],
                2 => (*self.l2)[win_idx as usize * 32],
                3 => (*self.l3)[win_idx as usize * 32],
                _ => 0,
            }
        }
        return tot_idx;
    }

    fn is_present(&self, idx: usize) -> bool {
        let (l3_idx, l3_offset) = position(idx, 1);
        (*self.l3)[l3_idx] & 1<<l3_offset == 1<<l3_offset
     }

    fn remove(&mut self, idx: usize) {
        let (l3_idx, l3_offset) = position(idx, 1);
        if (*self.l3)[l3_idx] ^ 1<<l3_offset != 0 {return;}
        (*self.l3)[l3_idx] ^= 1<<l3_offset;
        if (*self.l3)[l3_idx] != 0 {return;}
        let (l2_idx, l2_offset) = position(idx, 2);
        (*self.l2)[l2_idx] ^= 1<<l2_offset;
        if (*self.l2)[l2_idx] != 0 {return;}
        let (l1_idx, l1_offset) = position(idx, 3);
        (*self.l1)[l1_idx] ^= 1<<l1_offset;
        if (*self.l1)[l1_idx] != 0 {return;}
        let (_, root_offset) = position(idx, 4);
        self.root ^= 1<<root_offset;
    }

    fn next(&self, idx: usize) -> usize {
        let found = false;
        let mut win= self.root;
        let mut win_idx = 0;
        let mut tot_idx = 0;
        for i in 1..=4 {
            while self.root & 1 << win_idx == 1 << win_idx {
                win_idx += 1;
            }
            tot_idx = (tot_idx*32 + win_idx)*32;
            win = match i {
                1 => (*self.l1)[win_idx as usize * 32],
                2 => (*self.l2)[win_idx as usize * 32],
                3 => (*self.l3)[win_idx as usize * 32],
                _ => 0,
            }
        }
        return tot_idx;
    }
}

// BitVector is a vector that allows fast iteration over sparse set of data.
pub struct BVec<T> {
    mask: BMask,
    buffer: MVec<T, {32*32*32}>,
}

impl<T> BVec<T> {
    pub fn new() -> Self {
        let layout = Layout::array::<T>(16*32).unwrap();
        Self {
            mask: BMask::new(),
            buffer: MVec::new(),
        }
    }

    pub fn get(&self, idx: usize) -> Option<&T>{
        if !self.mask.is_present(idx) {
            None
        } else {
            Some(self.buffer.get(idx))
        }
    }

    pub fn get_mut(&mut self, idx: usize) -> Option<&mut T> {
        if !self.mask.is_present(idx) {
            None
        } else {
            Some(self.buffer.get_mut(idx))
        }
    }

    pub fn insert_first_empty(&mut self, elem: T) -> &T {
        let idx = self.mask.first_empty_spot();
        self.mask.add(idx);
        self.buffer.insert(idx, elem);
        // It is safe to unwrap here as we just inserted the element at the index
        self.get(idx).unwrap()
    }

    fn next_item_index(&mut self, idx: usize) -> usize {
        self.mask.next(idx)
    }

    pub fn remove(&mut self, idx: usize) {
        self.mask.remove(idx);
    }
}

impl<T> IntoIterator for BVec<T> {
    type Item = T;

    type IntoIter = BVecIterator<T>;

    fn into_iter(self) -> Self::IntoIter {
        BVecIterator {
            inner: self,
            cursor: 0,
        }
    }
}

pub struct BVecIterator<T> {
    inner: BVec<T>, 
    cursor: usize, 
}

impl<T> Iterator for BVecIterator<T> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        let idx = self.inner.next_item_index(self.cursor);
        self.inner.get_mut(idx)
    }
}

