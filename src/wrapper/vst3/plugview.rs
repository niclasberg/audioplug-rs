use raw_window_handle::{RawWindowHandle, Win32WindowHandle};
use vst3_sys::{VST3, VstPtr};
use vst3_sys::vst::IComponentHandler;
use vst3_sys::base::*;
use vst3_sys::gui::{IPlugView, ViewRect};
use std::cell::RefCell;
use std::ffi::{CStr, c_void};
use std::rc::Rc;

use crate::app::AppState;
use crate::core::{Color, Rectangle, Size, Shape, Point};
use crate::view::Fill;
use crate::window::Window;

const VST3_PLATFORM_HWND: &str = "HWND";
const VST3_PLATFORM_NSVIEW: &str = "NSView";

use vst3_sys as vst3_com;
#[VST3(implements(IPlugView))]
pub struct PlugView {
    window: RefCell<Option<Window>>,
    component_handler: Option<VstPtr<dyn IComponentHandler>>,
    app_state: Rc<RefCell<AppState>>,
}

impl PlugView {
    pub fn new(component_handler: Option<VstPtr<dyn IComponentHandler>>, app_state: Rc<RefCell<AppState>>) -> Box<Self> {
        Self::allocate(RefCell::new(None), component_handler, app_state)
    }

    pub fn create_instance(component_handler: Option<VstPtr<dyn IComponentHandler>>, app_state: Rc<RefCell<AppState>>) -> *mut c_void {
        Box::into_raw(Self::new(component_handler, app_state)) as *mut c_void
    }
}

impl IPlugView for PlugView {
    unsafe fn is_platform_type_supported(&self, type_: FIDString) -> tresult {
        let type_ = CStr::from_ptr(type_);
        match type_.to_str() {
            #[cfg(target_os = "windows")]
            Ok(type_) if type_ == VST3_PLATFORM_HWND => kResultOk,
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
                    let mut h = Win32WindowHandle::empty();
                    h.hwnd = parent;
                    RawWindowHandle::Win32(h)
                }, 
				#[cfg(target_os = "macos")]
				Ok(type_) if type_ == VST3_PLATFORM_NSVIEW => {
					let h = Win32WindowHandle::empty();
					RawWindowHandle::Win32(h)
				},
                _ => {
                    return kInvalidArgument;
                }
            };

            *window = Some(Window::attach(self.app_state.clone(), handle, |_| Rectangle::new(Point::ZERO, Size::new(20.0, 20.0)).fill(Color::BLACK)));

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

    unsafe fn get_size(&self, _size: *mut ViewRect) -> tresult {
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
        kResultOk
    }

    unsafe fn can_resize(&self) -> tresult {
        kResultTrue
    }

    unsafe fn check_size_constraint(&self, _rect: *mut ViewRect) -> tresult {
        kResultOk
    }
}