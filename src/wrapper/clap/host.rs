use std::ffi::{CStr, c_void};

use clap_sys::host::clap_host;

pub struct ClapHost {
    host: *const clap_host,
}

impl ClapHost {
    pub unsafe fn from_ptr(host: *const clap_host) -> Option<Self> {
        if host.is_null() {
            None
        } else {
            Some(Self { host })
        }
    }

    pub fn name(&self) -> &CStr {
        unsafe { CStr::from_ptr((*(self.host)).name) }
    }

    pub fn vendor(&self) -> &CStr {
        unsafe { CStr::from_ptr((*(self.host)).vendor) }
    }

    pub fn version(&self) -> &CStr {
        unsafe { CStr::from_ptr((*(self.host)).version) }
    }

    pub fn get_extension(&self, extension_id: &CStr) -> *const c_void {
        todo!()
        //unsafe { (*(self.host)).get_extension}
    }
}
