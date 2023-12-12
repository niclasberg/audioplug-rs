use std::ptr::NonNull;

use crate::platform::{IMut, IRef};

use super::{CFString, CFTyped, CFDictionary, CFAllocator, kCFAllocatorDefault, CFIndex, CFRange, CFTypeRef, CFTypeID, CFType};


#[repr(C)]
pub struct CFAttributedString {
	_data: [u8; 0],
    _marker: core::marker::PhantomData<(*mut u8, core::marker::PhantomPinned)>,
}

unsafe impl CFTyped for CFAttributedString {
	fn type_id() -> CFTypeID {
		unsafe { CFAttributedStringGetTypeID() }
	}
}

#[allow(dead_code)]
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

	pub fn set_attribute<T: CFTyped>(&mut self, range: CFRange, attr_name: &CFString, value: &T) {
		unsafe {
			CFAttributedStringSetAttribute(self, range, attr_name, value as *const _ as *const _)
		}
	}

	pub fn get_attribute<T: CFTyped>(&self, loc: CFIndex, attr_name: &CFString, effective_range: Option<CFRange>) -> Option<IRef<T>> {
		unsafe {
			let effective_range = effective_range.map_or_else(|| std::ptr::null(), |x| &x as *const _);
			IRef::wrap_and_retain_if_non_null(CFAttributedStringGetAttribute(self, loc, attr_name, effective_range))
				.and_then(CFTyped::from_iref)
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
	fn CFAttributedStringGetAttribute(aStr: *const CFAttributedString, loc: CFIndex, attrName: *const CFString, effectiveRange: *const CFRange) -> *const CFType;
	fn CFAttributedStringGetTypeID() -> CFTypeID;
}