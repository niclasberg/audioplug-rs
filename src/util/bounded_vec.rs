
use std::{mem::MaybeUninit, ops::Range};

pub struct BoundedVec<T> {
	data: Box<[MaybeUninit<T>]>,
	size: usize
}

impl<T> BoundedVec<T> {
	pub fn new(capacity: usize) -> Self {
		let data: Vec<_> = std::iter::repeat_with(MaybeUninit::uninit)
			.take(capacity)
			.collect();
		Self {
			data: data.into_boxed_slice(),
			size: 0
		}
	}

	pub fn clear(&mut self) {
        while self.size > 0 {
            self.size -= 1;
            unsafe { self.data.get_unchecked_mut(self.size).assume_init_drop() }
        }
	}

	pub fn push(&mut self, event: T) -> bool {
		if self.size >= self.data.len() {
			return false;
		}
		self.data[self.size] = MaybeUninit::new(event);
		self.size += 1;
		true
	}

	pub fn pop(&mut self) -> Option<T> {
        if self.size == 0 {
            return None;
        }

        // Safety: size > 0, and 
        let event = unsafe { self.data.get_unchecked_mut(self.size - 1).assume_init_read() };
        self.size -= 1;
        Some(event)
    }
}

impl<T> Drop for BoundedVec<T> {
    fn drop(&mut self) {

    }
}

pub struct Iter<'a, T: 'a> {
    current: *mut MaybeUninit<T>,
    end: *mut MaybeUninit<T>,
    _phantom: &'a T
}

