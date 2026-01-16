use raw_window_handle::RawWindowHandle;
use std::cell::RefCell;
use std::ffi::{CStr, c_void};
use std::rc::Rc;
#[cfg(target_os = "macos")]
use vst3::Steinberg::kResultOk;
use vst3::Steinberg::{
    FIDString, IPlugFrame, IPlugView, IPlugViewContentScaleSupport,
    IPlugViewContentScaleSupportTrait, IPlugViewTrait, TBool, ViewRect, char16, kInvalidArgument,
    kResultFalse, kResultTrue, tresult,
};
use vst3::{ComPtr, ComRef, ComWrapper};

use crate::Editor;
use crate::core::ScaleFactor;
use crate::param::ParameterMap;
use crate::ui::{AppState, Window};

#[cfg(target_os = "windows")]
use {raw_window_handle::Win32WindowHandle, std::num::NonZeroIsize};

#[cfg(target_os = "macos")]
use {raw_window_handle::AppKitWindowHandle, std::ptr::NonNull};

pub struct PlugView<E: Editor> {
    window: RefCell<Option<Window>>,
    app_state: Rc<RefCell<AppState>>,
    editor: Rc<RefCell<E>>,
    plugin_frame: RefCell<Option<ComPtr<IPlugFrame>>>,
    parameters: Rc<ParameterMap<E::Parameters>>,
}

impl<E: Editor> PlugView<E> {
    pub fn new(
        app_state: Rc<RefCell<AppState>>,
        editor: Rc<RefCell<E>>,
        parameters: Rc<ParameterMap<E::Parameters>>,
    ) -> ComWrapper<Self> {
        ComWrapper::new(Self {
            window: RefCell::new(None),
            app_state,
            editor,
            plugin_frame: RefCell::new(None),
            parameters,
        })
    }
}

impl<E: Editor> vst3::Class for PlugView<E> {
    type Interfaces = (IPlugView, IPlugViewContentScaleSupport);
}

#[cfg(target_os = "windows")]
const PLATFORM_TYPE_HWND: &'static CStr =
    unsafe { CStr::from_ptr(vst3::Steinberg::kPlatformTypeHWND) };
#[cfg(target_os = "macos")]
const PLATFORM_TYPE_NSVIEW: &'static CStr =
    unsafe { CStr::from_ptr(vst3::Steinberg::kPlatformTypeNSView) };

impl<E: Editor> IPlugViewTrait for PlugView<E> {
    unsafe fn isPlatformTypeSupported(&self, type_: FIDString) -> tresult {
        let type_ = unsafe { CStr::from_ptr(type_) };

        #[cfg(target_os = "windows")]
        if type_ == PLATFORM_TYPE_HWND {
            return kResultOk;
        }

        #[cfg(target_os = "macos")]
        if type_ == PLATFORM_TYPE_NSVIEW {
            return kResultOk;
        }

        kResultFalse
    }

    unsafe fn attached(&self, parent: *mut c_void, type_: FIDString) -> tresult {
        if parent.is_null() || type_.is_null() {
            return kResultFalse;
        }

        let mut window = self.window.borrow_mut();
        if window.is_none() {
            let type_ = unsafe { CStr::from_ptr(type_) };
            let handle = {
                #[cfg(target_os = "windows")]
                if type_ == PLATFORM_TYPE_HWND {
                    let h = Win32WindowHandle::new(NonZeroIsize::new(parent as isize).unwrap());
                    RawWindowHandle::Win32(h)
                }
                #[cfg(target_os = "macos")]
                if type_ == PLATFORM_TYPE_NSVIEW {
                    let h = AppKitWindowHandle::new(NonNull::new(parent).unwrap());
                    RawWindowHandle::AppKit(h)
                } else {
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

    unsafe fn setFrame(&self, frame: *mut IPlugFrame) -> tresult {
        let Some(frame) = (unsafe { ComRef::from_raw(frame) }) else {
            return kInvalidArgument;
        };
        self.plugin_frame.replace(Some(frame.to_com_ptr()));
        kResultOk
    }

    unsafe fn onWheel(&self, _distance: f32) -> tresult {
        // Handle in window class instead
        kResultOk
    }

    unsafe fn onKeyDown(&self, _key: char16, _key_code: i16, _modifiers: i16) -> tresult {
        kResultOk
    }

    unsafe fn onKeyUp(&self, _key: char16, _key_code: i16, _modifiers: i16) -> tresult {
        kResultOk
    }

    unsafe fn onFocus(&self, _state: TBool) -> tresult {
        kResultOk
    }

    // Size functions:
    // From the VST documentation:
    // The coordinates utilized within the ViewRect are native to the view system of the parent type. This implies that on
    // macOS (kPlatformTypeNSView), the coordinates are expressed in logical units (independent of the screen scale factor),
    // whereas on Windows (kPlatformTypeHWND) and Linux (kPlatformTypeX11EmbedWindowID), the coordinates are expressed in physical units (pixels).
    unsafe fn getSize(&self, size: *mut ViewRect) -> tresult {
        let Some(new_size) = (unsafe { size.as_mut() }) else {
            return kInvalidArgument;
        };

        if self.window.borrow().as_ref().is_some() {
            new_size.left = 0;
            new_size.right = 500;
            new_size.top = 0;
            new_size.bottom = 500;
        } else if let Some(pref_size) = self.editor.borrow().prefered_size() {
            new_size.left = 0;
            new_size.right = pref_size.width as i32;
            new_size.top = 0;
            new_size.bottom = pref_size.height as i32;
        }

        kResultOk
    }

    unsafe fn onSize(&self, new_size: *mut ViewRect) -> tresult {
        let Some(new_size) = (unsafe { new_size.as_ref() }) else {
            return kInvalidArgument;
        };

        if let Some(window) = self.window.borrow().as_ref() {
            #[cfg(target_os = "windows")]
            {
                use crate::core::{PhysicalCoord, PhysicalRect};
                window.set_physical_size(PhysicalRect {
                    left: PhysicalCoord(new_size.left),
                    top: PhysicalCoord(new_size.top),
                    right: PhysicalCoord(new_size.right),
                    bottom: PhysicalCoord(new_size.bottom),
                });
            }
            #[cfg(target_os = "macos")]
            {
                use crate::core::Rect;
                window.set_logical_size(Rect {
                    left: new_size.left as _,
                    top: new_size.top as _,
                    right: new_size.right as _,
                    bottom: new_size.bottom as _,
                });
            }

            kResultOk
        } else {
            kResultFalse
        }
    }

    unsafe fn canResize(&self) -> tresult {
        kResultTrue
    }

    unsafe fn checkSizeConstraint(&self, rect: *mut ViewRect) -> tresult {
        if rect.is_null() {
            return kInvalidArgument;
        }
        //let rect = &mut *rect;

        kResultOk
    }
}

impl<E: Editor> IPlugViewContentScaleSupportTrait for PlugView<E> {
    unsafe fn setContentScaleFactor(&self, scale_factor: f32) -> tresult {
        if let Some(window) = self.window.borrow().as_ref() {
            window.set_scale_factor(ScaleFactor(scale_factor as f64));
            kResultOk
        } else {
            kResultFalse
        }
    }
}
