use crate::{
    core::{Color, Rect, Shape, Size},
    ui::{
        BuildContext, RenderContext, Scene, View, Widget,
        style::{AvailableSpace, LayoutMode, Length, Measure, Style},
    },
};

pub trait Fill {
    fn fill(self, color: Color) -> Filled;
}

impl Fill for Shape {
    fn fill(self, color: Color) -> Filled {
        Filled { shape: self, color }
    }
}

impl Fill for Rect {
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
        ctx.set_default_style(Style {
            size: Size::new(Length::Px(bounds.width()), Length::Px(bounds.height())),
            ..Default::default()
        });

        self
    }
}

impl Measure for Filled {
    fn measure(&self, _style: &Style, _width: AvailableSpace, _height: AvailableSpace) -> Size {
        self.shape.bounds().size()
    }
}

impl Widget for Filled {
    fn debug_label(&self) -> &'static str {
        "Filled"
    }

    fn render(&mut self, ctx: &mut RenderContext) -> Scene {
        let mut scene = Scene::new();
        scene.fill(
            &self.shape.offset(ctx.global_bounds().top_left()),
            self.color,
        );
        scene
    }

    fn layout_mode(&self) -> LayoutMode {
        LayoutMode::Leaf(self)
    }
}
