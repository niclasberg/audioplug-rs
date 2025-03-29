use std::ops::{Deref, DerefMut};

pub unsafe trait IRefCounted {
    unsafe fn release(this: *const Self);
    unsafe fn retain(this: *const Self);
}

pub struct IRef<T: IRefCounted> {
    ptr: *const T,
}

impl<T: IRefCounted> IRef<T> {
    pub unsafe fn wrap(ptr: *const T) -> IRef<T> {
        IRef { ptr }
    }

    pub unsafe fn wrap_if_non_null(ptr: *const T) -> Option<IRef<T>> {
        if ptr.is_null() {
            None
        } else {
            Some(Self::wrap(ptr))
        }
    }

    pub unsafe fn wrap_and_retain(ptr: *const T) -> IRef<T> {
        T::retain(ptr);
        IRef { ptr }
    }

    pub unsafe fn wrap_and_retain_if_non_null(ptr: *const T) -> Option<IRef<T>> {
        if ptr.is_null() {
            None
        } else {
            T::retain(ptr);
            Some(IRef { ptr })
        }
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

impl<T: IRefCounted> Deref for IRef<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        unsafe { &*self.ptr }
    }
}

impl<T: IRefCounted> Clone for IRef<T> {
    fn clone(&self) -> Self {
        unsafe { Self::wrap_and_retain(self.ptr) }
    }
}

pub struct IMut<T: IRefCounted> {
    ptr: *mut T,
}

impl<T: IRefCounted> IMut<T> {
    pub unsafe fn wrap(ptr: *mut T) -> Self {
        Self { ptr }
    }

    pub unsafe fn wrap_and_retain(ptr: *mut T) -> Self {
        T::retain(ptr);
        Self { ptr }
    }

    pub fn as_ptr(&self) -> *const T {
        self.ptr
    }

    pub fn as_mut_ptr(&self) -> *mut T {
        self.ptr
    }
}

impl<T: IRefCounted> Drop for IMut<T> {
    fn drop(&mut self) {
        unsafe { <T as IRefCounted>::release(self.ptr) };
    }
}

impl<T: IRefCounted> Deref for IMut<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        unsafe { &*self.ptr }
    }
}

impl<T: IRefCounted> DerefMut for IMut<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { &mut *self.ptr }
    }
}

impl<T: IRefCounted> Clone for IMut<T> {
    fn clone(&self) -> Self {
        unsafe { Self::wrap_and_retain(self.ptr) }
    }
}

impl<T: IRefCounted> From<IMut<T>> for IRef<T> {
    fn from(value: IMut<T>) -> Self {
        Self { ptr: value.ptr }
    }
}
