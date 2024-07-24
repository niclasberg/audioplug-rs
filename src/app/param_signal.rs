use std::marker::PhantomData;

pub struct ParamSignal<T> {
	_phantom: PhantomData<T>
} 

impl<T> ParamSignal<T> {
	pub fn begin_edit() {

	}

	pub fn end_edit() {

	}
}

