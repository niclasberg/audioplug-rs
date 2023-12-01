use std::{ops::Range, os::raw::c_void};
use icrate::Foundation::NSString;
mod attributed_string;
mod allocator;
mod dictionary;
mod number;
mod string;

pub use attributed_string::CFAttributedString;
pub use allocator::*;
pub use dictionary::CFDictionary;
pub use string::*;

pub type CFIndex = isize;
pub type CFTypeRef = *const c_void;

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CFRange {
    pub location: CFIndex,
    pub length: CFIndex,
}

impl CFRange {
	pub fn empty() -> Self {
		Self { location: 0, length: 0 }
	}
}

impl From<Range<isize>> for CFRange {
    fn from(value: Range<isize>) -> Self {
        CFRange { location: value.start, length: value.end - value.start }
    }
}

impl From<CFRange> for Range<isize> {
    fn from(value: CFRange) -> Self {
    	Range { start: value.location, end: value.location + value.length }
    }
}
