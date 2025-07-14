use crate::{
    app::{
        Accessor, BuildContext, CallbackContext, EventContext, EventStatus, MouseEventContext,
        ParamSetter, RenderContext, StatusChange, View, Widget,
    },
    core::{Circle, Color, Modifiers, Point, Rectangle, Size},
    param::{AnyParameter, NormalizedValue, PlainValue},
    style::{AvailableSpace, DisplayStyle, Measure},
    MouseButton, MouseEvent,
};

use super::util::{denormalize_value, round_to_steps};

type DragStartFn = dyn Fn(&mut CallbackContext);
type DragEndFn = dyn Fn(&mut CallbackContext);
type ValueChangedFn = dyn Fn(&mut CallbackContext, f64);

pub struct Knob {
    min: f64,
    max: f64,
    value: Option<Accessor<f64>>,
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

    pub fn value(mut self, value: impl Into<Accessor<f64>>) -> Self {
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

    fn build(self, cx: &mut crate::app::BuildContext<Self::Element>) -> Self::Element {
        cx.set_focusable(true);
        KnobWidget {
            normalized_value: 0.0,
            last_mouse_pos: None,
            on_drag_start: self.on_drag_start,
            on_drag_end: self.on_drag_end,
            on_value_changed: self.on_value_changed,
            ..Default::default()
        }
    }
}

pub struct ParameterKnob<P> {
    editor: ParamSetter<P>,
    signal: Accessor<NormalizedValue>,
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
    last_mouse_pos: Option<Point>,
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
            last_mouse_pos: None,
            on_drag_start: None,
            on_drag_end: None,
            on_value_changed: None,
        }
    }
}

impl KnobWidget {
    fn shape(&self, bounds: Rectangle) -> Circle {
        let center = bounds.center();
        let radius = bounds.size().width.min(bounds.size().height) / 2.0;
        Circle::new(center, radius)
    }

    fn is_inside_knob(&self, bounds: Rectangle, point: Point) -> bool {
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

impl Measure for KnobWidget {
    fn measure(
        &self,
        _style: &crate::style::Style,
        width: AvailableSpace,
        height: AvailableSpace,
    ) -> Size {
        Size::new(width.unwrap_or(20.0), height.unwrap_or(20.0))
    }
}

impl Widget for KnobWidget {
    fn display_style(&self) -> DisplayStyle {
        DisplayStyle::Leaf(self)
    }

    fn debug_label(&self) -> &'static str {
        "Knob"
    }

    fn mouse_event(&mut self, event: MouseEvent, cx: &mut MouseEventContext) -> EventStatus {
        match event {
            MouseEvent::Down {
                button, position, ..
            } if button == MouseButton::LEFT && self.is_inside_knob(cx.bounds(), position) => {
                cx.capture_mouse();
                cx.request_render();
                self.last_mouse_pos = Some(position);
                if let Some(on_drag_start) = &self.on_drag_start {
                    on_drag_start(&mut cx.as_callback_context());
                }
                EventStatus::Handled
            }
            MouseEvent::Up { button, .. }
                if button == MouseButton::LEFT && cx.has_mouse_capture() =>
            {
                cx.release_capture();
                EventStatus::Handled
            }
            MouseEvent::Moved {
                position,
                modifiers,
            } => {
                if let Some(last_position) = self.last_mouse_pos {
                    let delta_y = position.y - last_position.y;
                    let delta_value = if modifiers.contains(Modifiers::SHIFT) {
                        delta_y * 0.001
                    } else {
                        delta_y * 0.01
                    };

                    let new_value = round_to_steps(
                        self.steps,
                        (self.normalized_value - delta_value).clamp(0.0, 1.0),
                    );
                    if new_value != self.normalized_value {
                        self.normalized_value = new_value;
                        cx.request_render();
                        if let Some(on_value_changed) = &self.on_value_changed {
                            on_value_changed(
                                &mut cx.as_callback_context(),
                                denormalize_value(self.min, self.max, new_value),
                            );
                        }
                        self.last_mouse_pos = Some(position);
                    }
                }

                EventStatus::Handled
            }
            MouseEvent::Wheel { delta, .. } => {
                let new_value = round_to_steps(
                    self.steps,
                    (self.normalized_value - 0.2 * delta.y).clamp(0.0, 1.0),
                );
                if new_value != self.normalized_value {
                    self.normalized_value = new_value;
                    cx.request_render();
                    if let Some(on_value_changed) = &self.on_value_changed {
                        on_value_changed(
                            &mut cx.as_callback_context(),
                            denormalize_value(self.min, self.max, new_value),
                        );
                    }
                }
                EventStatus::Handled
            }
            _ => EventStatus::Ignored,
        }
    }

    fn status_updated(&mut self, event: StatusChange, cx: &mut EventContext) {
        if event == StatusChange::MouseCaptureLost {
            self.last_mouse_pos = None;
            if let Some(on_drag_end) = &self.on_drag_end {
                on_drag_end(&mut cx.as_callback_context());
            }
        }
    }

    fn render(&mut self, cx: &mut RenderContext) {
        let bounds = cx.content_bounds();
        let shape = self.shape(bounds);

        let angle = self.current_angle();
        let dot_pos = shape.center + Point::new(angle.cos(), angle.sin()).scale(0.7 * shape.radius);
        cx.fill(shape, Color::GREEN);
        cx.fill(Circle::new(dot_pos, 0.15 * shape.radius), Color::BLACK);
    }
}
