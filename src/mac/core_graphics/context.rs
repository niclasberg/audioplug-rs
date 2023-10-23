use icrate::Foundation::CGFloat;
use objc2::{Encode, RefEncode};
use super::CGColor;
use super::CGRect;

#[repr(C)]
pub struct CGContext {
    _data: [u8; 0],
    _marker: core::marker::PhantomData<(*mut u8, core::marker::PhantomPinned)>,
}

unsafe impl Encode for CGContext {
    const ENCODING: objc2::Encoding = objc2::Encoding::Struct("CGContext", &[]);
}

unsafe impl RefEncode for CGContext {
	const ENCODING_REF: objc2::Encoding = objc2::Encoding::Pointer(&CGContext::ENCODING);
}

impl CGContext {
	pub fn flush(&self) {
		unsafe { CGContextFlush(self as _) }
	}

	pub fn set_fill_color(&self, color: &CGColor) {
		unsafe { CGContextSetFillColorWithColor(self, color) }
	}

	pub fn set_stroke_color(&self, color: &CGColor) {
		unsafe { CGContextSetStrokeColorWithColor(self, color)}
	}

	pub fn fill_rect(&self, rect: CGRect) {
		unsafe { CGContextFillRect(self, rect) }
	}

	pub fn stroke_rect(&self, rect: CGRect, width: CGFloat) {
		unsafe { CGContextStrokeRectWithWidth(self, rect, width)};
	}

	pub fn fill_ellipse_in_rect(&self, rect: CGRect) {
		unsafe { CGContextFillEllipseInRect(self, rect) }
	}

	pub fn translate(&self, tx: CGFloat, ty: CGFloat) {
		unsafe { CGContextTranslateCTM(self, tx, ty) };
	}

	pub fn move_to_point(&self, x: CGFloat, y: CGFloat) {
		unsafe { CGContextMoveToPoint(self, x, y) }
	}

	pub fn close_path(&self) {
		unsafe { CGContextClosePath(self) }
	}

	pub fn add_arc_to_point(&self, x1: CGFloat, y1: CGFloat, x2: CGFloat, y2: CGFloat, radius: CGFloat) {
		unsafe { CGContextAddArcToPoint(self, x1, y1, x2, y2, radius) }
	}

	pub fn fill_path(&self) {
		unsafe { CGContextFillPath(self) }
	}
}

#[link(name = "CoreGraphics", kind = "framework")]
extern "C" {
	fn CGContextFlush(c: *const CGContext);
	fn CGContextSetFillColorWithColor(c: *const CGContext, color: *const CGColor);
	fn CGContextSetStrokeColorWithColor(c: *const CGContext, color: *const CGColor);
	fn CGContextStrokeRectWithWidth(context: *const CGContext, rect: CGRect, width: CGFloat);
	fn CGContextFillRect(context: *const CGContext, rect: CGRect);
	fn CGContextTranslateCTM(context: *const CGContext, tx: CGFloat, ty: CGFloat);

	fn CGContextMoveToPoint(c: *const CGContext, x: CGFloat, y: CGFloat);
	fn CGContextClosePath(c: *const CGContext);
	fn CGContextAddArcToPoint(c: *const CGContext, x1: CGFloat, y1: CGFloat, x2: CGFloat, y2: CGFloat, radius: CGFloat);
	fn CGContextFillPath(c: *const CGContext);
	fn CGContextFillEllipseInRect(c: *const CGContext, rect: CGRect);

}