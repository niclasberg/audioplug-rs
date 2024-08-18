use raw_window_handle::RawWindowHandle;
use vst3_sys::{VST3, VstPtr};
use vst3_sys::vst::IComponentHandler;
use vst3_sys::base::*;
use vst3_sys::gui::{IPlugView, ViewRect};
use std::cell::RefCell;
use std::ffi::{CStr, c_void};
use std::marker::PhantomData;
use std::ptr::NonNull;
use std::rc::Rc;

use crate::app::AppState;
use crate::core::{Color, Rectangle, Size, Point};
use crate::param::Params;
use crate::window::Window;
use crate::Editor;

#[cfg(target_os = "windows")]
use {
	raw_window_handle::Win32WindowHandle,
	std::num::NonZeroIsize
};

#[cfg(target_os = "macos")]
use raw_window_handle::AppKitWindowHandle;

#[cfg(target_os = "windows")]
const VST3_PLATFORM_HWND: &str = "HWND";
#[cfg(target_os = "macos")]
const VST3_PLATFORM_NSVIEW: &str = "NSView";

use vst3_sys as vst3_com;
#[VST3(implements(IPlugView))]
pub struct PlugView<P: Params, E: Editor<P>> {
    window: RefCell<Option<Window>>,
    app_state: Rc<RefCell<AppState>>,
	editor: Rc<RefCell<E>>,
    _phantom: PhantomData<P>
}

impl<P: Params, E: Editor<P>> PlugView<P, E> {
    pub fn new(app_state: Rc<RefCell<AppState>>, editor: Rc<RefCell<E>>) -> Box<Self> {
        Self::allocate(RefCell::new(None), app_state, editor, PhantomData)
    }

    pub fn create_instance(app_state: Rc<RefCell<AppState>>, editor: Rc<RefCell<E>>) -> *mut c_void {
        Box::into_raw(Self::new(app_state, editor)) as *mut c_void
    }
}

impl<P: Params, E: Editor<P>> IPlugView for PlugView<P, E> {
    unsafe fn is_platform_type_supported(&self, type_: FIDString) -> tresult {
        let type_ = CStr::from_ptr(type_);
        match type_.to_str() {
            #[cfg(target_os = "windows")]
            Ok(type_) if type_ == VST3_PLATFORM_HWND => kResultOk,
			#[cfg(target_os = "macos")]
            Ok(type_) if type_ == VST3_PLATFORM_NSVIEW => kResultOk,
            _ => kResultFalse,
        }
    }

    unsafe fn attached(&self, parent: *mut c_void, type_: FIDString) -> tresult {
        if parent.is_null() || type_.is_null() {
            return kResultFalse;
        }

        let mut window = self.window.borrow_mut();
        if window.is_none() {
            let type_ = unsafe { CStr::from_ptr(type_) };
            let handle = match type_.to_str() {
                #[cfg(target_os = "windows")]
                Ok(type_) if type_ == VST3_PLATFORM_HWND => {
                    let h = Win32WindowHandle::new(NonZeroIsize::new(parent as isize).unwrap());
                    RawWindowHandle::Win32(h)
                }, 
				#[cfg(target_os = "macos")]
				Ok(type_) if type_ == VST3_PLATFORM_NSVIEW => {
					let h = AppKitWindowHandle::new(NonNull::new(parent).unwrap());
					RawWindowHandle::AppKit(h)
				},
                _ => {
                    return kInvalidArgument;
                }
            };

			let _editor = self.editor.clone();
            *window = Some(Window::attach(self.app_state.clone(), handle, move |ctx| {
				let editor = RefCell::borrow(&_editor);
				let params = P::default();
				editor.view(ctx, &params)
			}));

            kResultOk
        } else {
            kResultFalse
        }
    }

    unsafe fn removed(&self) -> tresult {
        *self.window.borrow_mut() = None;
        kResultOk
    }

    unsafe fn on_wheel(&self, _distance: f32) -> tresult {
        // Handle in window class instead
        kResultOk
    }

    unsafe fn on_key_down(&self, _key: char16, _key_code: i16, _modifiers: i16) -> tresult {
        kResultOk
    }

    unsafe fn on_key_up(&self, _key: char16, _key_code: i16, _modifiers: i16) -> tresult {
        kResultOk
    }

    unsafe fn get_size(&self, size: *mut ViewRect) -> tresult {
		if size.is_null() {
            return kInvalidArgument;
        }
		if let Some(window) = self.window.borrow().as_ref() {
			let new_size = &mut *size;
			new_size.left = 0;
			new_size.right = 500;
			new_size.top = 0;
			new_size.bottom = 500;
		} else if let Some(pref_size) = self.editor.borrow().prefered_size() {
			let new_size = &mut *size;
			new_size.left = 0;
			new_size.right = pref_size.width as i32;
			new_size.top = 0;
			new_size.bottom = pref_size.height as i32;
		}

        kResultOk
    }

    unsafe fn on_size(&self, new_size: *mut ViewRect) -> tresult {
        if new_size.is_null() {
            return kInvalidArgument;
        }
        let new_size = &*new_size;

        if let Some(window) = self.window.borrow().as_ref() {
            let rect = Rectangle::from_ltrb(new_size.left, new_size.top, new_size.right, new_size.bottom);
            window.set_size(rect);
            kResultOk
        } else {
            kResultFalse
        }
    }

    unsafe fn on_focus(&self, _state: TBool) -> tresult {
        if let Some(_window) = self.window.borrow().as_ref() {
            kResultOk
        } else {
            kResultFalse
        }
    }

    unsafe fn set_frame(&self, _frame: *mut c_void) -> tresult {
		// The void pointer is a IPlugFrame which can be used to resize the window
        kResultOk
    }

    unsafe fn can_resize(&self) -> tresult {
        kResultTrue
    }

    unsafe fn check_size_constraint(&self, rect: *mut ViewRect) -> tresult {
		if rect.is_null() {
            return kInvalidArgument;
        }
        let rect = &mut *rect;


        kResultOk
    }
}