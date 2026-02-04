mod audio_busses;
mod audioprocessor;
mod category;
mod editcontroller;
mod factory;
mod host_application;
#[cfg(target_os = "linux")]
mod linux_runloop;
mod parameters;
mod plugview;
mod shared_state;
mod util;

pub use audioprocessor::AudioProcessor;
pub use category::{VST3Categories, VSTCategory};
pub use factory::Factory;

#[macro_export]
macro_rules! audioplug_vst3_plugin {
    ($plugin: ty) => {
        #[cfg(target_os = "windows")]
        #[unsafe(no_mangle)]
        #[allow(non_snake_case)]
        pub extern "system" fn InitDll() -> bool {
            true
        }

        #[cfg(target_os = "windows")]
        #[unsafe(no_mangle)]
        #[allow(non_snake_case)]
        pub extern "system" fn ExitDll() -> bool {
            true
        }

        #[cfg(target_os = "macos")]
        #[unsafe(no_mangle)]
        #[allow(non_snake_case)]
        pub extern "system" fn bundleEntry() -> bool {
            true
        }

        #[cfg(target_os = "macos")]
        #[unsafe(no_mangle)]
        #[allow(non_snake_case)]
        pub extern "system" fn bundleExit() -> bool {
            true
        }

        #[cfg(target_os = "linux")]
        #[unsafe(no_mangle)]
        pub extern "system" fn ModuleEntry(_library_handle: *mut std::ffi::c_void) -> bool {
            true
        }

        #[cfg(target_os = "linux")]
        #[unsafe(no_mangle)]
        pub extern "system" fn ModuleExit() -> bool {
            true
        }

        #[unsafe(no_mangle)]
        #[allow(non_snake_case)]
        pub unsafe extern "system" fn GetPluginFactory() -> *mut std::ffi::c_void {
            $crate::wrapper::vst3::Factory::<$plugin>::new_raw() as *mut std::ffi::c_void
        }
    };
}
