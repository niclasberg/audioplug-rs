mod factory;
mod features;
mod host;
mod plugin;

pub use clap_sys::{entry::clap_plugin_entry, version::CLAP_VERSION};
pub use factory::Factory;
pub use features::ClapFeature;

#[macro_export]
macro_rules! audioplug_clap_plugin {
    ($plugin: ty) => {
        #[unsafe(no_mangle)]
        #[used]
        static clap_entry: $crate::wrapper::clap::clap_plugin_entry = {
            use ::std::ffi::{c_char, c_void};
            use ::std::sync::LazyLock;
            use $crate::wrapper::clap::Factory;

            static FACTORY: LazyLock<Factory<$plugin>> = LazyLock::new(|| Factory::new());

            unsafe extern "C" fn get_factory(_factory_id: *const c_char) -> *const c_void {
                LazyLock::force(&FACTORY) as *const Factory<$plugin> as *const _
            }

            $crate::wrapper::clap::clap_plugin_entry {
                clap_version: $crate::wrapper::clap::CLAP_VERSION,
                init: None,
                deinit: None,
                get_factory: Some(get_factory),
            }
        };
    };
}
