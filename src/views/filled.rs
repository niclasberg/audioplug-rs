use crate::{
    app::{BuildContext, RenderContext, Shape, View, Widget},
    core::{Color, Rectangle, Size},
    style::{AvailableSpace, DisplayStyle, Length, Measure, Style},
};

pub trait Fill {
    fn fill(self, color: Color) -> Filled;
}

impl Fill for Shape {
    fn fill(self, color: Color) -> Filled {
        Filled { shape: self, color }
    }
}

impl Fill for Rectangle {
    fn fill(self, color: Color) -> Filled {
        Filled {
            shape: self.into(),
            color,
        }
    }
}

pub struct Filled {
    shape: Shape,
    color: Color,
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

impl Measure for Filled {
    fn measure(&self, style: &Style, _width: AvailableSpace, _height: AvailableSpace) -> Size {
        self.shape.bounds().size()
    }
}

impl Widget for Filled {
    fn debug_label(&self) -> &'static str {
        "Filled"
    }

    fn render(&mut self, ctx: &mut RenderContext) {
        ctx.fill(
            &self.shape.offset(ctx.global_bounds().top_left()),
            self.color,
        )
    }

    fn display_style(&self) -> DisplayStyle {
        DisplayStyle::Leaf(self)
    }
}
