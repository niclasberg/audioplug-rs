use crate::{app::{Accessor, RenderContext, View, Widget}, core::{Color, Ellipse, Rectangle, Size}, style::{DisplayStyle, Measure}};

pub struct Knob {
    value: Option<Accessor<f64>>
}

impl Knob {
    pub fn new() -> Self {
        Self { 
            value: None
        }
    }
}

impl View for Knob {
    type Element = KnobWidget;

    fn build(self, ctx: &mut crate::app::BuildContext<Self::Element>) -> Self::Element {
        KnobWidget {
            normalized_value: 0.0,
        }
    }
}

pub struct KnobWidget {
    normalized_value: f64,
}

impl Measure for KnobWidget {
    fn measure(
        &self,
        _style: &crate::style::Style,
        width: Option<f64>,
        height: Option<f64>,
        _available_width: taffy::AvailableSpace,
        _available_height: taffy::AvailableSpace,
    ) -> Size {
        match (width, height) {
            (Some(width), Some(height)) => Size::new(width, height),
            (Some(len), None) | (None, Some(len)) => Size::new(len, len),
            (None, None) => Size::new(20.0, 20.0)
        }
    }
}

impl Widget for KnobWidget {
    fn display_style(&self) -> DisplayStyle {
        DisplayStyle::Leaf(self)
    }

    fn debug_label(&self) -> &'static str {
        "Knob"
    }

    fn render(&mut self, cx: &mut RenderContext) {
        let bounds = cx.content_bounds();
        let center = bounds.center();
        let radius = bounds.size().width.min(bounds.size().height) / 2.0;
        cx.fill(Ellipse::new(center, Size::new(radius, radius)), Color::GREEN);
        
    }
}