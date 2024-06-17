use icrate::Foundation::CGFloat;
use objc2::{Encode, RefEncode};
use super::CGAffineTransform;
use super::CGColor;
use super::CGRect;
use super::CGPoint;

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

	pub fn set_line_width(&self, width: CGFloat) {
		unsafe { CGContextSetLineWidth(self, width) }
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

	pub fn stroke_ellipse_in_rect(&self, rect: CGRect) {
		unsafe { CGContextStrokeEllipseInRect(self, rect) }
	}

	pub fn clip_to_rect(&self, rect: CGRect) {
		unsafe { CGContextClipToRect(self, rect) }
	}

	pub fn translate_ctm(&self, tx: CGFloat, ty: CGFloat) {
		unsafe { CGContextTranslateCTM(self, tx, ty) };
	}

	pub fn scale_ctm(&self, sx: CGFloat, sy: CGFloat) {
		unsafe { CGContextScaleCTM(self, sx, sy) }
	}

	pub fn concat_ctm(&self, transform: CGAffineTransform) {
		unsafe { CGContextConcatCTM(self, transform) }
	}

	pub fn get_ctm(&self) -> CGAffineTransform {
		unsafe { CGContextGetCTM(self) }
	}

	// Constructing paths
	pub fn move_to_point(&self, x: CGFloat, y: CGFloat) {
		unsafe { CGContextMoveToPoint(self, x, y) }
	}

	pub fn close_path(&self) {
		unsafe { CGContextClosePath(self) }
	}

	pub fn add_arc_to_point(&self, x1: CGFloat, y1: CGFloat, x2: CGFloat, y2: CGFloat, radius: CGFloat) {
		unsafe { CGContextAddArcToPoint(self, x1, y1, x2, y2, radius) }
	}

	pub fn add_line_to_point(&self, x: CGFloat, y: CGFloat) {
		unsafe { CGContextAddLineToPoint(self, x, y) }
	}

	pub fn fill_path(&self) {
		unsafe { CGContextFillPath(self) }
	}

	pub fn stroke_path(&self) {
		unsafe { CGContextStrokePath(self) }
	}

	pub fn set_text_matrix(&self, t: CGAffineTransform) {
		unsafe { CGContextSetTextMatrix(self, t) }
	}

	pub fn get_text_matrix(&self) -> CGAffineTransform {
		unsafe { CGContextGetTextMatrix(self) }
	}

	pub fn get_text_position(&self) -> CGPoint {
		unsafe { CGContextGetTextPosition(self) }
	}

	pub fn set_text_position(&self, x: CGFloat, y: CGFloat) {
		unsafe { CGContextSetTextPosition(self, x, y) }
	}

	pub fn save_state(&self) {
		unsafe { CGContextSaveGState(self) }
	}

	pub fn restore_state(&self) {
		unsafe { CGContextRestoreGState(self) }
	}

}

#[link(name = "CoreGraphics", kind = "framework")]
extern "C" {
	fn CGContextFlush(c: *const CGContext);
	fn CGContextSetFillColorWithColor(c: *const CGContext, color: *const CGColor);
	fn CGContextSetStrokeColorWithColor(c: *const CGContext, color: *const CGColor);
	fn CGContextSetLineWidth(c: *const CGContext, width: CGFloat);

	fn CGContextStrokeRectWithWidth(context: *const CGContext, rect: CGRect, width: CGFloat);
	fn CGContextFillRect(context: *const CGContext, rect: CGRect);
	
	fn CGContextTranslateCTM(context: *const CGContext, tx: CGFloat, ty: CGFloat);
	fn CGContextGetCTM(context: *const CGContext) -> CGAffineTransform;
	fn CGContextConcatCTM(context: *const CGContext, transform: CGAffineTransform);
	fn CGContextScaleCTM(context: *const CGContext, sx: CGFloat, sy: CGFloat);

	fn CGContextClipToRect(context: *const CGContext, rect: CGRect);

	fn CGContextMoveToPoint(c: *const CGContext, x: CGFloat, y: CGFloat);
	fn CGContextClosePath(c: *const CGContext);
	fn CGContextAddArcToPoint(c: *const CGContext, x1: CGFloat, y1: CGFloat, x2: CGFloat, y2: CGFloat, radius: CGFloat);
	fn CGContextAddLineToPoint(c: *const CGContext, x: CGFloat, y: CGFloat);
	fn CGContextFillPath(c: *const CGContext);
	fn CGContextStrokePath(c: *const CGContext);

	fn CGContextFillEllipseInRect(c: *const CGContext, rect: CGRect);
	fn CGContextStrokeEllipseInRect(c: *const CGContext, rect: CGRect);

	fn CGContextGetTextPosition(c: *const CGContext) -> CGPoint;
	fn CGContextSetTextPosition(c: *const CGContext, x: CGFloat, y: CGFloat);
	fn CGContextSetTextMatrix(c: *const CGContext, t: CGAffineTransform);
	fn CGContextGetTextMatrix(c: *const CGContext) -> CGAffineTransform;

	fn CGContextSaveGState(c: *const CGContext);
	fn CGContextRestoreGState(c: *const CGContext);
}