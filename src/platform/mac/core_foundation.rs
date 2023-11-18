use std::ops::Range;

pub type CFIndex = isize;

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CFRange {
    pub location: CFIndex,
    pub length: CFIndex,
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

#[repr(C)]
pub struct CFDictionary {
	_data: [u8; 0],
    _marker: core::marker::PhantomData<(*mut u8, core::marker::PhantomPinned)>,
}
