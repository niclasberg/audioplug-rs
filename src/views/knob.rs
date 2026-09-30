use crate::{
    MouseButton, MouseEvent,
    core::{Circle, Color, Modifiers, Point, Rect, Vec2},
    event::{MouseDownEvent, MouseDragEvent, MouseWheelEvent},
    param::{AnyParameter, NormalizedValue, PlainValue},
    ui::{BuildContext, EventContext, Prop, RenderContext, View, Widget, reactive::ParamSetter},
};

use super::util::{denormalize_value, round_to_steps};

type DragStartFn = dyn Fn(&mut EventContext);
type DragEndFn = dyn Fn(&mut EventContext);
type ValueChangedFn = dyn Fn(&mut EventContext, f64);

pub struct Knob {
    min: f64,
    max: f64,
    value: Option<Prop<f64>>,
    on_drag_start: Option<Box<DragStartFn>>,
    on_drag_end: Option<Box<DragEndFn>>,
    on_value_changed: Option<Box<ValueChangedFn>>,
}

impl Knob {
    pub fn new() -> Self {
        Self {
            min: 0.0,
            max: 1.0,
            value: None,
            on_drag_start: None,
            on_drag_end: None,
            on_value_changed: None,
        }
    }

    pub fn range(mut self, min: f64, max: f64) -> Self {
        self.min = min;
        self.max = max;
        self
    }

    pub fn value(mut self, value: impl Into<Prop<f64>>) -> Self {
        self.value = Some(value.into());
        self
    }
}

impl Default for Knob {
    fn default() -> Self {
        Self::new()
    }
}

impl View for Knob {
    type Element = KnobWidget;

    fn build(self, cx: &mut crate::ui::BuildContext<Self::Element>) -> Self::Element {
        cx.set_focusable(true);
        KnobWidget {
            normalized_value: 0.0,
            on_drag_start: self.on_drag_start,
            on_drag_end: self.on_drag_end,
            on_value_changed: self.on_value_changed,
            ..Default::default()
        }
    }
}

pub struct ParameterKnob<P> {
    editor: ParamSetter<P>,
    signal: Prop<NormalizedValue>,
}

impl<P: AnyParameter> ParameterKnob<P> {
    pub fn new(parameter: &P) -> Self {
        Self {
            signal: parameter.as_signal_normalized().into(),
            editor: ParamSetter::new(parameter),
        }
    }
}

impl<P: AnyParameter> View for ParameterKnob<P> {
    type Element = KnobWidget;

    fn build(self, cx: &mut BuildContext<Self::Element>) -> Self::Element {
        let editor = self.editor;
        KnobWidget {
            min: editor.info(cx).min_value().into(),
            max: editor.info(cx).max_value().into(),
            steps: editor.info(cx).step_count(),
            normalized_value: self.signal.get_and_bind_mapped(
                cx,
                |value| value.0,
                move |value, mut widget| {
                    widget.normalized_value = value;
                    widget.request_render();
                },
            ),
            on_drag_start: Some(Box::new(move |cx| editor.begin_edit(cx))),
            on_drag_end: Some(Box::new(move |cx| editor.end_edit(cx))),
            on_value_changed: Some(Box::new(move |cx, value| {
                editor.set_value_plain(cx, PlainValue(value))
            })),
            ..Default::default()
        }
    }
}

pub struct KnobWidget {
    min: f64,
    max: f64,
    steps: usize,
    normalized_value: f64,
    on_drag_start: Option<Box<DragStartFn>>,
    on_drag_end: Option<Box<DragEndFn>>,
    on_value_changed: Option<Box<ValueChangedFn>>,
}

impl Default for KnobWidget {
    fn default() -> Self {
        Self {
            min: 0.0,
            max: 1.0,
            steps: 0,
            normalized_value: 0.0,
            on_drag_start: None,
            on_drag_end: None,
            on_value_changed: None,
        }
    }
}

impl KnobWidget {
    fn shape(&self, bounds: Rect) -> Circle {
        let center = bounds.center();
        let radius = bounds.size().width.min(bounds.size().height) / 2.0;
        Circle::new(center, radius)
    }

    fn is_inside_knob(&self, bounds: Rect, point: Point) -> bool {
        self.shape(bounds).contains(point)
    }

    fn min_angle(&self) -> f64 {
        -1.25 * std::f64::consts::PI
    }

    fn max_angle(&self) -> f64 {
        0.25 * std::f64::consts::PI
    }

    fn current_angle(&self) -> f64 {
        self.min_angle() * (1.0 - self.normalized_value) + self.max_angle() * self.normalized_value
    }
}

impl Widget for KnobWidget {
    fn debug_label(&self) -> &'static str {
        "Knob"
    }

    fn mouse_event(&mut self, event: MouseEvent, cx: &mut EventContext) {
        match event {
            MouseEvent::Down(MouseDownEvent { button, .. }) if button == MouseButton::LEFT => {
                cx.request_render();
                if let Some(on_drag_start) = &self.on_drag_start {
                    on_drag_start(cx);
                }
            }
            MouseEvent::DragEnded | MouseEvent::DragCancelled => {
                if let Some(on_drag_end) = &self.on_drag_end {
                    on_drag_end(cx);
                }
            }
            MouseEvent::DragMoved(MouseDragEvent {
                delta, modifiers, ..
            }) => {
                let delta_value = if modifiers.contains(Modifiers::SHIFT) {
                    delta.y * 0.001
                } else {
                    delta.y * 0.01
                };

                let new_value = round_to_steps(
                    self.steps,
                    (self.normalized_value - delta_value as f64).clamp(0.0, 1.0),
                );
                if new_value != self.normalized_value {
                    self.normalized_value = new_value;
                    cx.request_render();
                    if let Some(on_value_changed) = &self.on_value_changed {
                        on_value_changed(cx, denormalize_value(self.min, self.max, new_value));
                    }
                }
            }
            MouseEvent::Wheel(MouseWheelEvent { delta, .. }) => {
                let delta_y = delta.y as f64;
                let new_value = round_to_steps(
                    self.steps,
                    (self.normalized_value - 0.2 * delta_y).clamp(0.0, 1.0),
                );
                if new_value != self.normalized_value {
                    self.normalized_value = new_value;
                    cx.request_render();
                    if let Some(on_value_changed) = &self.on_value_changed {
                        on_value_changed(cx, denormalize_value(self.min, self.max, new_value));
                    }
                }
            }
            _ => {}
        }
    }

    fn render(&mut self, cx: &mut RenderContext) {
        let bounds = cx.bounds();
        let shape = self.shape(bounds);

        let angle = self.current_angle() as f32;
        let dot_pos = shape.center + Vec2::new(angle.cos(), angle.sin()).scale(0.7 * shape.radius);
        cx.fill(shape, Color::GREEN);
        cx.fill(Circle::new(dot_pos, 0.15 * shape.radius), Color::BLACK);
    }
}
