use std::ffi::{CStr, c_void};
use std::marker::PhantomData;

use vst3::ComWrapper;
use vst3::Steinberg::PClassInfo_::ClassCardinality_;
use vst3::Steinberg::PFactoryInfo_::FactoryFlags_;
use vst3::Steinberg::Vst::SDKVersionString;
use vst3::Steinberg::{
    FIDString, FUnknown, IPluginFactory, IPluginFactory2, IPluginFactory2Trait,
    IPluginFactoryTrait, PClassInfo, PClassInfo2, PFactoryInfo, TUID, kInvalidArgument,
    kResultFalse, kResultOk, tresult,
};

use super::editcontroller::EditController;
use crate::VST3Plugin;
use crate::wrapper::vst3::util::{strcpy_cstr, tuid_from_uuid};

use super::AudioProcessor;
use super::util::strcpy;

pub struct Factory<P: VST3Plugin> {
    _phantom: PhantomData<P>,
}

impl<P: VST3Plugin> vst3::Class for Factory<P> {
    type Interfaces = (IPluginFactory, IPluginFactory2);
}

impl<P: VST3Plugin> Factory<P> {
    pub fn new_raw() -> *mut IPluginFactory {
        ComWrapper::new(Self {
            _phantom: PhantomData,
        })
        .to_com_ptr()
        .unwrap()
        .into_raw()
    }

    const EDITOR_CID: TUID = tuid_from_uuid(&P::EDITOR_UUID);
    const PROCESSOR_CID: TUID = tuid_from_uuid(&P::PROCESSOR_UUID);
}

impl<P: VST3Plugin> IPluginFactoryTrait for Factory<P> {
    unsafe fn getFactoryInfo(&self, info: *mut PFactoryInfo) -> tresult {
        let Some(info) = (unsafe { info.as_mut() }) else {
            return kInvalidArgument;
        };
        strcpy(P::URL, &mut info.url);
        strcpy(P::EMAIL, &mut info.email);
        strcpy(P::VENDOR, &mut info.vendor);
        info.flags = FactoryFlags_::kComponentNonDiscardable as _;
        kResultOk
    }

    unsafe fn countClasses(&self) -> i32 {
        2
    }

    unsafe fn getClassInfo(&self, index: i32, info: *mut PClassInfo) -> tresult {
        let Some(info) = (unsafe { info.as_mut() }) else {
            return kInvalidArgument;
        };
        match index {
            0 => {
                strcpy(P::NAME, &mut info.name);
                info.cid = tuid_from_uuid(&P::PROCESSOR_UUID);
                info.cardinality = ClassCardinality_::kManyInstances as _;
                strcpy("Audio Module Class", &mut info.category);
                kResultOk
            }
            1 => {
                info.cid = tuid_from_uuid(&P::EDITOR_UUID);
                strcpy(
                    (P::NAME.to_owned() + " edit controller").as_str(),
                    &mut info.name,
                );
                strcpy("Component Controller Class", &mut info.category);
                info.cardinality = ClassCardinality_::kManyInstances as _;
                kResultOk
            }
            _ => kInvalidArgument,
        }
    }

    unsafe fn createInstance(
        &self,
        cid: FIDString,
        iid: FIDString,
        obj: *mut *mut c_void,
    ) -> tresult {
        if cid.is_null() {
            return kInvalidArgument;
        }
        let cid = unsafe { *(cid as *const TUID) };

        let instance = if cid == Self::PROCESSOR_CID {
            Some(
                ComWrapper::new(AudioProcessor::<P>::new())
                    .to_com_ptr::<FUnknown>()
                    .unwrap(),
            )
        } else if cid == Self::EDITOR_CID {
            Some(
                ComWrapper::new(EditController::<P::Editor>::new())
                    .to_com_ptr::<FUnknown>()
                    .unwrap(),
            )
        } else {
            None
        };

        if let Some(instance) = instance {
            let ptr = instance.as_ptr();
            // This will assign the instance to the obj out variable (and increment the ref count),
            // if it fullfills the interface with id iid
            unsafe { ((*(*ptr).vtbl).queryInterface)(ptr, iid as *mut TUID, obj) }
        } else {
            kInvalidArgument
        }
    }
}

const SDK_VERSION_STRING: &CStr = unsafe { CStr::from_ptr(SDKVersionString) };

impl<P: VST3Plugin> IPluginFactory2Trait for Factory<P> {
    unsafe fn getClassInfo2(&self, index: i32, info: *mut PClassInfo2) -> tresult {
        let Some(info) = (unsafe { info.as_mut() }) else {
            return kInvalidArgument;
        };

        match index {
            0 => {
                strcpy(P::NAME, &mut info.name);
                info.cid = tuid_from_uuid(&P::PROCESSOR_UUID);
                info.cardinality = ClassCardinality_::kManyInstances as _;
                strcpy("Audio Module Class", &mut info.category);
                strcpy_cstr(SDK_VERSION_STRING, info.sdkVersion.as_mut_slice());
                strcpy(&P::CATEGORIES.to_string(), &mut info.subCategories);
                kResultOk
            }
            1 => {
                info.cid = tuid_from_uuid(&P::EDITOR_UUID);
                strcpy(
                    (P::NAME.to_owned() + " edit controller").as_str(),
                    &mut info.name,
                );
                strcpy("Component Controller Class", &mut info.category);
                info.cardinality = ClassCardinality_::kManyInstances as _;
                strcpy_cstr(SDK_VERSION_STRING, &mut info.sdkVersion);
                strcpy(&P::CATEGORIES.to_string(), &mut info.subCategories);
                kResultOk
            }
            _ => kInvalidArgument,
        }
    }
}
