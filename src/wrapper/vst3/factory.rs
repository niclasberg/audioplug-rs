use std::ffi::c_void;
use std::marker::PhantomData;
use std::sync::{Mutex, OnceLock};

use vst3::{ComPtr, ComWrapper};
use vst3::Steinberg::PClassInfo_::ClassCardinality_;
use vst3::Steinberg::PFactoryInfo_::FactoryFlags_;
use vst3::Steinberg::{
    FIDString, FUnknown, IPluginFactory, IPluginFactory2, IPluginFactory2Trait, IPluginFactory3, IPluginFactory3Trait, IPluginFactoryTrait, PClassInfo, PClassInfo2, PClassInfoW, PFactoryInfo, TUID, kInvalidArgument, kResultOk, tresult
};

use super::editcontroller::EditController;
use crate::VST3Plugin;
use crate::wrapper::vst3::VST3Categories;
use crate::wrapper::vst3::host_application::HostApplication;
use crate::wrapper::vst3::util::{strcpyw, tuid_from_uuid};

use super::AudioProcessor;
use super::util::strcpy;

pub struct Factory<P: VST3Plugin> {
    host_context: OnceLock<HostApplication>,
    _phantom: PhantomData<P>,
}

impl<P: VST3Plugin> vst3::Class for Factory<P> {
    type Interfaces = (IPluginFactory, IPluginFactory2, IPluginFactory3);
}

impl<P: VST3Plugin> Factory<P> {
    pub fn new_raw() -> *mut IPluginFactory {
        ComWrapper::new(Self {
            _phantom: PhantomData,
            host_context: OnceLock::new()
        })
        .to_com_ptr()
        .unwrap()
        .into_raw()
    }

    const EDITOR_CID: TUID = tuid_from_uuid(P::EDITOR_UUID.as_bytes());
    const PROCESSOR_CID: TUID = tuid_from_uuid(P::PROCESSOR_UUID.as_bytes());
}

const SDK_VERSION_STRING: &str = "VST 3.8.0";
const CLASS_FLAGS: u32 = 0;
const EDITOR_CATEGORY: &str = "Component Controller Class";
const PROCESSOR_CATEGORY: &str = "Audio Module Class";

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
                copy_class_info(info, Self::PROCESSOR_CID, P::NAME, PROCESSOR_CATEGORY);
                kResultOk
            }
            1 => {
                copy_class_info(info, Self::EDITOR_CID, P::NAME, EDITOR_CATEGORY);
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
                    .expect("The AudioProcessor should implement FUnknown"),
            )
        } else if cid == Self::EDITOR_CID {
            Some(
                ComWrapper::new(EditController::<P::Editor>::new())
                    .to_com_ptr::<FUnknown>()
                    .expect("The EditController should implement FUnknown"),
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

fn copy_class_info(info: &mut PClassInfo, cid: TUID, name: &str, category: &str) {
    info.cid = cid;
    info.cardinality = ClassCardinality_::kManyInstances as _;
    strcpy(category, &mut info.category);
    strcpy(name, &mut info.name);
}

impl<P: VST3Plugin> IPluginFactory2Trait for Factory<P> {
    unsafe fn getClassInfo2(&self, index: i32, info: *mut PClassInfo2) -> tresult {
        let Some(info) = (unsafe { info.as_mut() }) else {
            return kInvalidArgument;
        };

        match index {
            0 => {
                copy_class_info2(
                    info,
                    Self::PROCESSOR_CID,
                    P::NAME,
                    PROCESSOR_CATEGORY,
                    &P::CATEGORIES,
                    P::VENDOR,
                );
                kResultOk
            }
            1 => {
                copy_class_info2(
                    info,
                    Self::EDITOR_CID,
                    P::NAME,
                    EDITOR_CATEGORY,
                    &P::CATEGORIES,
                    P::VENDOR,
                );
                kResultOk
            }
            _ => kInvalidArgument,
        }
    }
}

fn copy_class_info2(
    info: &mut PClassInfo2,
    cid: TUID,
    name: &str,
    category: &str,
    sub_categories: &VST3Categories,
    vendor: &str,
) {
    info.cid = cid;
    info.cardinality = ClassCardinality_::kManyInstances as _;
    strcpy(category, &mut info.category);
    strcpy(name, &mut info.name);
    info.classFlags = CLASS_FLAGS;
    strcpy(&sub_categories.to_string(), &mut info.subCategories);
    strcpy(vendor, &mut info.vendor);
    //info.version: [char8; 64],
    strcpy(SDK_VERSION_STRING, info.sdkVersion.as_mut_slice());
}

impl<P: VST3Plugin> IPluginFactory3Trait for Factory<P> {
    unsafe fn getClassInfoUnicode(
        &self,
        index: vst3::Steinberg::int32,
        info: *mut PClassInfoW,
    ) -> tresult {
        let Some(info) = (unsafe { info.as_mut() }) else {
            return kInvalidArgument;
        };

        match index {
            0 => {
                copy_class_infow(
                    info,
                    Self::PROCESSOR_CID,
                    P::NAME,
                    PROCESSOR_CATEGORY,
                    &P::CATEGORIES,
                    P::VENDOR,
                );
                kResultOk
            }
            1 => {
                copy_class_infow(
                    info,
                    Self::EDITOR_CID,
                    P::NAME,
                    EDITOR_CATEGORY,
                    &P::CATEGORIES,
                    P::VENDOR,
                );
                kResultOk
            }
            _ => kInvalidArgument,
        }
    }

    unsafe fn setHostContext(&self, context: *mut FUnknown) -> tresult {
        if let Some(host_application) = unsafe { HostApplication::from_raw(context) } {
            self.host_context.set(host_application).ok();
            kResultOk
        } else {
            kInvalidArgument
        }
    }
}

fn copy_class_infow(
    info: &mut PClassInfoW,
    cid: TUID,
    name: &str,
    category: &str,
    sub_categories: &VST3Categories,
    vendor: &str,
) {
    info.cid = cid;
    info.cardinality = ClassCardinality_::kManyInstances as _;
    strcpy(category, &mut info.category);
    strcpyw(name, &mut info.name);
    info.classFlags = CLASS_FLAGS;
    strcpy(&sub_categories.to_string(), &mut info.subCategories);
    strcpyw(vendor, &mut info.vendor);
    //info.version: [char8; 64],
    strcpyw(SDK_VERSION_STRING, info.sdkVersion.as_mut_slice());
}
