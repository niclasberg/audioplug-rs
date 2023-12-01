use crate::platform::{IMut, IRef, CFType};

use super::{CFString, CFDictionary, CFAllocator, kCFAllocatorDefault, CFIndex, CFRange, CFTypeRef};


#[repr(C)]
pub struct CFAttributedString {
	_data: [u8; 0],
    _marker: core::marker::PhantomData<(*mut u8, core::marker::PhantomPinned)>,
}

unsafe impl CFType for CFAttributedString {}

impl CFAttributedString {
	pub fn new(str: &CFString, attributes: &CFDictionary) -> IRef<Self> {
		unsafe {
			IRef::wrap(CFAttributedStringCreate(kCFAllocatorDefault, str, attributes))
		}
	}

	pub fn new_mut(max_length: CFIndex) -> IMut<Self> {
		unsafe {
			IMut::wrap(CFAttributedStringCreateMutable(kCFAllocatorDefault, max_length))
		}
	}

	pub fn create_mutable_copy(&self, max_length: CFIndex) -> IMut<Self> {
		unsafe {
			IMut::wrap(CFAttributedStringCreateMutableCopy(kCFAllocatorDefault, max_length, self))
		}
	}

	pub fn set_attribute<T: CFType>(&mut self, range: CFRange, attr_name: &CFString, value: &T) {
		unsafe {
			CFAttributedStringSetAttribute(self, range, attr_name, value as *const _ as *const _)
		}
	}

	pub fn replace_string(&mut self, range: CFRange, replacement: &CFString) {
		unsafe {
			CFAttributedStringReplaceString(self, range, replacement)
		}
	}

	pub fn length(&self) -> CFIndex {
		unsafe {
			CFAttributedStringGetLength(self)
		}
	}
}

#[link(name = "CoreFoundation", kind = "framework")]
extern "C" {
	fn CFAttributedStringCreate(alloc: *const CFAllocator, str: *const CFString, attributes: *const CFDictionary) -> *const CFAttributedString;
	fn CFAttributedStringCreateMutable(alloc: *const CFAllocator, maxLength: CFIndex) -> *mut CFAttributedString;
	fn CFAttributedStringCreateMutableCopy(alloc: *const CFAllocator, maxLength: CFIndex, aStr: *const CFAttributedString) -> *mut CFAttributedString;
	fn CFAttributedStringSetAttribute(aStr: *mut CFAttributedString, range: CFRange, attrName: *const CFString, value: CFTypeRef );
	fn CFAttributedStringReplaceString(aStr: *mut CFAttributedString, range: CFRange, replacement: *const CFString);
	fn CFAttributedStringGetLength(aStr: *const CFAttributedString) -> CFIndex;
}