#[cfg(target_os = "macos")]
pub mod auv3;

pub mod clap;
pub mod standalone;
pub mod vst3;

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
