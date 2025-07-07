#[cfg(target_os = "windows")]
mod win;

#[cfg(target_os = "macos")]
pub(crate) mod mac;

#[cfg(target_os = "macos")]
pub use mac::*;
#[cfg(target_os = "windows")]
pub use win::*;

mod brush;
mod shape;
mod shared;
mod text;
pub use brush::{Brush, BrushRef, LinearGradient, RadialGradient};
pub use shape::{PathGeometry, PathGeometryBuilder, Shape, ShapeRef};
pub use shared::{WindowEvent, WindowHandler};
pub use text::{Font, TextLayout};
