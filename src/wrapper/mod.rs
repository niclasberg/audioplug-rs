pub mod standalone;
pub mod vst3;

#[cfg(target_os = "macos")]
pub mod au;

#[macro_export]
#[cfg(target_os = "windows")]
macro_rules! audioplug_auv3_plugin {
    ($plugin: ty) => {
        
    };
}


#[macro_export]
#[cfg(target_os = "macos")]
macro_rules! audioplug_auv3_plugin{
    ($plugin: ty) => {
        #[no_mangle]
        pub unsafe extern "C" fn AUV3_create_view_controller() -> *mut std::ffi::c_void {
            Box::into_raw(Box::new($crate::wrapper::au::ViewController::<$plugin>::new())) as *mut _
        }

        #[no_mangle]
        pub unsafe extern "C" fn AUV3_destroy_view_controller(view_controller: *mut std::ffi::c_void) {
            drop(unsafe { Box::from_raw(view_controller as *mut $crate::wrapper::au::ViewController::<$plugin>) });
        }

        #[no_mangle]
        pub unsafe extern "C" fn AUV3_create_audio_unit(view_controller: *mut std::ffi::c_void, desc: $crate::wrapper::au::audio_toolbox::AudioComponentDescription, error: *mut *mut $crate::wrapper::au::NSError) -> *mut std::ffi::c_void {
            (&mut *(view_controller as *mut $crate::wrapper::au::ViewController::<$plugin>)).create_audio_unit(desc, error) as *mut _
        }

        #[no_mangle]
        pub unsafe extern "C" fn AUV3_create_view(view_controller: *mut std::ffi::c_void) -> *mut std::ffi::c_void {
            (&mut *(view_controller as *mut $crate::wrapper::au::ViewController::<$plugin>)).create_view() as *mut _
        }

        #[no_mangle]
        pub unsafe extern "C" fn AUV3_preferred_content_size(view_controller: *mut std::ffi::c_void) -> $crate::wrapper::au::CGSize {
            (&mut *(view_controller as *mut $crate::wrapper::au::ViewController::<$plugin>)).preferred_size()
        }
    };
}

#[macro_export]
#[cfg(target_os = "windows")]
macro_rules! audioplug_vst3_plugin{
    ($plugin: ty) => {
        #[no_mangle]
        #[allow(non_snake_case)]
        pub extern "system" fn InitDll() -> bool {
            true
        }
        
        #[no_mangle]
        #[allow(non_snake_case)]
        pub extern "system" fn ExitDll() -> bool {
            true
        }

        #[no_mangle]
        #[allow(non_snake_case)]
        pub unsafe extern "system" fn GetPluginFactory() -> *mut std::ffi::c_void {
            Box::into_raw($crate::wrapper::vst3::Factory::<$plugin>::new()) as *mut std::ffi::c_void
        }
    };
}

#[macro_export]
#[cfg(target_os = "macos")]
macro_rules! audioplug_vst3_plugin{
    ($plugin: ty) => {
        #[no_mangle]
        #[allow(non_snake_case)]
        pub extern "system" fn bundleEntry() -> bool {
            true
        }

        #[no_mangle]
        #[allow(non_snake_case)]
        pub extern "system" fn bundleExit() -> bool {
            true
        }

        #[no_mangle]
        #[allow(non_snake_case)]
        pub unsafe extern "system" fn GetPluginFactory() -> *mut std::ffi::c_void {
            Box::into_raw($crate::wrapper::vst3::Factory::<$plugin>::new()) as *mut std::ffi::c_void
        }
    };
}
