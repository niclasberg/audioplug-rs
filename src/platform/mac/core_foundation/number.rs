use std::os::raw::c_void;

use crate::platform::{IRef, CFType};

use super::{CFIndex, CFAllocator, kCFAllocatorDefault};
use c_enum::c_enum;

c_enum! {
	#[derive(Copy, Clone, PartialEq, Eq, Hash)]
	pub enum CFNumberType: CFIndex {
		SINT8 = 1,
		SINT16 = 2,
		SINT32 = 3,
		SINT64 = 4,
		FLOAT32 = 5,
		FLOAT64 = 6,
		CHAR = 7,
		SHORT = 8,
		INT = 9,
		LONG = 10,
		LONGLONG = 11,
		FLOAT = 12,
		DOUBLE = 13,
		CFIndex = 14,
		NSIntegerType = 15,
		CGFloatType = 16,
		MaxType = 16
	}
} 

#[repr(C)]
pub struct CFNumber {
	_data: [u8; 0],
    _marker: core::marker::PhantomData<(*mut u8, core::marker::PhantomPinned)>,
}

unsafe impl CFType for CFNumber {}

impl CFNumber {
	pub fn from_i8(value: i8) -> IRef<Self> {
		unsafe {
			IRef::wrap(CFNumberCreate(kCFAllocatorDefault, CFNumberType::SINT8, &value as *const _ as *const _))
		}
	}

	pub fn from_i16(value: i16) -> IRef<Self> {
		unsafe {
			IRef::wrap(CFNumberCreate(kCFAllocatorDefault, CFNumberType::SINT16, &value as *const _ as *const _))
		}
	}

	pub fn from_i32(value: i32) -> IRef<Self> {
		unsafe {
			IRef::wrap(CFNumberCreate(kCFAllocatorDefault, CFNumberType::SINT32, &value as *const _ as *const _))
		}
	}

	pub fn from_i64(value: i64) -> IRef<Self> {
		unsafe {
			IRef::wrap(CFNumberCreate(kCFAllocatorDefault, CFNumberType::SINT64, &value as *const _ as *const _))
		}
	}

	pub fn from_f32(value: f32) -> IRef<Self> {
		unsafe {
			IRef::wrap(CFNumberCreate(kCFAllocatorDefault, CFNumberType::FLOAT32, &value as *const _ as *const _))
		}
	}

	pub fn from_f64(value: f64) -> IRef<Self> {
		unsafe {
			IRef::wrap(CFNumberCreate(kCFAllocatorDefault, CFNumberType::FLOAT64, &value as *const _ as *const _))
		}
	}
}


#[link(name = "CoreFoundation", kind = "framework")]
extern "C" {
	fn CFNumberCreate(allocator: *const CFAllocator, theType: CFNumberType, valuePtr: *const c_void) -> *const CFNumber;
}