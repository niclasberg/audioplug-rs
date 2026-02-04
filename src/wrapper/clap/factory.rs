use std::{ffi::c_char, marker::PhantomData};

use clap_sys::{
    factory::plugin_factory::clap_plugin_factory,
    host::clap_host,
    plugin::{clap_plugin, clap_plugin_descriptor},
};

use crate::{
    ClapPlugin,
    wrapper::clap::{host::ClapHost, plugin::PluginInstance},
};

#[repr(C)]
pub struct Factory<P: ClapPlugin> {
    raw: clap_plugin_factory,
    //description: &'static clap_plugin_descriptor,
    _phantom: PhantomData<P>,
}

impl<P: ClapPlugin> Factory<P> {
    pub fn new() -> Self {
        Self {
            raw: clap_plugin_factory {
                get_plugin_count: Some(Self::get_plugin_count),
                get_plugin_descriptor: Some(Self::get_plugin_descriptor),
                create_plugin: Some(Self::create_plugin),
            },
            _phantom: PhantomData,
        }
    }

    pub fn as_raw(&self) -> *const clap_plugin_factory {
        &raw const self.raw
    }

    unsafe extern "C" fn get_plugin_count(_fac: *const clap_plugin_factory) -> u32 {
        1
    }

    unsafe extern "C" fn get_plugin_descriptor(
        _fac: *const clap_plugin_factory,
        _id: u32,
    ) -> *const clap_plugin_descriptor {
        todo!()
    }

    unsafe extern "C" fn create_plugin(
        _fac: *const clap_plugin_factory,
        host: *const clap_host,
        _id: *const c_char,
    ) -> *const clap_plugin {
        let Some(host) = (unsafe { ClapHost::from_ptr(host) }) else {
            return std::ptr::null();
        };
        todo!()
        //PluginInstance::<P>::new_raw(&self.description, host)
    }
}
