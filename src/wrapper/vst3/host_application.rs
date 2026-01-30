use std::ffi::CStr;

use vst3::{
    ComPtr, ComRef, Interface,
    Steinberg::{
        FUnknown, Linux::IRunLoop, Vst::{IHostApplication, IHostApplicationTrait, IMessage, IMessageTrait}, kResultOk
    },
};

use crate::wrapper::vst3::util::tuid_from_uuid;

pub struct HostApplication {
    inner: ComPtr<IHostApplication>,
}

impl HostApplication {
    pub unsafe fn from_raw(ptr: *mut FUnknown) -> Option<Self> {
        let inner = unsafe { ComRef::from_raw(ptr) }
            .and_then(|cx| cx.cast::<IHostApplication>())?;

        Some(Self {
            inner
        })
    }

    pub unsafe fn allocate_message(&self, id: &CStr) -> Option<ComPtr<IMessage>> {
        let mut message_tuid = tuid_from_uuid(&IMessage::IID);
        let mut message = std::ptr::null_mut();
        let result = unsafe {
            self.inner
                .createInstance(&mut message_tuid, &mut message_tuid, &mut message)
        };
        if let Some(message) = unsafe { ComPtr::from_raw(message as *mut IMessage) }
            && result == kResultOk
        {
            unsafe { message.setMessageID(id.as_ptr()) };
            Some(message)
        } else {
            None
        }
    }

    pub unsafe fn get_runloop(&self) -> Option<ComPtr<IRunLoop>> {
        self.inner.cast()
    }
}
