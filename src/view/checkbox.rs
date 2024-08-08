use crate::{app::{Accessor, BuildContext, LayoutContext, RenderContext, SignalGet, WidgetMut}, core::{Color, Shape}};

use super::{View, Widget};

pub struct Checkbox {
    checked: Accessor<bool>
}

impl Checkbox {
    pub fn new(checked: impl Into<Accessor<bool>>) -> Self {
        Self {
            checked: checked.into()
        }
    }
}

impl View for Checkbox {
    type Element = CheckboxWidget;

    fn build(self, ctx: &mut BuildContext) -> Self::Element {
        let checked = ctx.get_and_track(self.checked, |value, mut widget: WidgetMut<'_, Self::Element>| {
            (*widget).checked = *value;
        });
        CheckboxWidget { checked }
    }
}


pub struct CheckboxWidget {
    checked: bool,
}

impl Widget for CheckboxWidget {
    fn measure(&self, _style: &taffy::Style, known_dimensions: taffy::Size<Option<f32>>, available_space: taffy::Size<taffy::AvailableSpace>) -> taffy::Size<f32> {
        if let taffy::Size { width: Some(width), height: Some(height) } = known_dimensions {
            taffy::Size { width, height }
        } else {
            taffy::Size::zero()
        }
    }

    fn style(&self) -> taffy::Style {
        taffy::Style {
            size: taffy::Size { width: taffy::Dimension::Length(10.0), height: taffy::Dimension::Length(10.0) },
            ..Default::default()
        }
        
    }

    fn render(&mut self, ctx: &mut RenderContext) {
        let color = if self.checked { Color::RED } else { Color::from_rgb(0.1, 0.1, 0.1) };
        ctx.fill(Shape::circle(ctx.global_bounds().center(), 5.0), color)
    }
}