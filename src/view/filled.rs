use crate::core::{Color, Rectangle, Shape, Size};
use super::{BuildContext, EventContext, LayoutContext, RenderContext, View, Widget};


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

    fn build(self, _ctx: &mut BuildContext) -> Self { 
        self
    }
}

impl Widget for Filled {
    fn event(&mut self, _event: crate::Event, _ctx: &mut EventContext) {
        
    }

    fn layout(&mut self, _inputs: taffy::LayoutInput, _ctx: &mut LayoutContext) -> taffy::LayoutOutput {
        let size = self.shape.bounds().size().map(|x| x as f32);
        taffy::LayoutOutput::from_outer_size(size.into())
    }

    fn render(&mut self, ctx: &mut RenderContext) {
        ctx.fill(self.shape.offset(ctx.global_bounds().top_left()), self.color)
    }
    
    fn style(&self) -> taffy::Style {
        let bounds = self.shape.bounds();
        taffy::Style {
            size: taffy::Size { width: taffy::Dimension::Length(bounds.width() as f32), height: taffy::Dimension::Length(bounds.height() as f32) },
            ..Default::default()
        }
    }
}