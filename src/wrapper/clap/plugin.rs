use std::{
    ffi::{c_char, c_void},
    ptr::NonNull,
};

use clap_sys::{
    plugin::{clap_plugin, clap_plugin_descriptor},
    process::{clap_process, clap_process_status},
};

use crate::{ClapPlugin, Plugin};

#[repr(C)]
pub struct PluginInstance<P: ClapPlugin> {
    raw: clap_plugin,
    plugin: P,
}

impl<P: ClapPlugin> PluginInstance<P> {
    pub unsafe fn new_raw(description: &'static clap_plugin_descriptor) -> *mut Self {
        let plugin_vtbl = clap_plugin {
            desc: description,
            plugin_data: std::ptr::null_mut(),
            init: Some(init::<P>),
            destroy: Some(destroy::<P>),
            activate: todo!(),
            deactivate: todo!(),
            start_processing: todo!(),
            stop_processing: todo!(),
            reset: todo!(),
            process: todo!(),
            get_extension: todo!(),
            on_main_thread: todo!(),
        };
        let plugin = Plugin::new();
        let mut this = Box::new(Self {
            raw: plugin_vtbl,
            plugin,
        });

        this.raw.plugin_data = this.as_mut() as *mut _ as *mut _;

        Box::into_raw(this)
    }

    pub fn init(&self) -> bool {}
}

unsafe extern "C" fn init<P: ClapPlugin>(plugin: *const clap_plugin) -> bool {
    todo!()
}

unsafe extern "C" fn destroy<P: ClapPlugin>(plugin: *const clap_plugin) {
    let plugin = unsafe { (*plugin).plugin_data as *mut _ as *mut PluginInstance<P> };
}

unsafe extern "C" fn activate(
    plugin: *const clap_plugin,
    sample_rate: f64,
    min_frames_count: u32,
    max_frames_count: u32,
) -> bool {
    todo!()
}

unsafe extern "C" fn deactivate(plugin: *const clap_plugin) {
    todo!()
}

unsafe extern "C" fn start_processing(plugin: *const clap_plugin) -> bool {
    todo!()
}

unsafe extern "C" fn stop_processing(plugin: *const clap_plugin) {
    todo!()
}

unsafe extern "C" fn reset(plugin: *const clap_plugin) {
    todo!()
}

unsafe extern "C" fn process(
    plugin: *const clap_plugin,
    process: *const clap_process,
) -> clap_process_status {
    todo!()
}

unsafe extern "C" fn get_extension(plugin: *const clap_plugin, id: *const c_char) -> *const c_void {
    todo!()
}

unsafe extern "C" fn on_main_thread(plugin: *const clap_plugin) {}
