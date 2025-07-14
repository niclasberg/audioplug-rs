use crate::{
    app::{Accessor, BuildContext, EventStatus, MouseEventContext, RenderContext, View, Widget},
    core::{Color, Rectangle, Size},
    style::{AvailableSpace, DisplayStyle, Length, Measure, Style, UiRect},
    MouseEvent,
};

pub struct Checkbox {
    checked: Option<Accessor<bool>>,
    enabled: Accessor<bool>,
}

impl Checkbox {
    pub fn new() -> Self {
        Self {
            checked: None,
            enabled: Accessor::Const(true),
        }
    }

    pub fn checked(mut self, val: impl Into<Accessor<bool>>) -> Self {
        self.checked = Some(val.into());
        self
    }

    pub fn enabled(mut self, val: impl Into<Accessor<bool>>) -> Self {
        self.enabled = val.into();
        self
    }
}

impl Default for Checkbox {
    fn default() -> Self {
        Self::new()
    }
}

impl View for Checkbox {
    type Element = CheckboxWidget;

    fn build(self, cx: &mut BuildContext<Self::Element>) -> Self::Element {
        cx.set_default_style(Style {
            size: Size::new(Length::Px(12.0), Length::Px(12.0)),
            border: Length::Px(1.0),
            border_color: Some(Color::BLACK),
            aspect_ratio: Some(1.0),
            corner_radius: Size::splat(3.0),
            padding: UiRect::all_px(0.5),
            ..Default::default()
        });
        CheckboxWidget {
            checked: self
                .checked
                .map(|checked| {
                    checked.get_and_bind(cx, |value, mut widget| {
                        widget.checked = value;
                        widget.request_render();
                    })
                })
                .unwrap_or_default(),
            enabled: self.enabled.get_and_bind(cx, |value, mut widget| {
                widget.enabled = value;
                widget.request_render();
            }),
        }
    }
}

#[derive(Default)]
pub struct CheckboxWidget {
    checked: bool,
    enabled: bool,
}

impl Measure for CheckboxWidget {
    fn measure(&self, _: &Style, width: AvailableSpace, height: AvailableSpace) -> Size<f64> {
        if let (Some(width), Some(height)) = (width.into(), height.into()) {
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

    fn mouse_event(&mut self, _event: MouseEvent, _cx: &mut MouseEventContext) -> EventStatus {
        EventStatus::Handled
    }

    fn display_style(&self) -> DisplayStyle {
        DisplayStyle::Leaf(self)
    }

    fn render(&mut self, ctx: &mut RenderContext) {
        if self.checked {
            let size = (ctx.content_bounds().size().min_element() - 1.0).max(0.0);
            let bounds = Rectangle::from_center(ctx.content_bounds().center(), Size::splat(size));
            ctx.draw_lines(
                &[
                    bounds.get_relative_point(0., 0.5),
                    bounds.get_relative_point(0.35, 1.0),
                    bounds.get_relative_point(1.0, 0.0),
                ],
                Color::BLACK,
                2.0,
            )
        }
    }
}
