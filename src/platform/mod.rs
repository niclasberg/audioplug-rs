#[cfg(target_os = "windows")]
mod win;

#[cfg(target_os = "macos")]
mod mac;

#[cfg(target_os = "windows")]
pub use win::*;
#[cfg(target_os = "macos")]
pub use mac::*;

mod shared;
pub use shared::WindowEvent;