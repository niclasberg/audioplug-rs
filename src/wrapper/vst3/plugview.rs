use raw_window_handle::RawWindowHandle;
use std::cell::RefCell;
use std::ffi::{c_void, CStr};
use std::rc::Rc;
use vst3_sys::base::*;
use vst3_sys::gui::IPlugFrame;
use vst3_sys::gui::IPlugViewContentScaleSupport;
use vst3_sys::gui::{IPlugView, ViewRect};
use vst3_sys::utils::SharedVstPtr;
use vst3_sys::VstPtr;
use vst3_sys::VST3;

use crate::ui::{AppState, Window};
use crate::core::Rectangle;
use crate::param::ParameterMap;
use crate::Editor;

#[cfg(target_os = "windows")]
use {raw_window_handle::Win32WindowHandle, std::num::NonZeroIsize};

#[cfg(target_os = "macos")]
use {raw_window_handle::AppKitWindowHandle, std::ptr::NonNull};

#[allow(unused)]
const VST3_PLATFORM_HWND: &str = "HWND";
#[allow(unused)]
const VST3_PLATFORM_NSVIEW: &str = "NSView";
#[allow(unused)]
const VST3_PLATFORM_X11_WINDOW: &str = "X11EmbedWindowID";

use vst3_sys as vst3_com;
#[VST3(implements(IPlugView, IPlugViewContentScaleSupport))]
pub struct PlugView<E: Editor> {
    window: RefCell<Option<Window>>,
    app_state: Rc<RefCell<AppState>>,
    editor: Rc<RefCell<E>>,
    plugin_frame: RefCell<Option<VstPtr<dyn IPlugFrame>>>,
    parameters: Rc<ParameterMap<E::Parameters>>,
}

impl<E: Editor> PlugView<E> {
    pub fn new(
        app_state: Rc<RefCell<AppState>>,
        editor: Rc<RefCell<E>>,
        parameters: Rc<ParameterMap<E::Parameters>>,
    ) -> Box<Self> {
        Self::allocate(
            RefCell::new(None),
            app_state,
            editor,
            RefCell::new(None),
            parameters,
        )
    }

    pub fn create_instance(
        app_state: Rc<RefCell<AppState>>,
        editor: Rc<RefCell<E>>,
        parameters: Rc<ParameterMap<E::Parameters>>,
    ) -> *mut c_void {
        Box::into_raw(Self::new(app_state, editor, parameters)) as *mut c_void
    }
}

impl<E: Editor> IPlugView for PlugView<E> {
    unsafe fn is_platform_type_supported(&self, type_: FIDString) -> tresult {
        let type_ = CStr::from_ptr(type_);
        match type_.to_str() {
            #[cfg(target_os = "windows")]
            Ok(type_) if type_ == VST3_PLATFORM_HWND => kResultOk,
            #[cfg(target_os = "macos")]
            Ok(type_) if type_ == VST3_PLATFORM_NSVIEW => kResultOk,
            #[cfg(target_os = "linux")]
            Ok(type_) if type_ == VST3_PLATFORM_X11_WINDOW => kResultOk,
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
                }
                #[cfg(target_os = "macos")]
                Ok(type_) if type_ == VST3_PLATFORM_NSVIEW => {
                    let h = AppKitWindowHandle::new(NonNull::new(parent).unwrap());
                    RawWindowHandle::AppKit(h)
                }
                _ => {
                    return kInvalidArgument;
                }
            };

            let view = {
                let editor = RefCell::borrow(&self.editor);
                editor.view(self.parameters.parameters_ref())
            };
            *window = Some(Window::attach(self.app_state.clone(), handle, view));

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
        if self.window.borrow().as_ref().is_some() {
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
            let rect =
                Rectangle::from_ltrb(new_size.left, new_size.top, new_size.right, new_size.bottom);
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

    unsafe fn set_frame(&self, frame: *mut c_void) -> tresult {
        let frame: SharedVstPtr<dyn IPlugFrame> = std::mem::transmute(frame);
        self.plugin_frame.replace(frame.upgrade());
        kResultOk
    }

    unsafe fn can_resize(&self) -> tresult {
        kResultTrue
    }

    unsafe fn check_size_constraint(&self, rect: *mut ViewRect) -> tresult {
        if rect.is_null() {
            return kInvalidArgument;
        }
        //let rect = &mut *rect;

        kResultOk
    }
}

impl<E: Editor> IPlugViewContentScaleSupport for PlugView<E> {
    unsafe fn set_scale_factor(&self, scale_factor: f32) -> tresult {
        if let Some(window) = self.window.borrow().as_ref() {
            window.set_scale_factor(scale_factor);
            kResultOk
        } else {
            kResultFalse
        }
    }
}
