#[cfg(target_os = "windows")]
mod win;

#[cfg(target_os = "macos")]
pub(crate) mod mac;

#[cfg(target_os = "macos")]
pub use mac::*;
#[cfg(target_os = "windows")]
pub use win::*;

mod shared;
pub use shared::{WindowEvent, WindowHandler};
