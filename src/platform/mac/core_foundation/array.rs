use std::os::raw::c_void;

use crate::platform::{IMut, IRef};

use super::{CFTyped, CFTypeID, CFIndex, CFAllocator, CFString, Boolean, kCFAllocatorDefault, CFRange, CFType};

pub type CFArrayRetainCallBack = extern "C" fn(allocator: *const CFAllocator, value: *const c_void) -> *const c_void;
pub type CFArrayReleaseCallBack = extern "C" fn(allocator: *const CFAllocator, value: *const c_void);
pub type CFArrayCopyDescriptionCallBack = extern "C" fn(value: *const c_void) -> *const CFString;
pub type CFArrayEqualCallBack = extern "C" fn(value1: *const c_void, value2: *const c_void) -> Boolean;
pub type CFArrayApplierFunction = extern "C" fn(value: *const c_void, context: *mut c_void);

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct CFArrayCallBacks {
    pub version: CFIndex,
    pub retain: CFArrayRetainCallBack,
    pub release: CFArrayReleaseCallBack,
    pub copy_description: CFArrayCopyDescriptionCallBack,
    pub equal: CFArrayEqualCallBack,
}


#[repr(C)]
pub struct CFArray {
	_data: [u8; 0],
    _marker: core::marker::PhantomData<(*mut u8, core::marker::PhantomPinned)>,
}

unsafe impl CFTyped for CFArray {
    fn type_id() -> super::CFTypeID {
        unsafe { CFArrayGetTypeID() }
    }
}

pub struct CFArrayIterator<'a> {
	idx: CFIndex,
	length: CFIndex,
	array: &'a CFArray
}

impl<'a> Iterator for CFArrayIterator<'a> {
    type Item = IRef<CFType>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.idx == self.length {
			None
		} else {
			unsafe {
				let result = IRef::wrap_and_retain(CFArrayGetValueAtIndex(self.array, self.idx) as *const CFType);
				self.idx += 1;
				Some(result)
			}
		}
    }
}

#[allow(dead_code)]
impl CFArray {
	pub fn new() -> IMut<CFArray> {
		unsafe {
			IMut::wrap(CFArrayCreateMutable(kCFAllocatorDefault, 0, &kCFTypeArrayCallBacks))
		}
	}

	pub fn get_count(&self) -> CFIndex {
		unsafe {
			CFArrayGetCount(self)
		}
	}

	pub fn contains_value(&self, range: CFRange, value: &impl CFTyped) -> bool {
		unsafe {
			CFArrayContainsValue(self, range, value.as_void_ptr()) != 0
		}
	}

	pub fn push(&mut self, value: &impl CFTyped) {
		unsafe {
			CFArrayAppendValue(self, value.as_void_ptr())
		}
	}

	pub fn iter(&self) -> CFArrayIterator {
		CFArrayIterator { idx: 0, length: self.get_count(), array: self }
	}

	pub unsafe fn as_vec_of<T: CFTyped>(&self) -> Vec<IRef<T>> {
		let count = self.get_count() as usize;
		let mut result = Vec::with_capacity(count);
		for i in 0..count {
			let value = unsafe { IRef::wrap_and_retain(CFArrayGetValueAtIndex(self, i as CFIndex) as *const T) };
			result.push(value);
		}
		result
	}

	pub fn get_value_at_index(&self, idx: CFIndex) -> Option<IRef<CFType>> {
		if idx < self.get_count() {
			let value = unsafe { IRef::wrap_and_retain(CFArrayGetValueAtIndex(self, idx) as *const CFType) };
			Some(value)
		} else {
			None
		}
	}

}

#[link(name = "CoreFoundation", kind = "framework")]
extern "C" {
	pub static kCFTypeArrayCallBacks: CFArrayCallBacks;

	fn CFArrayGetTypeID() -> CFTypeID;
	fn CFArrayGetCount(theArray: *const CFArray) -> CFIndex;

	fn CFArrayCreate(allocator: *const CFAllocator, values: *const *const c_void, numValues: CFIndex, callBacks: *const CFArrayCallBacks) -> *const CFArray;
	fn CFArrayCreateMutable(allocator: *const CFAllocator, capacity: CFIndex, callBacks: *const CFArrayCallBacks) -> *mut CFArray;
	fn CFArrayContainsValue(theArray: *const CFArray, range: CFRange, value: *const c_void) -> Boolean;
	fn CFArrayAppendValue(theArray: *mut CFArray, value: *const c_void);
	fn CFArrayGetValueAtIndex(theArray: *const CFArray, idx: CFIndex) -> *const c_void;
	fn CFArrayGetValues(theArray: *const CFArray, range: CFRange, values: *const *mut c_void);


}