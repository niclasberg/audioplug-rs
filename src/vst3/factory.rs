use std::ffi::c_void;
use std::marker::PhantomData;

use vst3_sys::{VST3, IID};
use vst3_sys::base::{IPluginFactory, PFactoryInfo, PClassInfo, tresult, FactoryFlags, kResultOk, kInvalidArgument, ClassCardinality, kResultFalse};
use vst3_sys as vst3_com;
use crate::Plugin;
use crate::vst3::editcontroller::EditController;

use super::Vst3Plugin;
use super::util::strcpy;

#[VST3(implements(IPluginFactory))]
pub struct Factory<P: Plugin> {
    _phantom: PhantomData<P>,
}

impl<P: Plugin> Factory<P> {
    pub fn new() -> Box<Self> {
        Self::allocate(PhantomData)
    }
}

impl<P: Plugin> IPluginFactory for Factory<P> {
    unsafe fn get_factory_info(&self, info: *mut PFactoryInfo) -> tresult {
        if info.is_null() {
            return kInvalidArgument;
        }

        let info = &mut *info;
        strcpy(P::URL, &mut info.url);
        strcpy(P::EMAIL, &mut info.email);
        strcpy(P::VENDOR, &mut info.vendor);
        info.flags = FactoryFlags::kComponentNonDiscardable as i32;
        kResultOk
    }

    unsafe fn count_classes(&self) -> i32 {
        2
    }

    unsafe fn get_class_info(&self, index: i32, info: *mut PClassInfo) -> tresult {
        if info.is_null() {
            return kInvalidArgument;
        }

        let info = &mut *info;
        match index {
            0 => {
                strcpy(P::NAME, &mut info.name);
                info.cid = Vst3Plugin::<P>::CID;
                info.cardinality = ClassCardinality::kManyInstances as i32;
                strcpy("Audio Module Class", &mut info.category);
                kResultOk
            },
            1 => {
                info.cid = EditController::CID;
                strcpy((P::NAME.to_owned() + " edit controller").as_str(), &mut info.name);
                strcpy("Component Controller Class", &mut info.category);
                info.cardinality = ClassCardinality::kManyInstances as i32;
                kResultOk
            },
            _ => kInvalidArgument
        }
    }

    unsafe fn create_instance(&self, cid: *const IID, _iid: *const IID, obj: *mut *mut c_void,) -> tresult {
        println!("Create instance");
        if cid.is_null() || obj.is_null() {
            return kInvalidArgument;
        }

        match *cid {
            Vst3Plugin::<P>::CID => {
                *obj = Vst3Plugin::<P>::create_instance();
                kResultOk
            },
            EditController::CID => {
                *obj = EditController::create_instance();
                kResultOk
            }
            _ => kResultFalse,
        }
    }
}