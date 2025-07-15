use crate::{
    app::{
        Accessor, BuildContext, CallbackContext, EventContext, EventStatus, MouseEventContext,
        RenderContext, StatusChange, View, Widget,
    },
    core::{Color, Key},
    event::{KeyEvent, MouseButton},
    style::{DisplayStyle, FlexStyle, Length, Style, UiRect},
    views::Label,
    MouseEvent,
};

type ClickFn = dyn FnMut(&mut CallbackContext);

pub struct Button<V> {
    child: V,
    click_fn: Option<Box<ClickFn>>,
}

impl<V: View> Button<V> {
    pub fn new(child: V) -> Self {
        Self {
            child,
            click_fn: None,
        }
    }
}

impl Button<Label> {
    pub fn new_with_label(text: impl Into<Accessor<String>>) -> Self {
        Self {
            child: Label::new(text),
            click_fn: None,
        }
    }
}

impl<V: View> View for Button<V> {
    type Element = ButtonWidget;

    fn build(self, ctx: &mut BuildContext<Self::Element>) -> Self::Element {
        ctx.set_focusable(true);
        ctx.add_child(self.child);
        ctx.set_default_style(Style {
            background: Some(Color::from_rgb8(121, 153, 141).into()),
            padding: UiRect::all(Length::Px(4.0)),
            ..Default::default()
        });

        ButtonWidget {
            is_hot: false,
            mouse_down: false,
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

const FLEX_STYLE: FlexStyle = FlexStyle::DEFAULT;

pub struct ButtonWidget {
    is_hot: bool,
    mouse_down: bool,
    click_fn: Option<Box<ClickFn>>,
}

impl Widget for ButtonWidget {
    fn debug_label(&self) -> &'static str {
        "Button"
    }

    fn mouse_event(&mut self, event: MouseEvent, ctx: &mut MouseEventContext) -> EventStatus {
        match event {
            MouseEvent::Down { button, .. } => {
                if button == MouseButton::LEFT {
                    ctx.capture_mouse();
                    ctx.request_render();
                }
                EventStatus::Handled
            }
            MouseEvent::Up {
                button: MouseButton::LEFT,
                position,
                ..
            } => {
                if ctx.release_capture() && ctx.bounds().contains(position) {
                    if let Some(f) = self.click_fn.as_mut() {
                        f(&mut ctx.as_callback_context());
                    }
                }
                ctx.request_render();
                EventStatus::Handled
            }
            _ => EventStatus::Ignored,
        }
    }

    fn status_updated(&mut self, event: StatusChange, ctx: &mut EventContext) {
        match event {
            StatusChange::FocusGained => {
                self.is_hot = true;
                ctx.request_render();
            }
            StatusChange::FocusLost => {
                self.is_hot = false;
                ctx.request_render();
            }
            StatusChange::MouseCaptureLost => {
                if self.mouse_down {
                    self.mouse_down = false;
                    ctx.request_render();
                }
            }
            _ => {}
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

    fn render(&mut self, ctx: &mut RenderContext) {
        ctx.render_children()
    }

    fn display_style(&self) -> DisplayStyle {
        DisplayStyle::Flex(&FLEX_STYLE)
    }
}
