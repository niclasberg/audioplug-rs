#[repr(C)]
pub struct CFAllocator {
	_data: [u8; 0],
    _marker: core::marker::PhantomData<(*mut u8, core::marker::PhantomPinned)>,
}

extern "C" {
	pub static kCFAllocatorDefault: *const CFAllocator;
}