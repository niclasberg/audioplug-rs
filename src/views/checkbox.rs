use crate::{
    KeyEvent, MouseButton, MouseEvent,
    core::{Color, Key, Rect, Size, Zero},
    ui::{
        Accessor, BuildContext, CallbackContext, EventContext, EventStatus, MouseEventContext,
        RenderContext, Scene, View, Widget,
        style::{AvailableSpace, LayoutMode, Length, Measure, Style, UiRect},
    },
};

type OnClickFn = dyn Fn(&mut CallbackContext);

pub struct Checkbox {
    checked: Option<Accessor<bool>>,
    enabled: Accessor<bool>,
    click_fn: Option<Box<OnClickFn>>,
}

impl Checkbox {
    pub fn new() -> Self {
        Self {
            checked: None,
            enabled: Accessor::Const(true),
            click_fn: None,
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
        cx.set_focusable(true);
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

    fn on_click<F>(mut self, f: F) -> impl View
    where
        Self: Sized,
        F: Fn(&mut CallbackContext) + 'static,
    {
        self.click_fn = Some(Box::new(f));
        self
    }
}

#[derive(Default)]
pub struct CheckboxWidget {
    checked: bool,
    enabled: bool,
    click_fn: Option<Box<OnClickFn>>,
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

    fn mouse_event(&mut self, event: MouseEvent, ctx: &mut MouseEventContext) -> EventStatus {
        match event {
            MouseEvent::Down { button, .. } => {
                if button == MouseButton::LEFT {
                    ctx.capture_mouse();
                }
                EventStatus::Handled
            }
            MouseEvent::Up {
                button: MouseButton::LEFT,
                position,
                ..
            } => {
                if ctx.release_capture()
                    && ctx.bounds().contains(position)
                    && let Some(f) = self.click_fn.as_mut()
                {
                    f(&mut ctx.as_callback_context());
                }
                EventStatus::Handled
            }
            _ => EventStatus::Ignored,
        }
    }

    fn key_event(&mut self, event: KeyEvent, ctx: &mut EventContext) -> EventStatus {
        match event {
            KeyEvent::KeyDown {
                key: Key::Enter, ..
            } => {
                if let Some(f) = self.click_fn.as_mut() {
                    f(&mut ctx.as_callback_context());
                }
                EventStatus::Handled
            }
            _ => EventStatus::Ignored,
        }
    }

    fn layout_mode(&self) -> LayoutMode<'_> {
        LayoutMode::Leaf(self)
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
