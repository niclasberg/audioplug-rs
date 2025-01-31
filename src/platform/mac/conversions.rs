use objc2_core_foundation::{CFIndex, CFRetained, CFString, CFStringBuiltInEncodings, CFStringCreateWithBytes, CFStringEncoding, CGAffineTransform, CGPoint, CGRect, CGSize};
use objc2_core_graphics::{CGColor, CGColorCreateSRGB};
use crate::core::{Color, Point, Rectangle, Size, Transform};

impl Into<CGPoint> for Point {
    fn into(self) -> CGPoint {
        CGPoint { x: self.x, y: self.y }
    }
}

impl From<CGPoint> for Point {
    fn from(value: CGPoint) -> Self {
        Point::new(value.x, value.y)
    }
}

impl Into<CGSize> for Size {
    fn into(self) -> CGSize {
        CGSize { width: self.width, height: self.height }
    }
}

impl From<CGSize> for Size {
    fn from(value: CGSize) -> Self {
        Size::new(value.width, value.height)
    }
}

impl Into<CGRect> for Rectangle {
    fn into(self) -> CGRect {
        CGRect { origin: self.position().into(), size: self.size().into() }
    }
}

impl From<CGRect> for Rectangle {
    fn from(value: CGRect) -> Self {
        Rectangle::new(value.origin.into(), value.size.into())
    }
}

impl From<Transform> for CGAffineTransform {
    fn from(value: Transform) -> Self {
        CGAffineTransform { a: value.m11, b: value.m12, c: value.m21, d: value.m22, tx: value.tx, ty: value.ty }
    }
}

impl From<CGAffineTransform> for Transform {
    fn from(value: CGAffineTransform) -> Self {
        Transform { m11: value.a, m12: value.b, m21: value.c, m22: value.d, tx: value.tx, ty: value.ty }
    }
}

pub fn cgcolor_from_color(color: Color) -> CFRetained<CGColor> {
	unsafe { CGColorCreateSRGB(color.r.into(), color.g.into(), color.b.into(), color.a.into()) }
}

pub fn cfstring_from_str(str: &str) -> CFRetained<CFString> {
	unsafe {
		CFStringCreateWithBytes(
			None, 
			str.as_ptr(), 
			str.len() as CFIndex,
			CFStringBuiltInEncodings::EncodingUTF8.0, 
			false)
	}.unwrap()
}