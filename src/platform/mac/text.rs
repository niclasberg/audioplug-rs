use std::{ffi::c_void, mem::MaybeUninit, ops::Deref, ptr::NonNull};

use objc2_core_foundation::{CFArrayApplyFunction, CFArrayGetCount, CFAttributedStringCreateMutable, CFAttributedStringGetLength, CFAttributedStringReplaceString, CFAttributedStringSetAttribute, CFDictionary, CFIndex, CFMutableAttributedString, CFRange, CFRetained, CFString, CFType, CGFloat, CGPoint, CGRect, CGSize};
use objc2_core_graphics::{CGColor, CGPath, CGPathCreateWithRect, CGPathGetBoundingBox};
use objc2_core_text::{kCTFontAttributeName, kCTForegroundColorAttributeName, CTFont, CTFrame, CTFrameGetLineOrigins, CTFrameGetLines, CTFrameGetPath, CTFrameGetVisibleStringRange, CTFramesetter, CTFramesetterCreateFrame, CTFramesetterCreateWithAttributedString, CTFramesetterSuggestFrameSizeWithConstraints, CTLine, CTLineGetOffsetForStringIndex, CTLineGetStringIndexForPosition, CTLineGetStringRange};
use crate::{text::FontWeight, core::{Color, Size, Point, Rectangle}};

use super::conversions::{cfrange_contains, cfstring_from_str, cgcolor_from_color};

struct TextLine {
	line: CFRetained<CTLine>,
	origin: CGPoint,
	char_range: CFRange
}

impl TextLine {
	pub fn new(line: CFRetained<CTLine>, origin: CGPoint) -> Self {
		let char_range = unsafe { CTLineGetStringRange(&line) };
		Self { line, origin, char_range }
	}

	pub fn offset_for_string_index(&self, index: CFIndex) -> CGFloat {
		unsafe {
			CTLineGetOffsetForStringIndex(&self.line, index, std::ptr::null_mut())
		}
	}

	pub fn string_index_for_position(&self, point: CGPoint) -> CFIndex {
		unsafe {
			CTLineGetStringIndexForPosition(&self.line, point)
		}
	}
}

struct TextFrame {
	frame: CFRetained<CTFrame>,
	lines: Vec<TextLine>
}

impl TextFrame {
	fn new(frame_setter: &CTFramesetter, string_range: CFRange, max_size: CGSize) -> Self {
		let (string_range, size) = get_suggest_frame_size_with_constraints(frame_setter, string_range, None, max_size.into());

		let rect = Rectangle::new(Point::ZERO, size.into());
		let path = unsafe { CGPathCreateWithRect(rect.into(), std::ptr::null()) };
		let frame = unsafe { CTFramesetterCreateFrame(&frame_setter, string_range, &path, None) };
		let lines = get_lines_from_frame(&frame);

		Self {
			frame,
			lines
		}
	}

	pub fn get_visible_string_range(&self) -> CFRange {
		unsafe {
			CTFrameGetVisibleStringRange(&self.frame)
		}
	}

	pub fn path(&self) -> CFRetained<CGPath> {
		unsafe { CTFrameGetPath(&self.frame) }
	}

	pub fn bounding_box(&self) -> CGRect {
		unsafe { CGPathGetBoundingBox(Some(&self.path())) }
	}
}

impl Deref for TextFrame {
	type Target = CTFrame;

	fn deref(&self) -> &Self::Target {
		&self.frame
	}
}

fn get_suggest_frame_size_with_constraints(frame_setter: &CTFramesetter, string_range: CFRange, frame_attributes: Option<&CFDictionary>, constraints: CGSize) -> (CFRange, CGSize) {
	unsafe {
		let mut fit_range = MaybeUninit::<CFRange>::uninit();
		let result = CTFramesetterSuggestFrameSizeWithConstraints(&frame_setter, string_range, frame_attributes, constraints, fit_range.as_mut_ptr());
		let fit_range = fit_range.assume_init();
		(fit_range, result)
	}
}

fn get_lines_from_frame(frame: &CTFrame) -> Vec<TextLine> {
	let lines = unsafe {
		let lines_array = CTFrameGetLines(&frame);
		let count = CFArrayGetCount(&lines_array) as usize;
		let mut lines = Vec::<CFRetained<CTLine>>::new();

		unsafe extern "C-unwind" fn push_line(line_ptr: *const c_void, lines_vec: *mut c_void) {
			let lines = &mut *(lines_vec as *mut Vec<CFRetained<CTLine>>);
			let line = NonNull::new_unchecked(std::mem::transmute(line_ptr));
			lines.push(CFRetained::retain(line));
		}

		CFArrayApplyFunction(&lines_array, CFRange { location: 0, length: count as _ }, Some(push_line), &mut lines as *mut _ as *mut _);
		lines
	};

	let origins = {
		let mut origins = Vec::with_capacity(lines.len());
		unsafe {
			CTFrameGetLineOrigins(&frame, CFRange { location: 0, length: lines.len() as _ }, NonNull::new_unchecked(origins.as_mut_ptr()));
			origins.set_len(lines.len());
		}
		origins
	};

	lines.into_iter().zip(origins.into_iter())
		.map(|(line, origin)| TextLine::new(line, origin)).collect()
}

