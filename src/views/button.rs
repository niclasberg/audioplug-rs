use crate::{
    core::{Color, Key, Size},
    event::KeyEvent,
    ui::{
        BuildContext, Capabilities, Event, EventContext, EventHandlerExt, EventResult, Prop,
        StyleExt, TextExt, View, ViewEventHandlers, ViewStyle, ViewText,
        reactive::{Computed, ReactiveValue, WidgetStatus},
        style::{Edges, Fill},
    },
};

type ClickFn = dyn FnMut(&mut EventContext);

fn default_style() -> ViewStyle {
    ViewStyle::default()
        .padding(Edges::all_px(5.0))
        .corner_radius(Size::splat(5.0))
        .background(Computed::new_const(|cx| {
            let color = if WidgetStatus::PRESSED.get(cx) {
                Color::from_rgb8(121, 153, 141)
            } else {
                Color::from_rgb8(101, 133, 121)
            };
            Fill::Solid(color)
        }))
}

pub struct LabelButton {
    style: ViewStyle,
    text: ViewText,
    event_handlers: ViewEventHandlers,
}

impl LabelButton {
    pub fn new(
        label: impl Into<Prop<String>>,
        click_fn: impl Fn(&mut EventContext) + 'static,
    ) -> Self {
        let event_handlers = ViewEventHandlers::new().on_event(
            Capabilities::CLICKABLE | Capabilities::FOCUSABLE,
            move |cx, event| match event {
                Event::Click(_) => {
                    click_fn(cx);
                    EventResult::Stop
                }
                Event::Key(KeyEvent::KeyDown {
                    key: Key::Enter, ..
                }) => {
                    click_fn(cx);
                    EventResult::Stop
                }
                _ => EventResult::Continue,
            },
        );

        Self {
            style: default_style(),
            text: ViewText::new(label),
            event_handlers,
        }
    }
}

impl StyleExt for LabelButton {
    fn style_mut(&mut self) -> &mut ViewStyle {
        &mut self.style
    }
}

impl TextExt for LabelButton {
    fn text_mut(&mut self) -> &mut ViewText {
        &mut self.text
    }
}

impl EventHandlerExt for LabelButton {
    fn event_handlers_mut(&mut self) -> &mut ViewEventHandlers {
        &mut self.event_handlers
    }
}

impl View for LabelButton {
    fn build(self, cx: &mut BuildContext) {
        cx.text(self.text)
            .style(self.style)
            .event_handlers(self.event_handlers);
    }
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

impl<V: View> View for Button<V> {
    fn build(self, cx: &mut BuildContext) {
        cx.style(self.style);
        cx.add_child(self.child);
    }
}

impl<V: View> StyleExt for Button<V> {
    fn style_mut(&mut self) -> &mut ViewStyle {
        &mut self.style
    }
}
