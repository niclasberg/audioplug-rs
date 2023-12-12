use std::os::raw::c_void;

use crate::platform::{IRefCounted, IRef, IMut};

use super::{CFTypeID, CFTypeRef};

// Marker traits for CFTypes
pub unsafe trait CFTyped: Sized{
	fn type_id() -> CFTypeID;

	fn as_void_ptr(&self) -> *const c_void {
		self as *const _ as *const c_void
	}

	fn from_iref(cf_type: IRef<CFType>) -> Option<IRef<Self>> {
		if cf_type.is_a::<Self>() {
			Some(unsafe { IRef::wrap(cf_type.as_ptr() as *const _) })
		} else {
			None
		}
	}

	fn from_imut(cf_type: IMut<CFType>) -> Option<IMut<Self>> {
		if cf_type.is_a::<Self>() {
			Some(unsafe { IMut::wrap(cf_type.as_mut_ptr() as *mut _) })
		} else {
			None
		}
	}
}

unsafe impl<T: CFTyped> IRefCounted for T {
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

#[repr(C)]
pub struct CFType {
	_data: [u8; 0],
    _marker: core::marker::PhantomData<(*mut u8, core::marker::PhantomPinned)>,
}

impl CFType {
	pub fn type_id(&self) -> CFTypeID {
		unsafe { CFGetTypeID(self as *const Self as *const _) }
	}

	pub fn is_a<T: CFTyped>(&self) -> bool {
		self.type_id() == T::type_id()
	}

	pub fn downcast<T: CFTyped>(&self) -> Option<&T> {
		if self.is_a::<T>() {
			Some(unsafe { &*(self as *const _ as *const _) })
		} else {
			None
		}
	}

	pub fn downcast_mut<T: CFTyped>(&mut self) -> Option<&mut T> {
		if self.is_a::<T>() {
			Some(unsafe { &mut *(self as *mut _ as *mut _) })
		} else {
			None
		}
	}

}

impl<T: CFTyped> From<IRef<T>> for IRef<CFType> {
    fn from(value: IRef<T>) -> Self {
        unsafe { Self::wrap(value.as_ptr() as *const _) }
    }
}

impl<T: CFTyped> From<IMut<T>> for IMut<CFType> {
    fn from(value: IMut<T>) -> Self {
        unsafe { Self::wrap(value.as_mut_ptr() as *mut _) }
    }
}

unsafe impl IRefCounted for CFType {
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

#[link(name = "CoreFoundation", kind = "framework")]
extern "C" {
	fn CFRelease(cf: *const c_void);
	fn CFRetain(cf: *const c_void) -> *const c_void;
	fn CFGetTypeID(cf: CFTypeRef) -> CFTypeID;
}