use crate::{
    KeyEvent, MouseEvent,
    core::{Color, Key, Rect, Size},
    ui::{
        BuildContext, EventContext, EventResult, Prop, RenderContext, StyleExt, View, ViewStyle,
        Widget,
        style::{Border, Edges, Fill, Length},
    },
};

type OnClickFn = dyn Fn(&mut EventContext);

pub struct Checkbox {
    style: ViewStyle,
    checked: Option<Prop<bool>>,
    enabled: Prop<bool>,
    click_fn: Option<Box<OnClickFn>>,
}

impl Checkbox {
    pub fn new() -> Self {
        Self {
            checked: None,
            enabled: Prop::Const(true),
            click_fn: None,
            style: ViewStyle {
                width: Some(Prop::Const(Length::Px(12.0))),
                height: Some(Prop::Const(Length::Px(12.0))),
                border: Some(Prop::Const(Border {
                    fill: Fill::Solid(Color::BLACK),
                    width: Length::Px(1.0),
                })),
                aspect_ratio: Some(Prop::Const(1.0)),
                corner_radius: Some(Prop::Const(Size::splat(3.0))),
                padding: Some(Prop::Const(Edges::all_px(0.5))),
                ..ViewStyle::DEFAULT
            },
        }
    }

    pub fn checked(mut self, val: impl Into<Prop<bool>>) -> Self {
        self.checked = Some(val.into());
        self
    }

    pub fn enabled(mut self, val: impl Into<Prop<bool>>) -> Self {
        self.enabled = val.into();
        self
    }

    pub fn on_click(mut self, f: impl Fn(&mut EventContext) + 'static) -> Self {
        self.click_fn.replace(Box::new(f));
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
        cx.set_focusable(true);
        cx.set_clickable(true);
        cx.apply_style(self.style);
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
            click_fn: self.click_fn,
        }
    }
}

impl StyleExt for Checkbox {
    fn style_mut(&mut self) -> &mut ViewStyle {
        &mut self.style
    }
}

#[derive(Default)]
pub struct CheckboxWidget {
    checked: bool,
    enabled: bool,
    click_fn: Option<Box<OnClickFn>>,
}

impl Widget for CheckboxWidget {
    fn debug_label(&self) -> &'static str {
        "Checkbox"
    }

    fn mouse_event(&mut self, event: MouseEvent, cx: &mut EventContext) {
        if let MouseEvent::Click(_) = event {
            self.checked = !self.checked;
            if let Some(f) = self.click_fn.as_mut() {
                f(cx);
            }
            cx.request_render();
        }
    }

    fn key_event(&mut self, event: KeyEvent, ctx: &mut EventContext) -> EventResult {
        match event {
            KeyEvent::KeyDown {
                key: Key::Enter, ..
            } => {
                if let Some(f) = self.click_fn.as_mut() {
                    f(ctx);
                }
                EventResult::Stop
            }
            _ => EventResult::Continue,
        }
    }

    fn render(&mut self, cx: &mut RenderContext) {
        if self.checked {
            let size = (cx.bounds().size().min_element() - 1.0).max(0.0);
            let bounds = Rect::from_center(cx.bounds().center(), Size::splat(size));
            cx.stroke_path(
                |path| {
                    path.move_to(bounds.get_relative_point(0.05, 0.5))
                        .line_to(bounds.get_relative_point(0.35, 0.95))
                        .quad_to(
                            bounds.get_relative_point(0.5, 0.5),
                            bounds.get_relative_point(0.95, 0.05),
                        );
                },
                Color::BLACK,
                2.0,
            )
        }
    }
}
