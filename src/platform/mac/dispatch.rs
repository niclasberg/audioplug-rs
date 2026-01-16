use std::ffi::c_void;

use block2::RcBlock;
use objc2::Encode;
use objc2_foundation::MainThreadMarker;

type DispatchFunction = unsafe extern "C" fn(*mut c_void);

#[repr(C)]
pub struct DispatchQueue {
    _data: [u8; 0],
}

unsafe extern "C" fn trampoline_once<F: FnOnce() + 'static>(context: *mut c_void) {
    let f = unsafe { Box::from_raw(context as *mut F) };
    (*f)();
}

fn trampoline_and_context<F: FnOnce() + 'static>(f: F) -> (DispatchFunction, *mut c_void) {
    let f = Box::new(f);
    (trampoline_once::<F>, Box::into_raw(f) as *mut _)
}

pub fn create_block_dispatching_to_main2<'f, A, B, F>(
    _mtm: MainThreadMarker,
    f: F,
) -> RcBlock<dyn Fn(A, B)>
where
    A: Encode + Copy + Send + 'static,
    B: Encode + Copy + Send + 'static,
    F: Fn(A, B) + Clone + 'static,
{
    let queue = dispatch_get_main_queue();
    RcBlock::new(move |a: A, b: B| {
        let _f = f.clone();
        let f = Box::new(move || _f(a, b));
        let queue = queue.clone();
        let (trampoline, ctx) = trampoline_and_context(f);
        unsafe { dispatch_async_f(queue, ctx, trampoline) }
    })
}

#[link(name = "System", kind = "dylib")]
unsafe extern "C" {
    static _dispatch_main_q: DispatchQueue;
    unsafe fn dispatch_async_f(
        queue: *const DispatchQueue,
        context: *mut c_void,
        work: DispatchFunction,
    );
}

pub fn dispatch_get_main_queue() -> *mut DispatchQueue {
    unsafe { &_dispatch_main_q as *const _ as *mut DispatchQueue }
}
