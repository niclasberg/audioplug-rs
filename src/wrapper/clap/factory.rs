use std::{ffi::c_char, marker::PhantomData};

use clap_sys::{
    factory::plugin_factory::clap_plugin_factory,
    host::clap_host,
    plugin::{clap_plugin, clap_plugin_descriptor},
    version::CLAP_VERSION,
};

use crate::{
    ClapPlugin,
    wrapper::clap::{ClapFeature, host::ClapHost, plugin::PluginInstance},
};

#[repr(C)]
pub struct Factory<P: ClapPlugin> {
    raw: clap_plugin_factory,
    description: clap_plugin_descriptor,
    _phantom: PhantomData<P>,
}

impl<P: ClapPlugin> Factory<P> {
    pub fn new() -> Self {
        let description = clap_plugin_descriptor {
            clap_version: CLAP_VERSION,
            id: todo!(),
            name: c"test".as_ptr(),
            vendor: c"test".as_ptr(),
            url: c"test.com".as_ptr(),
            manual_url: c"help.test.com".as_ptr(),
            support_url: c"help.test.com".as_ptr(),
            version: c"1.0.0".as_ptr(),
            description: c"Very cool".as_ptr(),
            features: &[ClapFeature::Instrument.as_cstr().as_ptr()] as *const _,
        };
        Self {
            raw: clap_plugin_factory {
                get_plugin_count: Some(Self::get_plugin_count),
                get_plugin_descriptor: Some(Self::get_plugin_descriptor),
                create_plugin: Some(Self::create_plugin),
            },
            description,
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
        fac: *const clap_plugin_factory,
        host: *const clap_host,
        _id: *const c_char,
    ) -> *const clap_plugin {
        let Some(host) = (unsafe { ClapHost::from_ptr(host) }) else {
            return std::ptr::null();
        };

        // Safety: The plugin_factory is the first member of the struct, and we have repr(C)
        // so members will never be shuffled. The `fac` pointer will thus point to the memory location
        // of Self - so we can just cast the pointer.
        let this = unsafe { &*(fac as *const Self) };
        PluginInstance::<P>::new_raw(&this.description, host)
    }
}
