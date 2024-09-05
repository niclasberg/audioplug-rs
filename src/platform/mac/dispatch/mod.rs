use std::{ffi::{c_long, c_ulong, c_void}, marker::PhantomData, time::Duration};

use block2::{Block, IntoBlock, RcBlock};
use c_enum::c_enum;
use objc2::{encode::{EncodeArguments, EncodeReturn}, Encode};
use objc2_foundation::MainThreadMarker;

use super::{IMut, IRef, IRefCounted};

type DispatchFunction = unsafe extern "C" fn(*mut c_void);
type DispatchBlock = Block<dyn Fn()>;

type DispatchTime = u64;
const DISPATCH_TIME_NOW: DispatchTime     = 0;
const DISPATCH_TIME_FOREVER: DispatchTime = !0;

c_enum!(
	#[repr(transparent)]
	pub enum QueuePriority: c_long {
		Low = -2,
		Default = 0,
		High = 2,
		Background = -1 << 15
	}
);

#[repr(C)]
pub struct DispatchQueue {
	_data: [u8; 0]
}

unsafe impl IRefCounted for DispatchQueue {
	unsafe fn release(this: *const Self) {
		dispatch_release(this);
	}

	unsafe fn retain(this: *const Self) {
		dispatch_retain(this);
	}
}

impl DispatchQueue {
	pub fn main() -> IRef<Self> {
		unsafe {
			let queue = dispatch_get_main_queue();
			IRef::wrap_and_retain(queue)
		}
	}

	pub fn global(priority: QueuePriority) -> IRef<Self> {
		unsafe {
			let queue = dispatch_get_global_queue(priority, 0);
			IRef::wrap_and_retain(queue)
		}
	}

	pub fn dispatch_async<F>(&self, f: F) 
	where 
		F: FnOnce() + Send + 'static
	{
		let f = Box::new(f);
		unsafe {
			dispatch_async_f(self, Box::into_raw(f) as *mut _, trampoline_once::<F>)
		}
	}

	pub fn dispatch_after<F>(&self, delay: Duration, f: F)
	where 
		F: FnOnce() + Send + 'static
	{
		todo!()
		// let f = Box::new(f);
	}
}

/// Main thread dispatch queue that can only be constructed and called from the main thread. It can therefore execute 
/// functions that does not implement the Send trait.
pub struct MainThreadQueue {
	queue: *mut DispatchQueue,
	// Ensure !Send
    _marker: core::marker::PhantomData<(*mut u8, core::marker::PhantomPinned)>,
}

impl MainThreadQueue {
	pub fn new(_mtm: MainThreadMarker) -> Self {
		let queue = unsafe { dispatch_get_main_queue() };
		Self {
			queue,
			_marker: PhantomData
		}
	}

	pub fn dispatch_async<F>(&self, f: F) 
	where 
		F: FnOnce() + 'static
	{
		let (trampoline, ctx) = trampoline_and_context(f);
		unsafe {
			dispatch_async_f(self.queue, ctx, trampoline)
		}
	}
}

unsafe extern "C" fn trampoline_once<F: FnOnce() + 'static>(context: *mut c_void) {
	let f = Box::from_raw(context as *mut F);
	(*f)();
}

fn trampoline_and_context<F: FnOnce() + 'static>(f: F) -> (DispatchFunction, *mut c_void) {
	let f = Box::new(f);
	(trampoline_once::<F>, Box::into_raw(f) as *mut _)
}

pub fn create_block_dispatching_to_main2<'f, A, B, F>(_mtm: MainThreadMarker, f: F) -> RcBlock<dyn Fn(A, B)>
where
	A: Encode + Copy + Send + 'static,
	B: Encode + Copy + Send + 'static,
	F: Fn(A, B) + Clone + 'static
{
	let queue = unsafe { dispatch_get_main_queue() };
	RcBlock::new(move |a: A, b: B| {	
		let _f = f.clone();	
		let f = Box::new(move || { _f(a, b) });
		let queue = queue.clone();
		let (trampoline, ctx) = trampoline_and_context(f);
		unsafe {
			dispatch_async_f(queue, ctx, trampoline)
		}
	})
}

#[link(name = "System", kind = "dylib")]
extern "C" {
	static _dispatch_main_q: DispatchQueue;
	fn dispatch_get_global_queue(identifier: QueuePriority, flags: c_ulong) -> *mut DispatchQueue;

	fn dispatch_release(object: *const DispatchQueue);
	fn dispatch_retain(object: *const DispatchQueue);

	fn dispatch_async(queue: *const DispatchQueue, block: *mut DispatchBlock);
	fn dispatch_async_f(queue: *const DispatchQueue, context: *mut c_void, work: DispatchFunction);
	fn dispatch_after_f(when: DispatchTime, queue: *const DispatchQueue, context: *mut c_void, work: DispatchFunction);
}

pub fn dispatch_get_main_queue() -> *mut DispatchQueue {
    unsafe { &_dispatch_main_q as *const _ as *mut DispatchQueue }
}