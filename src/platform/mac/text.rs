use std::{ffi::c_void, mem::MaybeUninit, ops::Deref, ptr::NonNull};

use crate::core::{Color, FontFamily, FontOptions, FontStyle, FontWeight, Point, Rectangle, Size};
use objc2_app_kit::{NSFontWeightBold, NSFontWeightRegular};
use objc2_core_foundation::{
    CFDictionary, CFIndex, CFMutableAttributedString, CFMutableDictionary, CFNumber, CFRange,
    CFRetained, CFString, CFType, CGFloat, CGPoint, CGRect, CGSize,
};
use objc2_core_graphics::{CGColor, CGPath};
use objc2_core_text::{
    CTFont, CTFontDescriptor, CTFrame, CTFramesetter, CTLine, kCTFontAttributeName,
    kCTFontFamilyNameAttribute, kCTFontSlantTrait, kCTFontTraitsAttribute, kCTFontWeightTrait,
    kCTForegroundColorAttributeName,
};

use super::conversions::{cfrange_contains, cfstring_from_str, cgcolor_from_color};

pub struct TextLine {
    line: CFRetained<CTLine>,
    origin: CGPoint,
    char_range: CFRange,
    descent: f64,
}

impl TextLine {
    pub fn new(line: CFRetained<CTLine>, origin: CGPoint, descent: f64) -> Self {
        let char_range = unsafe { line.string_range() };
        Self {
            line,
            origin,
            char_range,
            descent,
        }
    }

    pub fn offset_for_string_index(&self, index: CFIndex) -> CGFloat {
        unsafe {
            self.line
                .offset_for_string_index(index, std::ptr::null_mut())
        }
    }

    pub fn string_index_for_position(&self, point: CGPoint) -> CFIndex {
        unsafe { self.line.string_index_for_position(point) }
    }
}

pub struct TextFrame {
    frame: CFRetained<CTFrame>,
    lines: Vec<TextLine>,
}

impl TextFrame {
    fn new(frame_setter: &CTFramesetter, string_range: CFRange, max_size: CGSize) -> Self {
        let (string_range, size) = get_suggest_frame_size_with_constraints(
            frame_setter,
            string_range,
            None,
            max_size.into(),
        );

        let rect = Rectangle::from_origin(Point::ZERO, size.into());
        let path = unsafe { CGPath::with_rect(rect.into(), std::ptr::null()) };
        let frame = unsafe { frame_setter.frame(string_range, &path, None) };
        let lines = get_lines_from_frame(&frame);

        Self { frame, lines }
    }

    pub fn get_visible_string_range(&self) -> CFRange {
        unsafe { self.frame.visible_string_range() }
    }

    pub fn path(&self) -> CFRetained<CGPath> {
        unsafe { self.frame.path() }
    }

    pub fn bounding_box(&self) -> CGRect {
        unsafe { CGPath::bounding_box(Some(&self.path())) }
    }
}

impl Deref for TextFrame {
    type Target = CTFrame;

    fn deref(&self) -> &Self::Target {
        &self.frame
    }
}

fn get_suggest_frame_size_with_constraints(
    frame_setter: &CTFramesetter,
    string_range: CFRange,
    frame_attributes: Option<&CFDictionary>,
    constraints: CGSize,
) -> (CFRange, CGSize) {
    unsafe {
        let mut fit_range = MaybeUninit::<CFRange>::uninit();
        let result = frame_setter.suggest_frame_size_with_constraints(
            string_range,
            frame_attributes,
            constraints,
            fit_range.as_mut_ptr(),
        );
        let fit_range = fit_range.assume_init();
        (fit_range, result)
    }
}

fn get_lines_from_frame(frame: &CTFrame) -> Vec<TextLine> {
    let lines = unsafe {
        let lines_array = frame.lines();
        let count = lines_array.count() as usize;
        let mut lines = Vec::<CFRetained<CTLine>>::new();

        unsafe extern "C-unwind" fn push_line(line_ptr: *const c_void, lines_vec: *mut c_void) {
            let line = unsafe { NonNull::new_unchecked(std::mem::transmute(line_ptr)) };
            let lines = unsafe { &mut *(lines_vec as *mut Vec<CFRetained<CTLine>>) };
            lines.push(unsafe { CFRetained::retain(line) });
        }

        lines_array.apply_function(
            CFRange {
                location: 0,
                length: count as _,
            },
            Some(push_line),
            &mut lines as *mut _ as *mut _,
        );
        lines
    };

    let origins = {
        let mut origins = Vec::with_capacity(lines.len());
        unsafe {
            frame.line_origins(
                CFRange {
                    location: 0,
                    length: lines.len() as _,
                },
                NonNull::new_unchecked(origins.as_mut_ptr()),
            );
            origins.set_len(lines.len());
        }
        origins
    };

    lines
        .into_iter()
        .zip(origins.into_iter())
        .map(|(line, origin)| {
            let mut descent = 0.0;
            unsafe {
                line.typographic_bounds(
                    std::ptr::null_mut(),
                    &mut descent as *mut _,
                    std::ptr::null_mut(),
                )
            };
            TextLine::new(line, origin, descent as _)
        })
        .collect()
}

