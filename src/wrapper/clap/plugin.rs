use std::ffi::{c_char, c_void};

use atomic_refcell::AtomicRefCell;
use clap_sys::{
    plugin::{clap_plugin, clap_plugin_descriptor},
    process::{CLAP_PROCESS_CONTINUE, CLAP_PROCESS_ERROR, clap_process, clap_process_status},
};

use crate::{ClapPlugin, Plugin, wrapper::clap::host::ClapHost};

#[repr(C)]
pub struct PluginInstance<P: ClapPlugin> {
    // This needs to be the first member in order for casts from *mut clap_plugin to *mut Self to work!
    // This struct has C representation, so the members will not be reordered.
    raw: clap_plugin,
    host: ClapHost,
    plugin: AtomicRefCell<P>,
}

impl<P: ClapPlugin> PluginInstance<P> {
    pub fn new_raw(
        description: &'static clap_plugin_descriptor,
        host: ClapHost,
    ) -> *mut clap_plugin {
        let plugin_vtbl = clap_plugin {
            desc: description,
            // Will be set to &Self
            plugin_data: std::ptr::null_mut(),
            init: Some(Self::init),
            destroy: Some(Self::destroy),
            activate: Some(Self::clap_activate),
            deactivate: Some(Self::clap_deactivate),
            start_processing: Some(Self::clap_start_processing),
            stop_processing: Some(Self::clap_stop_processing),
            reset: Some(Self::clap_reset),
            process: Some(Self::clap_process),
            get_extension: Some(Self::clap_get_extension),
            on_main_thread: Some(Self::clap_on_main_thread),
        };
        let plugin = AtomicRefCell::new(Plugin::new(crate::HostInfo {
            name: host.name().to_str().unwrap().to_string(),
        }));
        let this = Box::new(Self {
            raw: plugin_vtbl,
            plugin,
            host,
        });
        let this_ptr = Box::into_raw(this);
        let clap_plugin = &mut unsafe { &mut *this_ptr }.raw;
        clap_plugin.plugin_data = this_ptr.cast();
        clap_plugin
    }

    unsafe extern "C" fn destroy(plugin: *const clap_plugin) {
        drop(unsafe { Box::from_raw((*plugin).plugin_data) })
    }

    unsafe fn use_self<'a>(plugin: *const clap_plugin) -> Option<&'a Self> {
        if plugin.is_null() || unsafe { (*plugin).plugin_data }.is_null() {
            return None;
        }

        Some(unsafe { &*((*plugin).plugin_data.cast::<Self>()) })
    }

    // clap_plugin methods
    unsafe extern "C" fn init(plugin: *const clap_plugin) -> bool {
        let Some(_this) = (unsafe { Self::use_self(plugin) }) else {
            return false;
        };

        true
    }

    unsafe extern "C" fn clap_activate(
        plugin: *const clap_plugin,
        sample_rate: f64,
        _min_frames_count: u32,
        max_frames_count: u32,
    ) -> bool {
        let Some(this) = (unsafe { Self::use_self(plugin) }) else {
            return false;
        };
        this.plugin
            .borrow_mut()
            .prepare(sample_rate, max_frames_count as _);
        true
    }

    unsafe extern "C" fn clap_deactivate(plugin: *const clap_plugin) {
        let Some(_this) = (unsafe { Self::use_self(plugin) }) else {
            return;
        };
    }

    unsafe extern "C" fn clap_start_processing(plugin: *const clap_plugin) -> bool {
        let Some(_this) = (unsafe { Self::use_self(plugin) }) else {
            return false;
        };
        true
    }

    unsafe extern "C" fn clap_stop_processing(plugin: *const clap_plugin) {
        let Some(_this) = (unsafe { Self::use_self(plugin) }) else {
            return;
        };
    }

    unsafe extern "C" fn clap_reset(plugin: *const clap_plugin) {
        let Some(_this) = (unsafe { Self::use_self(plugin) }) else {
            return;
        };
    }

    unsafe extern "C" fn clap_process(
        plugin: *const clap_plugin,
        process: *const clap_process,
    ) -> clap_process_status {
        let Some(_this) = (unsafe { Self::use_self(plugin) }) else {
            return CLAP_PROCESS_ERROR;
        };

        CLAP_PROCESS_CONTINUE
    }

    unsafe extern "C" fn clap_get_extension(
        plugin: *const clap_plugin,
        id: *const c_char,
    ) -> *const c_void {
        todo!()
    }

    unsafe extern "C" fn clap_on_main_thread(plugin: *const clap_plugin) {}
}
