use crate::{
    KeyEvent, MouseEvent,
    core::{Color, Key, Rect, Size, Zero},
    ui::{
        BuildContext, EventContext, EventStatus, RenderContext, Scene, StyleExt, View, ViewProp,
        ViewStyle, Widget,
        style::{AvailableSpace, LayoutMode, Length, Measure, Style, UiRect},
    },
};

type OnClickFn = dyn Fn(&mut EventContext);

pub struct Checkbox {
    style: ViewStyle,
    checked: Option<ViewProp<bool>>,
    enabled: ViewProp<bool>,
    click_fn: Option<Box<OnClickFn>>,
}

impl Checkbox {
    pub fn new() -> Self {
        Self {
            checked: None,
            enabled: ViewProp::Const(true),
            click_fn: None,
            style: ViewStyle {
                width: Some(ViewProp::Const(Length::Px(12.0))),
                height: Some(ViewProp::Const(Length::Px(12.0))),
                border: Some(ViewProp::Const(Length::Px(1.0))),
                border_color: Some(ViewProp::Const(Color::BLACK)),
                aspect_ratio: Some(ViewProp::Const(1.0)),
                corner_radius: Some(ViewProp::Const(Size::splat(3.0))),
                padding: Some(ViewProp::Const(UiRect::all_px(0.5))),
                ..ViewStyle::DEFAULT
            },
        }
    }

    pub fn checked(mut self, val: impl Into<ViewProp<bool>>) -> Self {
        self.checked = Some(val.into());
        self
    }

    pub fn enabled(mut self, val: impl Into<ViewProp<bool>>) -> Self {
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

    fn mouse_event(&mut self, event: MouseEvent, ctx: &mut EventContext) {
        if let MouseEvent::Click(_) = event {
            if let Some(f) = self.click_fn.as_mut() {
                f(ctx);
            }
        }
    }

    fn key_event(&mut self, event: KeyEvent, ctx: &mut EventContext) -> EventStatus {
        match event {
            KeyEvent::KeyDown {
                key: Key::Enter, ..
            } => {
                if let Some(f) = self.click_fn.as_mut() {
                    f(ctx);
                }
                EventStatus::Handled
            }
            _ => EventStatus::Ignored,
        }
    }

    fn render(&mut self, ctx: &mut RenderContext) -> Scene {
        let mut scene = Scene::new();
        if self.checked {
            let size = (ctx.content_bounds().size().min_element() - 1.0).max(0.0);
            let bounds = Rect::from_center(ctx.content_bounds().center(), Size::splat(size));
            scene.draw_lines(
                &[
                    bounds.get_relative_point(0., 0.5),
                    bounds.get_relative_point(0.35, 1.0),
                    bounds.get_relative_point(1.0, 0.0),
                ],
                Color::BLACK,
                2.0,
            )
        }
        scene
    }
}
