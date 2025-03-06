use std::cell::RefCell;

use crate::{app::{Accessor, BuildContext, RenderContext, Widget, WidgetMut}, core::{Color, Size}, style::{DisplayStyle, Measure}, text::TextLayout};

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
    fn measure(&self, 
        _style: &crate::style::Style,
        width: Option<f64>, 
        height: Option<f64>, 
        available_width: taffy::AvailableSpace, 
        available_height: taffy::AvailableSpace) -> Size
    {
        let mut text_layout = self.text_layout.borrow_mut();

        let width_constraint = width.unwrap_or(match available_width {
            taffy::AvailableSpace::MinContent => text_layout.min_word_width(),
            taffy::AvailableSpace::MaxContent => f64::INFINITY,
            taffy::AvailableSpace::Definite(width) => width.into(),
        });

        let height_constraint = height.unwrap_or(match available_height {
            taffy::AvailableSpace::MinContent => f64::INFINITY,
            taffy::AvailableSpace::MaxContent => f64::INFINITY,
            taffy::AvailableSpace::Definite(height) => height.into(),
        }); 

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