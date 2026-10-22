use bitflags::bitflags;
use slotmap::SecondaryMap;

use crate::{
    KeyEvent,
    core::{Axis, Key, Modifiers, Point, Rect},
    event::{MouseDrag, MouseWheelEvent},
    ui::{Event, EventHandlers, EventResult, WidgetId},
};

bitflags! {
    #[derive(Default, Debug, Clone, Copy)]
    pub struct Sense : u32 {
        const FOCUSABLE = 1 << 0;
        const CLICKABLE = 1 << 2;
        const DRAGGABLE = 1 << 3;
        const HOVERABLE = 1 << 4;
        const MOUSE_WHEEL = 1 << 5;
    }
}

use super::EventContext;

pub type ClickFn = dyn FnMut(&mut EventContext);
pub type OnDragCallback = dyn Fn(&mut EventContext);
pub type OnValueChangeCallback = dyn Fn(&mut EventContext, f64);

#[derive(Default)]
pub struct Interactions {
    handlers: SecondaryMap<WidgetId, Interaction>,
}

impl Interactions {
    pub fn insert(&mut self, widget_id: WidgetId, interaction: Interaction) {
        self.handlers.insert(widget_id, interaction);
    }

    pub fn remove(&mut self, widget_id: WidgetId) {
        self.handlers.remove(widget_id);
    }

    pub fn has_sense(&self, widget_id: WidgetId, sense: Sense) -> bool {
        self.senses(widget_id).contains(sense)
    }

    pub fn senses(&self, widget_id: WidgetId) -> Sense {
        if let Some(handler) = self.handlers.get(widget_id) {
            match handler {
                Interaction::Events(e) => e.capabilities,
                Interaction::Button(_) => ButtonInteraction::CAPABILITIES,
                Interaction::Knob(_) => KnobInteraction::CAPABILITIES,
                Interaction::Slider(_) => SliderInteraction::CAPABILITIES,
            }
        } else {
            Sense::empty()
        }
    }

    pub fn dispatch(
        &mut self,
        cx: &mut EventContext,
        widget_id: WidgetId,
        event: &Event,
    ) -> EventResult {
        if let Some(handler) = self.handlers.get_mut(widget_id) {
            match handler {
                Interaction::Events(event_handlers) => {
                    for handler in event_handlers.handlers.iter_mut().rev() {
                        if handler(cx, event) == EventResult::Stop {
                            return EventResult::Stop;
                        }
                    }
                    EventResult::Continue
                }
                Interaction::Button(b) => b.event(cx, event),
                Interaction::Knob(k) => k.event(cx, event),
                Interaction::Slider(s) => s.event(cx, event),
            }
        } else {
            EventResult::Continue
        }
    }
}

pub enum Interaction {
    Events(EventHandlers),
    Button(ButtonInteraction),
    Knob(KnobInteraction),
    Slider(SliderInteraction),
}

pub struct ButtonInteraction {
    on_click: Box<ClickFn>,
}

impl ButtonInteraction {
    pub const CAPABILITIES: Sense = Sense::CLICKABLE.union(Sense::FOCUSABLE);

    pub fn new(f: impl FnMut(&mut EventContext) + 'static) -> Self {
        Self {
            on_click: Box::new(f),
        }
    }

    pub fn event(&mut self, cx: &mut EventContext, event: &Event) -> EventResult {
        match event {
            Event::Click(_) => {
                (self.on_click)(cx);
                EventResult::Stop
            }
            Event::Key(KeyEvent::KeyDown {
                key: Key::Enter, ..
            }) => {
                (self.on_click)(cx);
                EventResult::Stop
            }
            _ => EventResult::Continue,
        }
    }
}

pub enum KnobEvent {
    DragStart,
    DragEnd,
    ValueOffset { normalized_offset: f64 },
}

pub struct KnobInteraction {
    f: Box<dyn FnMut(&mut EventContext, KnobEvent)>,
}

impl KnobInteraction {
    pub const CAPABILITIES: Sense = Sense::DRAGGABLE.union(Sense::MOUSE_WHEEL);

