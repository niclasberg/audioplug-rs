pub mod audio_unit;
mod buffers;
mod view_controller;
mod utils;

pub use objc2_foundation::NSError;
pub use objc2_core_foundation::CGSize;
pub use audio_unit::MyAudioUnit;
pub use view_controller::ViewController;
pub use crate::platform::mac::audio_toolbox::AudioComponentDescription;
