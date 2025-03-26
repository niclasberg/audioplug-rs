use std::cell::RefCell;

use crate::{app::{Accessor, BuildContext, RenderContext, TextLayout, Widget, WidgetMut}, core::{Color, Size}, style::{AvailableSpace, DisplayStyle, Measure}};

use super::View;

pub struct Label {
    pub text: Accessor<String>,
	color: Accessor<Color>
}

impl Label {
    pub fn new(str: impl Into<Accessor<String>>) -> Self {
        Self { text: str.into(), color: Accessor::Const(Color::BLACK ) }
    }

	pub fn color(mut self, color: impl Into<Accessor<Color>>) -> Self {
		self.color = color.into();
		self
	}
}

impl View for Label {
    type Element = TextWidget;

    fn build(self, ctx: &mut BuildContext<Self::Element>) -> Self::Element {
        let text = self.text.get_and_bind(ctx, |value, mut widget: WidgetMut<'_, Self::Element>| {
            widget.text_layout.replace_with(|_text_layout| {
				TextLayout::new(value.as_str(), widget.color, Size::INFINITY)
			});
            widget.request_layout();
        });
        let color = self.color.get_and_bind(ctx, |value, mut widget: WidgetMut<'_, Self::Element>| {
            widget.request_render();
            let mut text_layout = widget.text_layout.borrow_mut();
			text_layout.set_color(value);
        });

        let text_layout = RefCell::new(TextLayout::new(text.as_str(), color, Size::INFINITY));
        TextWidget { text_layout, color }
    }
}

pub struct TextWidget {
    text_layout: RefCell<TextLayout>,
	color: Color
}

impl Measure for TextWidget {
    fn measure(&self, _style: &crate::style::Style, width: AvailableSpace, height: AvailableSpace) -> Size
    {
        let mut text_layout = self.text_layout.borrow_mut();

        let width_constraint = match width {
            AvailableSpace::MinContent => text_layout.min_word_width(),
            AvailableSpace::MaxContent => f64::INFINITY,
            AvailableSpace::Exact(width) => width,
        };

        let height_constraint = match height {
            AvailableSpace::MinContent => f64::INFINITY,
            AvailableSpace::MaxContent => f64::INFINITY,
            AvailableSpace::Exact(height) => height,
        }; 

        text_layout.set_max_size(Size::new(width_constraint, height_constraint));
        text_layout.measure()
    }
}

impl Widget for TextWidget {
	fn debug_label(&self) -> &'static str {
		"Label"
	}

    fn display_style(&self) -> DisplayStyle {
        DisplayStyle::Leaf(self)
    }

    fn render(&mut self, ctx: &mut RenderContext) {
		let mut text_layout = self.text_layout.borrow_mut();
        let bounds = ctx.content_bounds();
        text_layout.set_max_size(bounds.size());
        ctx.draw_text(&text_layout, bounds.top_left())
    }
}