    pub fn new(f: impl FnMut(&mut EventContext, KnobEvent) + 'static) -> Self {
        Self { f: Box::new(f) }
    }

    pub fn event(&mut self, cx: &mut EventContext, event: &Event) -> EventResult {
        match event {
            Event::Drag(drag_event) => {
                match drag_event {
                    MouseDrag::Started { .. } => {
                        (self.f)(cx, KnobEvent::DragStart);
                    }
                    MouseDrag::Moved {
                        modifiers, delta, ..
                    } => {
                        let delta_value = if modifiers.contains(Modifiers::SHIFT) {
                            delta.y * 0.001
                        } else {
                            delta.y * 0.01
                        };
                        (self.f)(
                            cx,
                            KnobEvent::ValueOffset {
                                normalized_offset: delta_value as f64,
                            },
                        );
                    }
                    MouseDrag::Ended { .. } | MouseDrag::Cancelled { .. } => {
                        (self.f)(cx, KnobEvent::DragEnd);
                    }
                };
                EventResult::Stop
            }
            Event::Wheel(MouseWheelEvent { delta, .. }) => {
                let delta_y = delta.y as f64;
                (self.f)(
                    cx,
                    KnobEvent::ValueOffset {
                        normalized_offset: -0.2 * delta_y,
                    },
                );
                EventResult::Stop
            }
            _ => EventResult::Continue,
        }
    }
}

pub enum SliderEvent {
    DragStart,
    DragEnd,
    ValueOffset { normalized_offset: f64 },
    ValueTo { normalized_value: f64 },
}

pub struct SliderInteraction {
    f: Box<dyn FnMut(&mut EventContext, SliderEvent)>,
    direction: Axis,
}

impl SliderInteraction {
    pub const CAPABILITIES: Sense = Sense::DRAGGABLE.union(Sense::FOCUSABLE);

    pub fn new(f: impl FnMut(&mut EventContext, SliderEvent) + 'static, direction: Axis) -> Self {
        Self {
            f: Box::new(f),
            direction,
        }
    }

    fn absolute_to_normalized_position(&self, bounds: Rect, position: Point) -> f64 {
        let normalized_position = match self.direction {
            Axis::Horizontal => {
                ((position.x - bounds.left - 2.5) / (bounds.width() - 5.0)).clamp(0.0, 1.0)
            }
            Axis::Vertical => {
                ((position.y - bounds.top - 2.5) / (bounds.height() - 5.0)).clamp(0.0, 1.0)
            }
        };
        normalized_position as f64
    }

    fn event(&mut self, cx: &mut EventContext, event: &Event) -> EventResult {
        match event {
            Event::Drag(drag_event) => match drag_event {
                MouseDrag::Started { position, .. } => {
                    (self.f)(cx, SliderEvent::DragStart);
                    let normalized_value =
                        self.absolute_to_normalized_position(cx.bounds(), *position);
                    (self.f)(cx, SliderEvent::ValueTo { normalized_value });

                    EventResult::Stop
                }
                MouseDrag::Moved { position, .. } => {
                    let normalized_value =
                        self.absolute_to_normalized_position(cx.bounds(), *position);
                    (self.f)(cx, SliderEvent::ValueTo { normalized_value });
                    EventResult::Stop
                }
                MouseDrag::Ended { .. } | MouseDrag::Cancelled { .. } => {
                    (self.f)(cx, SliderEvent::DragEnd);
                    EventResult::Stop
                }
            },
            Event::Key(key_event) => match key_event {
                KeyEvent::KeyDown { key, .. } => match key {
                    Key::Left | Key::Down => {
                        (self.f)(
                            cx,
                            SliderEvent::ValueOffset {
                                normalized_offset: -0.1,
                            },
                        );
                        EventResult::Stop
                    }
                    Key::Right | Key::Up => {
                        (self.f)(
                            cx,
                            SliderEvent::ValueOffset {
                                normalized_offset: 0.1,
                            },
                        );
                        EventResult::Stop
                    }
                    _ => EventResult::Continue,
                },
                _ => EventResult::Continue,
            },
            _ => EventResult::Continue,
        }
    }
}
