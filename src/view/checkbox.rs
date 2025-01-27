use crate::{app::{Accessor, BuildContext, EventStatus, MouseEventContext, RenderContext, Widget, WidgetMut}, core::{Color, Shape, Size}, style::{DisplayStyle, Length, Measure, Style}, MouseEvent};

use super::View;

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

    fn build(self, ctx: &mut BuildContext<Self::Element>) -> Self::Element {
        let checked = self.checked.get_and_track(ctx, |value, mut widget| {
            (*widget).checked = value;
			widget.request_render();
        });
		ctx.set_style(Style {
            size: Size::new(Length::Px(20.0), Length::Px(10.0)),
            ..Default::default()
        });
        CheckboxWidget { checked }
    }
}

pub struct CheckboxWidget {
    checked: bool,
}

impl Measure for CheckboxWidget {
    fn measure(&self, _: &Style, width: Option<f64>, height: Option<f64>, _: taffy::AvailableSpace, _: taffy::AvailableSpace) -> Size<f64>  {
        if let (Some(width), Some(height)) = (width, height) {
            Size::new(width, height)
        } else {
            Size::ZERO
        }
    }
}

impl Widget for CheckboxWidget {
	fn debug_label(&self) -> &'static str {
		"Checkbox"
	}

	fn mouse_event(&mut self, event: MouseEvent, ctx: &mut MouseEventContext) -> EventStatus {
		EventStatus::Handled
	}

    fn display_style(&self) -> DisplayStyle {
        DisplayStyle::Leaf(self)
    }

    fn render(&mut self, ctx: &mut RenderContext) {
        let color = if self.checked { Color::RED } else { Color::from_rgb(0.1, 0.1, 0.1) };
        ctx.fill(Shape::circle(ctx.global_bounds().center(), 5.0), color)
    }
}