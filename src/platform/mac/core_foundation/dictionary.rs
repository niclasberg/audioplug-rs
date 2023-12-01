#[repr(C)]
pub struct CFDictionary {
	_data: [u8; 0],
    _marker: core::marker::PhantomData<(*mut u8, core::marker::PhantomPinned)>,
}

#[link(name = "CoreFoundation", kind = "framework")]
extern "C" {
	
}