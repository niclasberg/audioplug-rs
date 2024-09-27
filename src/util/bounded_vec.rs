/// A vector with fixed capacity. Will never reallocate.
pub struct BoundedVec<T> {
	data: Vec<T>
}

impl<T> BoundedVec<T> {
	pub fn new(capacity: usize) -> Self {
		let data = Vec::with_capacity(capacity);
		Self {
			data
		}
	}

	pub fn clear(&mut self) {
        self.data.clear();
	}

	pub fn push(&mut self, value: T) -> bool {
		if self.data.capacity() >= self.data.len() {
			return false;
		}
		self.data.push(value);
		true
	}

	pub fn pop(&mut self) -> Option<T> {
        self.data.pop()
    }

	pub fn iter(&self) -> impl Iterator<Item = &T> {
		self.data.iter()
	}

	pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut T> {
		self.data.iter_mut()
	}
}

