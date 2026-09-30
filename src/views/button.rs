use crate::{
    MouseEvent,
    core::{Color, Key, Paint},
    event::KeyEvent,
    ui::{
        BuildContext, EventContext, EventResult, StyleExt, View, Prop, ViewStyle, Widget,
        reactive::{ReactiveValue, WidgetStatus},
        style::UiRect,
    },
    views::Label,
};

type ClickFn = dyn FnMut(&mut EventContext);

fn default_style() -> ViewStyle {
    ViewStyle::default()
        .padding(UiRect::all_px(5.0))
        .background_fn(|cx| {
            let color = if WidgetStatus::PRESSED.get(cx) {
                Color::from_rgb8(121, 153, 141)
            } else {
                Color::from_rgb8(101, 133, 121)
            };
            Paint::Solid(color)
        })
}

pub struct Button<V> {
    style: ViewStyle,
    child: V,
    click_fn: Box<ClickFn>,
}

impl<V: View> Button<V> {
    pub fn new(child: V, click_fn: impl FnMut(&mut EventContext) + 'static) -> Self {
        Self {
            style: default_style(),
            child,
            click_fn: Box::new(click_fn),
        }
    }
}

impl Button<Label> {
    pub fn new_with_label(
        text: impl Into<Prop<String>>,
        click_fn: impl FnMut(&mut EventContext) + 'static,
    ) -> Self {
        Self {
            style: default_style(),
            child: Label::new(text),
            click_fn: Box::new(click_fn),
        }
    }
}

impl<V: View> View for Button<V> {
    type Element = ButtonWidget;

    fn build(self, cx: &mut BuildContext<Self::Element>) -> Self::Element {
        cx.set_focusable(true);
        cx.set_clickable(true);
        cx.add_child(self.child);
        cx.apply_style(self.style);

        ButtonWidget {
            click_fn: self.click_fn,
        }
    }
}

impl<V: View> StyleExt for Button<V> {
    fn style_mut(&mut self) -> &mut ViewStyle {
        &mut self.style
    }
}

pub struct ButtonWidget {
    click_fn: Box<ClickFn>,
}

impl Widget for ButtonWidget {
    fn debug_label(&self) -> &'static str {
        "Button"
    }

    fn mouse_event(&mut self, event: MouseEvent, ctx: &mut EventContext) {
        if let MouseEvent::Click(_) = event {
            (self.click_fn)(ctx);
        }
    }

    fn key_event(&mut self, event: KeyEvent, ctx: &mut EventContext) -> EventResult {
        match event {
            KeyEvent::KeyDown {
                key: Key::Enter, ..
            } => {
                (self.click_fn)(ctx);
                EventResult::Stop
            }
            _ => EventResult::Continue,
        }
    }
}
