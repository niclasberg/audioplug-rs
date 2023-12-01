use std::{ops::{Deref, DerefMut}, os::raw::c_void};

pub unsafe trait IRefCounted {
	unsafe fn release(this: *const Self);
	unsafe fn retain(this: *const Self);
}

// Marker traits for CFTypes
pub unsafe trait CFType {
	
}

unsafe impl<T: CFType> IRefCounted for T {
    unsafe fn release(this: *const Self) {
        unsafe {
			CFRelease(this as *const c_void)
		}
    }

    unsafe fn retain(this: *const Self) {
        unsafe {
			CFRetain(this as *mut c_void);
		}
    }
}

pub struct IRef<T: IRefCounted> {
	ptr: *const T
}

impl<T: IRefCounted> IRef<T> {
	pub unsafe fn wrap(ptr: *const T) -> IRef<T> {
		IRef { ptr }
	}

	pub unsafe fn wrap_and_retain(ptr: *const T) -> IRef<T> {
		T::retain(ptr);
		IRef { ptr }
	}

	pub fn as_ptr(&self) -> *const T {
		self.ptr
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

pub struct IMut<T: IRefCounted> {
	ptr: *mut T
}

impl<T: IRefCounted> IMut<T> {
	pub unsafe fn wrap(ptr: *mut T) -> IMut<T> {
		IMut { ptr }
	}
}

impl<T: IRefCounted> Drop for IMut<T> {
    fn drop(&mut self) {
		unsafe { <T as IRefCounted>::release(self.ptr) };
    }
}

impl<T:IRefCounted> Deref for IMut<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        unsafe { &*self.ptr }
    }
}

impl<T:IRefCounted> DerefMut for IMut<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { &mut *self.ptr }
    }
}

impl<T: IRefCounted> From<IMut<T>> for IRef<T> {
    fn from(value: IMut<T>) -> Self {
        Self { ptr: value.ptr }
    }
}


#[link(name = "CoreFoundation", kind = "framework")]
extern "C" {
	fn CFRelease(cf: *const c_void);
	fn CFRetain(cf: *const c_void) -> *const c_void;
}