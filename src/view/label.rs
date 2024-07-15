use crate::{core::{Color, Size}, text::TextLayout};

use super::{BuildContext, LayoutContext, RenderContext, View, Widget};

pub struct Label {
    pub text: String,
	color: Color
}

impl Label {
    pub fn new(str: impl Into<String>) -> Self {
        Self { text: str.into(), color: Color::BLACK }
    }

	pub fn with_color(mut self, color: Color) -> Self {
		self.color = color;
		self
	}
}

impl View for Label {
    type Element = TextWidget;

    fn build(self, _ctx: &mut BuildContext) -> Self::Element {
        let text_layout = TextLayout::new(self.text.as_str(), self.color, Size::INFINITY);
        TextWidget { text_layout }
    }
}

pub struct TextWidget {
    text_layout: TextLayout
}

impl Widget for TextWidget {
    fn layout(&mut self, inputs: taffy::LayoutInput, ctx: &mut LayoutContext) -> taffy::LayoutOutput {
        ctx.compute_leaf_layout(inputs, |known_dimensions, available_space| {
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
        })
    }

    fn render(&mut self, ctx: &mut RenderContext) {
        self.text_layout.set_max_size(ctx.global_bounds().size());
        ctx.draw_text(&self.text_layout, ctx.global_bounds().top_left())
    }
    
    fn style(&self) -> taffy::Style {
        Default::default()
    }
}