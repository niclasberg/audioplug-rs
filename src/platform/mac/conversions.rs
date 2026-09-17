use crate::core::{Color, Point, Rect, Size, Vec2};
use objc2_core_foundation::{
    CFIndex, CFRange, CFRetained, CFString, CFStringBuiltInEncodings, CGPoint, CGRect, CGSize,
};
use objc2_core_graphics::CGColor;

impl Into<CGPoint> for Point {
    fn into(self) -> CGPoint {
        CGPoint {
            x: self.x as _,
            y: self.y as _,
        }
    }
}

impl From<CGPoint> for Point {
    fn from(value: CGPoint) -> Self {
        Point::new(value.x as _, value.y as _)
    }
}

impl From<CGPoint> for Vec2 {
    fn from(value: CGPoint) -> Self {
        Vec2::new(value.x as _, value.y as _)
    }
}

impl Into<CGSize> for Size {
    fn into(self) -> CGSize {
        CGSize {
            width: self.width as _,
            height: self.height as _,
        }
    }
}

impl Into<CGSize> for Vec2 {
    fn into(self) -> CGSize {
        CGSize {
            width: self.x as _,
            height: self.y as _,
        }
    }
}

impl From<CGSize> for Size {
    fn from(value: CGSize) -> Self {
        Size {
            width: value.width as _,
            height: value.height as _,
        }
    }
}

impl Into<CGRect> for Rect {
    fn into(self) -> CGRect {
        CGRect {
            origin: self.top_left().into(),
            size: self.size().into(),
        }
    }
}

impl From<CGRect> for Rect {
    fn from(value: CGRect) -> Self {
        Rect::from_origin(value.origin.into(), value.size.into())
    }
}

pub fn cgcolor_from_color(color: Color) -> CFRetained<CGColor> {
    CGColor::new_srgb(
        color.r.into(),
        color.g.into(),
        color.b.into(),
        color.a.into(),
    )
}

pub fn cfstring_from_str(str: &str) -> CFRetained<CFString> {
    unsafe {
        CFString::with_bytes(
            None,
            str.as_ptr(),
            str.len() as CFIndex,
            CFStringBuiltInEncodings::EncodingUTF8.0,
            false,
        )
    }
    .unwrap()
}

pub fn cfrange_contains(cf_range: &CFRange, index: CFIndex) -> bool {
    index >= cf_range.location && (index + cf_range.location) <= cf_range.length
}

/*pub fn cfrange_as_range(cf_range: &CFRange) -> Range<isize> {
    Range::new(cf_range.location, cf_range.location + cf_range.length)
}*/
