use std::ffi::c_void;
use std::marker::PhantomData;

use super::editcontroller::EditController;
use crate::VST3Plugin;
use vst3_sys as vst3_com;
use vst3_sys::base::{
    ClassCardinality, FactoryFlags, IPluginFactory, IPluginFactory2, PClassInfo, PClassInfo2,
    PFactoryInfo, kInvalidArgument, kResultFalse, kResultOk, tresult,
};
use vst3_sys::{IID, VST3};

use super::AudioProcessor;
use super::util::strcpy;

pub const VST3_SDK_VERSION: &str = "VST 3.6.14";

#[VST3(implements(IPluginFactory, IPluginFactory2))]
pub struct Factory<P: VST3Plugin> {
    _phantom: PhantomData<P>,
}

impl<P: VST3Plugin> Factory<P> {
    const EDITOR_CID: IID = IID {
        data: P::EDITOR_UUID,
    };
    const PROCESSOR_CID: IID = IID {
        data: P::PROCESSOR_UUID,
    };

    pub fn new() -> Box<Self> {
        Self::allocate(PhantomData)
    }
}

impl<P: VST3Plugin> IPluginFactory for Factory<P> {
    unsafe fn get_factory_info(&self, info: *mut PFactoryInfo) -> tresult {
        let Some(info) = (unsafe { info.as_mut() }) else {
            return kInvalidArgument;
        };
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
        let Some(info) = (unsafe { info.as_mut() }) else {
            return kInvalidArgument;
        };
        match index {
            0 => {
                strcpy(P::NAME, &mut info.name);
                info.cid = Self::PROCESSOR_CID;
                info.cardinality = ClassCardinality::kManyInstances as i32;
                strcpy("Audio Module Class", &mut info.category);
                kResultOk
            }
            1 => {
                info.cid = Self::EDITOR_CID;
                strcpy(
                    (P::NAME.to_owned() + " edit controller").as_str(),
                    &mut info.name,
                );
                strcpy("Component Controller Class", &mut info.category);
                info.cardinality = ClassCardinality::kManyInstances as i32;
                kResultOk
            }
            _ => kInvalidArgument,
        }
    }

    unsafe fn create_instance(
        &self,
        cid: *const IID,
        _iid: *const IID,
        obj: *mut *mut c_void,
    ) -> tresult {
        println!("Create instance");
        let Some(cid) = (unsafe { cid.as_ref() }) else {
            return kInvalidArgument;
        };
        let Some(obj) = (unsafe { obj.as_mut() }) else {
            return kInvalidArgument;
        };

        if *cid == Self::PROCESSOR_CID {
            *obj = AudioProcessor::<P>::create_instance();
            kResultOk
        } else if *cid == Self::EDITOR_CID {
            *obj = EditController::<P::Editor>::create_instance();
            kResultOk
        } else {
            kResultFalse
        }
    }
}

impl<P: VST3Plugin> IPluginFactory2 for Factory<P> {
    unsafe fn get_class_info2(&self, index: i32, info: *mut PClassInfo2) -> tresult {
        let Some(info) = (unsafe { info.as_mut() }) else {
            return kInvalidArgument;
        };

        match index {
            0 => {
                strcpy(P::NAME, &mut info.name);
                info.cid = Self::PROCESSOR_CID;
                info.cardinality = ClassCardinality::kManyInstances as i32;
                strcpy("Audio Module Class", &mut info.category);
                strcpy(VST3_SDK_VERSION, &mut info.sdk_version);
                strcpy(&P::CATEGORIES.to_string(), &mut info.subcategories);
                kResultOk
            }
            1 => {
                info.cid = Self::EDITOR_CID;
                strcpy(
                    (P::NAME.to_owned() + " edit controller").as_str(),
                    &mut info.name,
                );
                strcpy("Component Controller Class", &mut info.category);
                info.cardinality = ClassCardinality::kManyInstances as i32;
                strcpy(VST3_SDK_VERSION, &mut info.sdk_version);
                strcpy(&P::CATEGORIES.to_string(), &mut info.subcategories);
                kResultOk
            }
            _ => kInvalidArgument,
        }
    }
}
