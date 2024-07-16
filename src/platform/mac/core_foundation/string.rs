use objc2_foundation::NSString;
use c_enum::c_enum;
use objc2::runtime::Bool;
use crate::platform::IRef;

use super::{CFAllocator, CFIndex, kCFAllocatorDefault, CFTyped, CFTypeID};


c_enum! {
	#[derive(Copy, Clone, PartialEq, Eq, Hash)]
	pub enum CFStringEncoding: u32 {
		UTF8 = 0x08000100
	}
}

#[repr(C)]
pub struct CFString {
	_data: [u8; 0],
    _marker: core::marker::PhantomData<(*mut u8, core::marker::PhantomPinned)>,
}

unsafe impl CFTyped for CFString {
    fn type_id() -> CFTypeID {
        unsafe { CFStringGetTypeID() }
    }
}

impl CFString {
	pub fn new(str: &str) -> IRef<Self> {
		unsafe {
			IRef::wrap(CFStringCreateWithBytes(
				kCFAllocatorDefault, 
				str.as_ptr(), 
				str.len() as CFIndex,
				CFStringEncoding::UTF8.0, 
				Bool::NO))
		}
	}
}

impl AsRef<NSString> for CFString {
    fn as_ref(&self) -> &NSString {
        unsafe { &*((self as *const _) as *const NSString) }
    }
}

impl AsMut<NSString> for CFString {
    fn as_mut(&mut self) -> &mut NSString {
        unsafe { &mut *((self as *mut _) as *mut NSString) }
    }
}

#[link(name = "CoreFoundation", kind = "framework")]
extern "C" {
	fn CFStringCreateWithBytes(alloc: *const CFAllocator, bytes: *const u8, numBytes: CFIndex, encoding: u32, isExternalRepresentation: Bool) -> *const CFString;
	fn CFStringGetTypeID() -> CFTypeID;
}