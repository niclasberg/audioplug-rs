use slotmap::SecondaryMap;

use crate::{
    KeyEvent,
    event::{MouseClick, MouseDown, MouseDrag, MouseWheelEvent},
    ui::{EventContext, RenderContext, WidgetId, widget_tree::Capabilities},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EventResult {
    Stop,
    Continue,
}

impl EventResult {
    pub fn or_else(self, f: impl FnOnce() -> Self) -> Self {
        match self {
            Self::Stop => Self::Stop,
            Self::Continue => f(),
        }
    }
}

pub trait WidgetHandler: 'static {
    #[allow(unused)]
    fn event(&mut self, cx: &mut EventContext, event: &Event) -> EventResult {
        EventResult::Continue
    }
    #[allow(unused)]
    fn render(&mut self, cx: &mut RenderContext) {}
}

pub enum Event {
    Click(MouseClick),
    MouseDown(MouseDown),
    Drag(MouseDrag),
    Key(KeyEvent),
    Wheel(MouseWheelEvent),
}

type EventFn = dyn FnMut(&mut EventContext, &Event) -> EventResult;

#[derive(Default)]
pub struct EventHandler {
    event_fns: Vec<Box<EventFn>>,
    handler: Option<Box<dyn WidgetHandler>>,
}

#[derive(Default)]
pub struct WidgetHandlers {
    handlers: SecondaryMap<WidgetId, EventHandler>,
}

impl WidgetHandlers {
    pub fn set_widget_handler(
        &mut self,
        widget_id: WidgetId,
        widget_handler: Box<dyn WidgetHandler>,
    ) {
        self.handlers
            .entry(widget_id)
            .unwrap()
            .or_default()
            .handler
            .replace(widget_handler);
    }

    pub fn push_event_handler(&mut self, widget_id: WidgetId, handler: Box<EventFn>) {
        self.handlers
            .entry(widget_id)
            .unwrap()
            .or_default()
            .event_fns
            .push(handler);
    }

    pub fn remove(&mut self, widget_id: WidgetId) {
        self.handlers.remove(widget_id);
    }

    pub fn dispatch(
        &mut self,
        cx: &mut EventContext,
        widget_id: WidgetId,
        ev: &Event,
    ) -> EventResult {
        if let Some(handlers) = self.handlers.get_mut(widget_id) {
            for handler in handlers.event_fns.iter_mut().rev() {
                if handler(cx, ev) == EventResult::Stop {
                    return EventResult::Stop;
                }
            }
            if let Some(widget_handler) = handlers.handler.as_deref_mut() {
                return widget_handler.event(cx, ev);
            }
        }
        EventResult::Continue
    }

    pub fn render(&mut self, cx: &mut RenderContext, widget_id: WidgetId) {
        if let Some(widget_handler) = self
            .handlers
            .get_mut(widget_id)
            .and_then(|h| h.handler.as_deref_mut())
        {
            widget_handler.render(cx);
        }
    }
}

#[derive(Default)]
pub struct ViewEventHandlers {
    pub(super) handlers: Vec<Box<EventFn>>,
    pub(super) capabilities: Capabilities,
}

impl ViewEventHandlers {
    pub fn new() -> Self {
        Self::default()
    }
}

pub trait EventHandlerExt: Sized {
    fn event_handlers_mut(&mut self) -> &mut ViewEventHandlers;

    fn on_click(
        mut self,
        f: impl Fn(&mut EventContext, &MouseClick) -> EventResult + 'static,
    ) -> Self {
        let handlers = self.event_handlers_mut();
        handlers.capabilities.insert(Capabilities::CLICKABLE);
        handlers.handlers.push(Box::new(move |cx, ev| {
            let Event::Click(click_event) = &ev else {
                return EventResult::Continue;
            };
            f(cx, click_event)
        }));
        self
    }
}

impl EventHandlerExt for ViewEventHandlers {
    fn event_handlers_mut(&mut self) -> &mut ViewEventHandlers {
        self
    }
}
