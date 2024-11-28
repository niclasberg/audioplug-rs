use crate::{app::{BuildContext, RenderContext, Widget}, core::{Color, Rectangle, Shape, Size}, style::{Length, Style}};
use super::View;


pub trait Fill {
    fn fill(self, color: Color) -> Filled;
}

impl Fill for Shape {
    fn fill(self, color: Color) -> Filled {
        Filled { shape: self, color}
    }
}

impl Fill for Rectangle {
    fn fill(self, color: Color) -> Filled {
        Filled { shape: self.into(), color}
    }
}

pub struct Filled {
    shape: Shape,
    color: Color
}

impl View for Filled {
    type Element = Self;

    fn build(self, ctx: &mut BuildContext<Self::Element>) -> Self { 
		let bounds = self.shape.bounds();
        ctx.set_style(Style {
            size: Size::new(Length::Px(bounds.width()), Length::Px(bounds.height())),
            ..Default::default()
        });

        self
    }
}

impl Widget for Filled {
	fn debug_label(&self) -> &'static str {
		"Filled"
	}

    fn measure(&self, _style: &taffy::Style, _known_dimensions: taffy::Size<Option<f32>>, _available_space: taffy::Size<taffy::AvailableSpace>) -> taffy::Size<f32> {
        let size = self.shape.bounds().size().map(|x| x as f32);
        size.into()
    }

    fn render(&mut self, ctx: &mut RenderContext) {
        ctx.fill(self.shape.offset(ctx.global_bounds().top_left()), self.color)
    }
}