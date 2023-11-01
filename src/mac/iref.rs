use std::{ops::Deref, os::raw::c_void};

pub unsafe trait IRefCounted {
	unsafe fn release(this: *mut Self);
	unsafe fn retain(this: *mut Self);
}

// Marker traits for CFTypes
pub unsafe trait CFType {}

unsafe impl<T: CFType> IRefCounted for T {
    unsafe fn release(this: *mut Self) {
        unsafe {
			CFRelease(this as *mut c_void)
		}
    }

    unsafe fn retain(this: *mut Self) {
        unsafe {
			CFRetain(this as *mut c_void);
		}
    }
}

pub struct IRef<T: IRefCounted> {
	ptr: *mut T
}

impl<T: IRefCounted> IRef<T> {
	pub unsafe fn wrap(ptr: *mut T) -> IRef<T> {
		IRef { ptr }
	}
}

impl<T: IRefCounted> Drop for IRef<T> {
    fn drop(&mut self) {
		unsafe { <T as IRefCounted>::release(self.ptr) };
    }
}

impl<T:IRefCounted> Deref for IRef<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        unsafe { &*self.ptr }
    }
} 

#[link(name = "CoreFoundation", kind = "framework")]
extern "C" {
	fn CFRelease(cf: *mut c_void);
	fn CFRetain(cf: *mut c_void) -> *mut c_void;
}