use crate::{app::{Accessor, BuildContext, LayoutContext, RenderContext, WidgetMut}, core::{Color, Size}, text::TextLayout};

use super::{View, Widget};

pub struct Label {
    pub text: Accessor<String>,
	color: Accessor<Color>
}

impl Label {
    pub fn new(str: impl Into<Accessor<String>>) -> Self {
        Self { text: str.into(), color: Accessor::Const(Color::BLACK ) }
    }

	pub fn with_color(mut self, color: impl Into<Accessor<Color>>) -> Self {
		self.color = color.into();
		self
	}
}

impl View for Label {
    type Element = TextWidget;

    fn build(self, ctx: &mut BuildContext) -> Self::Element {
        let text = ctx.get_and_track(self.text, |value, mut widget: WidgetMut<'_, Self::Element>| {
            let text_layout = TextLayout::new(value.as_str(), *widget.text_layout.color(), Size::INFINITY);
            widget.text_layout = text_layout;
            widget.request_layout();
        });
        let color = ctx.get_and_track(self.color, |value, mut widget: WidgetMut<'_, Self::Element>| {
            widget.text_layout.set_color(*value);
            //widget.request_render();
        });

        let text_layout = TextLayout::new(text.as_str(), color, Size::INFINITY);
        TextWidget { text_layout }
    }
}

pub struct TextWidget {
    text_layout: TextLayout
}

impl Widget for TextWidget {
    fn measure(&self, style: &taffy::Style, known_dimensions: taffy::Size<Option<f32>>, available_space: taffy::Size<taffy::AvailableSpace>) -> taffy::Size<f32> {
        let width_constraint = known_dimensions.width.unwrap_or(match available_space.width {
            taffy::AvailableSpace::MinContent => 0.0,
            taffy::AvailableSpace::MaxContent => f32::INFINITY,
            taffy::AvailableSpace::Definite(width) => width,
        }) as f64;

        let height_constraint = known_dimensions.height.unwrap_or(match available_space.height {
            taffy::AvailableSpace::MinContent => f32::INFINITY,
            taffy::AvailableSpace::MaxContent => f32::INFINITY,
            taffy::AvailableSpace::Definite(height) => height,
        }) as f64; 

        self.text_layout.set_max_size(Size::new(width_constraint, height_constraint));
        let measured_size: taffy::Size<f32> = self.text_layout.measure().map(|x| x as f32).into();
        
        known_dimensions.unwrap_or(measured_size)
    }

    fn render(&mut self, ctx: &mut RenderContext) {
        self.text_layout.set_max_size(ctx.global_bounds().size());
        ctx.draw_text(&self.text_layout, ctx.global_bounds().top_left())
    }
    
    fn style(&self) -> taffy::Style {
        Default::default()
    }
}