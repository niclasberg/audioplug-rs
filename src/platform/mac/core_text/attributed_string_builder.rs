use crate::platform::{mac::{core_foundation::{CFString, CFAttributedString, CFRange}, core_graphics::CGColor}, IMut};

use super::CTFont;

pub struct AttributedStringBuilder(pub IMut<CFAttributedString>);

impl AttributedStringBuilder {
	pub fn new(str: &CFString) -> Self {
		let mut attr_str = CFAttributedString::new_mut(0);
		attr_str.replace_string(CFRange::empty(), &str);

		Self(attr_str)
	}

	pub fn range(&self) -> CFRange {
		CFRange { location: 0, length: self.0.length() }
	}

	pub fn set_foreground_color(&mut self, range: CFRange, color: &CGColor) {
		self.0.set_attribute(range, unsafe { &*kCTForegroundColorAttributeName }, color)
	}

	pub fn set_font(&mut self, range: CFRange, font: &CTFont) {
		self.0.set_attribute(range, unsafe { &*kCTFontAttributeName }, font);
	}
}

extern "C" {
	static kCTFontAttributeName: *const CFString;
	static kCTKernAttributeName: *const CFString;

	/// The type of ligatures to use.
	static kCTLigatureAttributeName: *const CFString;
	static kCTForegroundColorAttributeName: *const CFString;

	/// Sets a foreground color using the context's fill color.
	pub static kCTForegroundColorFromContextAttributeName: *const CFString;

	/// The paragraph style of the text to which this attribute applies.
	pub static kCTParagraphStyleAttributeName: *const CFString;

	/// The stroke width.
	pub static kCTStrokeWidthAttributeName: *const CFString;

	/// The stroke color.
	pub static kCTStrokeColorAttributeName: *const CFString;

	/// Controls vertical text positioning.
	pub static kCTSuperscriptAttributeName: *const CFString;

	/// The underline color.
	pub static kCTUnderlineColorAttributeName: *const CFString;

	/// The style of underlining, to be applied at render time, for the text to which this attribute applies.
	pub static kCTUnderlineStyleAttributeName: *const CFString;

	/// The orientation of the glyphs in the text to which this attribute applies.
	pub static kCTVerticalFormsAttributeName: *const CFString;

	///The glyph info object to apply to the text associated with this attribute.
	pub static kCTGlyphInfoAttributeName: *const CFString;

	/// The run-delegate object to apply to an attribute range of the string.
	pub static kCTRunDelegateAttributeName: *const CFString;

	/// Vertical offset for text position.
	pub static kCTBaselineOffsetAttributeName: *const CFString;

	/// The tracking for the text.
	pub static kCTTrackingAttributeName: *const CFString;
}