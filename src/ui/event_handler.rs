use crate::{
    KeyEvent,
    event::{MouseClick, MouseDown, MouseDrag, MouseMove, MouseUp, MouseWheelEvent},
    ui::{Sense, EventContext},
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

pub enum Event {
    Click(MouseClick),
    MouseDown(MouseDown),
    MouseUp(MouseUp),
    MouseMove(MouseMove),
    Drag(MouseDrag),
    Key(KeyEvent),
    Wheel(MouseWheelEvent),
}

type EventFn = dyn FnMut(&mut EventContext, &Event) -> EventResult;

#[derive(Default)]
pub struct EventHandlers {
    pub(super) handlers: Vec<Box<EventFn>>,
    pub(super) capabilities: Sense,
}

impl EventHandlers {
    pub fn new() -> Self {
        Self::default()
    }
}

pub trait EventHandlerExt: Sized {
    fn event_handlers_mut(&mut self) -> &mut EventHandlers;

    fn on_click(
        mut self,
        f: impl Fn(&mut EventContext, &MouseClick) -> EventResult + 'static,
    ) -> Self {
        let handlers = self.event_handlers_mut();
        handlers.capabilities.insert(Sense::CLICKABLE);
        handlers.handlers.push(Box::new(move |cx, ev| {
            let Event::Click(click_event) = &ev else {
                return EventResult::Continue;
            };
            f(cx, click_event)
        }));
        self
    }

    fn on_key_event(
        mut self,
        f: impl Fn(&mut EventContext, &KeyEvent) -> EventResult + 'static,
    ) -> Self {
        let handlers = self.event_handlers_mut();
        handlers.capabilities.insert(Sense::FOCUSABLE);
        handlers.handlers.push(Box::new(move |cx, ev| {
            let Event::Key(key_event) = &ev else {
                return EventResult::Continue;
            };
            f(cx, key_event)
        }));
        self
    }

    fn on_event(
        mut self,
        capabilities: Sense,
        f: impl Fn(&mut EventContext, &Event) -> EventResult + 'static,
    ) -> Self {
        let handlers = self.event_handlers_mut();
        handlers.capabilities.insert(capabilities);
        handlers.handlers.push(Box::new(f));
        self
    }
}

impl EventHandlerExt for EventHandlers {
    fn event_handlers_mut(&mut self) -> &mut EventHandlers {
        self
    }
}
