use crate::{app::{Accessor, BuildContext, EventStatus, MouseEventContext, RenderContext, Widget, WidgetMut}, core::{Color, Shape}, MouseEvent};

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
        let checked = ctx.get_and_track(self.checked, |value, mut widget| {
            (*widget).checked = *value;
			widget.request_render();
        });
		ctx.set_style(taffy::Style {
            size: taffy::Size { width: taffy::Dimension::Length(20.0), height: taffy::Dimension::Length(10.0) },
            ..Default::default()
        });
        CheckboxWidget { checked }
    }
}


pub struct CheckboxWidget {
    checked: bool,
}

impl Widget for CheckboxWidget {
	fn debug_label(&self) -> &'static str {
		"Checkbox"
	}

	fn mouse_event(&mut self, event: MouseEvent, ctx: &mut MouseEventContext) -> EventStatus {
		
		EventStatus::Handled
	}

    fn measure(&self, _style: &taffy::Style, known_dimensions: taffy::Size<Option<f32>>, _available_space: taffy::Size<taffy::AvailableSpace>) -> taffy::Size<f32> {
        if let taffy::Size { width: Some(width), height: Some(height) } = known_dimensions {
            taffy::Size { width, height }
        } else {
            taffy::Size::zero()
        }
    }

    fn render(&mut self, ctx: &mut RenderContext) {
        let color = if self.checked { Color::RED } else { Color::from_rgb(0.1, 0.1, 0.1) };
        ctx.fill(Shape::circle(ctx.global_bounds().center(), 5.0), color)
    }
}