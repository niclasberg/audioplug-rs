pub mod standalone;
pub mod vst3;

#[cfg(target_os = "macos")]
pub mod auv3;

#[macro_export]
#[cfg(any(target_os = "windows", target_os = "linux"))]
macro_rules! audioplug_auv3_plugin {
    ($plugin: ty) => {};
}

#[macro_export]
#[cfg(target_os = "macos")]
macro_rules! audioplug_auv3_plugin {
    ($plugin: ty) => {
        #[unsafe(no_mangle)]
        pub unsafe extern "C" fn AUV3_create_view_controller() -> *mut std::ffi::c_void {
            let vc = Box::new($crate::wrapper::auv3::ViewController::<$plugin>::new());
            Box::into_raw(vc) as *mut _
        }

        #[unsafe(no_mangle)]
        pub unsafe extern "C" fn AUV3_destroy_view_controller(
            view_controller: *mut std::ffi::c_void,
        ) {
            let vc = Box::from_raw(
                view_controller as *mut $crate::wrapper::auv3::ViewController<$plugin>,
            );
            drop(vc);
        }

        #[unsafe(no_mangle)]
        pub unsafe extern "C" fn AUV3_create_audio_unit(
            view_controller: *mut std::ffi::c_void,
            desc: $crate::wrapper::auv3::AudioComponentDescription,
            error: *mut *mut $crate::wrapper::auv3::NSError,
        ) -> *mut std::ffi::c_void {
            $crate::wrapper::auv3::ViewController::<$plugin>::create_audio_unit(
                &mut *(view_controller as *mut _),
                desc,
                error,
            ) as *mut _
        }

        #[unsafe(no_mangle)]
        pub unsafe extern "C" fn AUV3_create_view(
            view_controller: *mut std::ffi::c_void,
        ) -> *mut std::ffi::c_void {
            $crate::wrapper::auv3::ViewController::<$plugin>::create_view(
                &mut *(view_controller as *mut _),
            ) as *mut _
        }

        #[unsafe(no_mangle)]
        pub unsafe extern "C" fn AUV3_preferred_content_size(
            view_controller: *mut std::ffi::c_void,
        ) -> $crate::wrapper::auv3::CGSize {
            $crate::wrapper::auv3::ViewController::<$plugin>::preferred_size(
                &mut *(view_controller as *mut _),
            )
        }

        #[unsafe(no_mangle)]
        pub unsafe extern "C" fn AUV3_view_did_layout_subviews(
            view_controller: *mut std::ffi::c_void,
        ) {
        }
    };
}

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

        #[cfg(target_os="linux")]
        #[unsafe(no_mangle)]
        pub extern "system" fn ModuleEntry(_library_handle: *mut std::ffi::c_void) -> bool {
            true
        }
        
        #[cfg(target_os="linux")]
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