pub struct NativeTextLayout {
    pub(super) attributed_string: CFRetained<CFMutableAttributedString>,
    pub(super) frame_setter: CFRetained<CTFramesetter>,
    text_frame: TextFrame,
    max_size: CGSize,
    text: String,
    default_height: f64,
}

impl NativeTextLayout {
    pub fn new(string: &str, font: &NativeFont, max_size: Size, color: Color) -> Self {
        let string = cfstring_from_str(&string);
        let text = string.to_string();
        let mut builder = AttributedStringBuilder::new(&string);
        let color = cgcolor_from_color(color);
        builder.set_foreground_color(builder.range(), &color);
        builder.set_font(builder.range(), &font.0);
        let string_range = builder.range();
        let attributed_string = builder.0;
        let frame_setter = unsafe { CTFramesetter::with_attributed_string(&attributed_string) };

        let text_frame = TextFrame::new(&frame_setter, string_range, max_size.into());

        let default_height = unsafe { font.0.ascent() + font.0.descent() + font.0.leading() };

        Self {
            attributed_string,
            frame_setter,
            text_frame,
            max_size: max_size.into(),
            text,
            default_height,
        }
    }

    pub fn length(&self) -> isize {
        self.attributed_string.length()
    }

    pub fn range(&self) -> CFRange {
        CFRange {
            location: 0,
            length: self.length(),
        }
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
        if self.text.is_empty() {
            Size::new(0.0, self.default_height)
        } else {
            self.text_frame.bounding_box().size.into()
        }
    }

    pub fn text_index_at_point(&self, point: Point) -> Option<usize> {
        self.text_frame.lines.iter().find_map(|line| {
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
        self.text_frame
            .lines
            .iter()
            .find(|line| cfrange_contains(&line.char_range, index))
            .map(|line| {
                let origin: Point = line.origin.into();
                let line_index = index - line.char_range.location;
                let offset = line.offset_for_string_index(line_index);
                origin + Point::new(offset, -line.descent)
            })
            .unwrap_or(Point::ZERO)
    }

    pub fn frame(&self) -> &TextFrame {
        &self.text_frame
    }

    pub fn as_str(&self) -> &str {
        &self.text
    }

    pub fn min_word_width(&self) -> f64 {
        self.text_frame.bounding_box().size.width
    }
}

pub struct AttributedStringBuilder(pub CFRetained<CFMutableAttributedString>);

impl AttributedStringBuilder {
    pub fn new(str: &CFString) -> Self {
        let attr_str = CFMutableAttributedString::new(None, 0).unwrap();
        unsafe {
            CFMutableAttributedString::replace_string(
                Some(&attr_str),
                CFRange {
                    location: 0,
                    length: 0,
                },
                Some(&str),
            )
        };
        Self(attr_str)
    }

    pub fn length(&self) -> isize {
        self.0.length()
    }

    pub fn range(&self) -> CFRange {
        CFRange {
            location: 0,
            length: self.length(),
        }
    }

    pub fn set_foreground_color(&mut self, range: CFRange, color: &CGColor) {
        self.set_attribute(range, unsafe { kCTForegroundColorAttributeName }, color)
    }

    pub fn set_font(&mut self, range: CFRange, font: &CTFont) {
        self.set_attribute(range, unsafe { kCTFontAttributeName }, font);
    }

    fn set_attribute(&mut self, range: CFRange, attr_name: &CFString, value: impl AsRef<CFType>) {
        unsafe {
            CFMutableAttributedString::set_attribute(
                Some(&self.0),
                range,
                Some(attr_name),
                Some(value.as_ref()),
            )
        }
    }
}

pub struct NativeFont(CFRetained<CTFont>);

impl NativeFont {
    pub fn new(options: &FontOptions) -> Self {
        let attributes = CFMutableDictionary::<_, CFType>::empty();

        let family = match &options.family {
            FontFamily::Name(name) => CFString::from_str(name),
            FontFamily::Serif => CFString::from_str("Times New Roman"),
            FontFamily::SansSerif => CFString::from_str("Arial"),
        };
        attributes.add(unsafe { kCTFontFamilyNameAttribute }, family.as_ref());

        let weight = match options.weight {
            FontWeight::Normal => unsafe { NSFontWeightRegular },
            FontWeight::Bold => unsafe { NSFontWeightBold },
        };

        let slant = match options.style {
            FontStyle::Normal => 0.0,
            FontStyle::Italic => 0.2,
            FontStyle::Oblique => 0.1,
        };

        let traits = CFDictionary::<_, CFType>::from_slices(
            &[unsafe { kCTFontWeightTrait }, unsafe { kCTFontSlantTrait }],
            &[
                CFNumber::new_f64(weight).as_ref(),
                CFNumber::new_f64(slant).as_ref(),
            ],
        );
        attributes.add(unsafe { kCTFontTraitsAttribute }, &traits);

        let descriptor = unsafe { CTFontDescriptor::with_attributes(&attributes.as_opaque()) };
        let font =
            unsafe { CTFont::with_font_descriptor(&descriptor, options.size, std::ptr::null()) };
        Self(font)
    }

    pub fn family_name(&self) -> String {
        unsafe { self.0.family_name() }.to_string()
    }
}
