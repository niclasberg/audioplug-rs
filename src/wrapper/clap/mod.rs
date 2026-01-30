pub use clap_sys::{entry::clap_plugin_entry, version::CLAP_VERSION};

#[macro_export]
macro_rules! audioplug_clap_plugin {
    ($plugin: ty) => {
        #[unsafe(no_mangle)]
        #[used]
        static clap_entry: $crate::wrapper::clap::clap_plugin_entry = {
            $crate::wrapper::clap::clap_plugin_entry {
                clap_version: $crate::wrapper::clap::CLAP_VERSION,
                init: None,
                deinit: None,
                get_factory: None,
            }
        };
    };
}
