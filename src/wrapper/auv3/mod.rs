// An AudioUnit v3 plugin is built as an app extension (https://developer.apple.com/documentation/technologyoverviews/app-extensions)
// It consist of:
// - An AudioUnit (implements the audio processing)
// - An NSView (for the UI)
// - A ViewController (acts as an entry point which creates the view and the audio unit)
// In order for the compiled app extension to have the correct binary format, we have to compile it with
// clang (it needs the _NSExtensionMain instead of a regular main function).
// We therefore implement the actual viewcontroller class in objective C and expose a small c api
// that the view controller interacts with.

pub mod audio_unit;
mod buffers;
mod render_event;
mod utils;
mod view_controller;

pub use audio_unit::MyAudioUnit;
pub use objc2_audio_toolbox::AudioComponentDescription;
pub use objc2_core_foundation::CGSize;
pub use objc2_foundation::NSError;
pub use view_controller::ViewController;
