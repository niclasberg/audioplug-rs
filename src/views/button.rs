use crate::{
    MouseEvent,
    core::{Color, Key},
    event::{KeyEvent, MouseButton},
    ui::{
        BuildContext, CallbackContext, EventContext, EventStatus, MouseEventContext, StatusChange,
        View, ViewProp, Widget,
        reactive::ReactiveValue,
        style::{FlexStyle, LayoutMode, Length, UiRect},
    },
    views::Label,
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
    pub fn new_with_label(text: impl Into<ViewProp<String>>) -> Self {
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
        ctx.apply_style(|s, signals| {
            s.background(signals.clicked.map(move |&clicked| {
                println!("Clicked: {clicked}");
                if clicked {
                    Color::from_rgb8(121, 153, 141)
                } else {
                    Color::from_rgb8(101, 133, 121)
                }
                .into()
            }))
            .padding(UiRect::all(Length::Px(4.0)));
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
                if ctx.release_capture()
                    && ctx.bounds().contains(position)
                    && let Some(f) = self.click_fn.as_mut()
                {
                    f(&mut ctx.as_callback_context());
                }
                ctx.request_render();
                EventStatus::Handled
            }
            _ => EventStatus::Ignored,
        }
    }

    fn status_change(&mut self, event: StatusChange, ctx: &mut EventContext) {
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

    fn layout_mode(&self) -> LayoutMode<'_> {
        LayoutMode::Flex(&FLEX_STYLE)
    }
}