pub struct TextLayout{
    pub(super) attributed_string: CFRetained<CFMutableAttributedString>,
	pub(super) frame_setter: CFRetained<CTFramesetter>,
	text_frame: TextFrame,
	max_size: CGSize,
	text: String
}

impl TextLayout {
    pub fn new(
        string: &str, 
        _font_family: &str, 
        _font_weight: FontWeight,
        _font_size: f32,
        max_size: Size,
		color: Color
    ) -> Self {
		let string = cfstring_from_str(&string);
		let text = string.to_string();
		let mut builder = AttributedStringBuilder::new(&string);
		let color = cgcolor_from_color(color);
		builder.set_foreground_color(builder.range(), &color);
		let string_range = builder.range();
		let attributed_string = builder.0;
		let frame_setter = unsafe { CTFramesetterCreateWithAttributedString(&attributed_string) };

		let text_frame = TextFrame::new(&frame_setter, string_range, max_size.into());

        Self {
			attributed_string,
			frame_setter,
			text_frame,
			max_size: max_size.into(),
			text
		}
    }

	pub fn length(&self) -> isize {
		unsafe { CFAttributedStringGetLength(&self.attributed_string) }
	}

	pub fn range(&self) -> CFRange {
		CFRange { location: 0, length: self.length() }
	}

    pub fn set_max_size(&mut self, size: Size) {
		let size = size.into();
		if self.max_size != size {
			self.max_size = size;
			let string_range = self.range();
			self.text_frame = TextFrame::new(&self.frame_setter, string_range, size);
		}
    }

	pub fn measure(&self) -> Size {
		self.text_frame.bounding_box().size.into()
	}

	pub fn text_index_at_point(&self, point: Point) -> Option<usize> {
        self.text_frame.lines.iter()
			.find_map(|line| {
				let origin: Point = (line.origin).into();
				let point: Point = point + origin;
				let point = (origin + point).into();
				let index = line.string_index_for_position(point);
				if index < 0 {
					None
				} else {
					let absolute_index = index + line.char_range.location;
					Some(absolute_index as usize)
				}
			})
    }

    pub fn point_at_text_index(&self, index: usize) -> Point {
		let index = index as CFIndex;
		self.text_frame.lines.iter()
			.find(|line| cfrange_contains(&line.char_range, index))
			.map(|line| {
				let origin: Point = line.origin.into();
				let line_index = index - line.char_range.location;
				let offset = line.offset_for_string_index(line_index);
				origin + Point::new(offset, 0.0)
			})
			.unwrap_or(Point::ZERO)
	}

	pub fn frame(&self) -> &TextFrame {
		&self.text_frame
	}

	pub fn as_str(&self) -> &str {
		&self.text
	}
}

pub struct AttributedStringBuilder(pub CFRetained<CFMutableAttributedString>);

impl AttributedStringBuilder {
	pub fn new(str: &CFString) -> Self {
		let attr_str = unsafe { CFAttributedStringCreateMutable(None, 0) }.unwrap();
		unsafe { CFAttributedStringReplaceString(Some(&attr_str), CFRange { location: 0, length: 0 }, Some(&str)) };
		Self(attr_str)
	}

	pub fn length(&self) -> isize {
		unsafe { CFAttributedStringGetLength(&self.0) }
	}

	pub fn range(&self) -> CFRange {
		CFRange { location: 0, length: self.length() }
	}

	pub fn set_foreground_color(&mut self, range: CFRange, color: &CGColor) {
		self.set_attribute(range, unsafe { kCTForegroundColorAttributeName }, color)
	}

	pub fn set_font(&mut self, range: CFRange, font: &CTFont) {
		self.set_attribute(range, unsafe { kCTFontAttributeName }, font);
	}

	fn set_attribute(&mut self, range: CFRange, attr_name: &CFString, value: impl AsRef<CFType>) {
		unsafe {
			CFAttributedStringSetAttribute(Some(&self.0), range, Some(attr_name), Some(value.as_ref()))
		}
	}
}