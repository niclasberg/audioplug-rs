use crate::{
    core::{Circle, Color, Modifiers, Size, Vec2},
    event::{MouseDrag, MouseWheelEvent},
    param::{AnyParameter, NormalizedValue, PlainValue},
    ui::{
        BuildContext, Capabilities, Event, EventContext, EventResult, Prop, RenderContext, View,
        ViewStyle, WidgetHandler,
        reactive::{CanRead, ParamSetter, ReactiveValue, ReadContext},
    },
    views::util::ValueMapper,
};

type DragStartFn = dyn Fn(&mut EventContext);
type DragEndFn = dyn Fn(&mut EventContext);
type ValueChangedFn = dyn Fn(&mut EventContext, f64);

pub struct Knob {
    min: f64,
    max: f64,
    steps: usize,
    value: Option<Prop<f64>>,
    on_value_changed: Box<ValueChangedFn>,
}

impl Knob {
    pub fn new(f: impl Fn(&mut EventContext, f64) + 'static) -> Self {
        Self {
            min: 0.0,
            max: 1.0,
            steps: 0,
            value: None,
            on_value_changed: Box::new(f),
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

impl View for Knob {
    fn build(self, cx: &mut BuildContext) {
        let mapper = ValueMapper {
            min: self.min,
            max: self.max,
            steps: self.steps,
        };

        if let Some(value) = self.value {
            build_knob(
                cx,
                ViewStyle::default(),
                mapper,
                move |cx| value.get(cx),
                |_| {},
                self.on_value_changed,
                |_| {},
            );
        }
    }
}

fn build_knob(
    cx: &mut BuildContext,
    style: ViewStyle,
    mapper: ValueMapper,
    value: impl Fn(&mut ReadContext) -> f64 + 'static,
    on_drag_start: impl Fn(&mut EventContext) + 'static,
    on_value_changed: impl Fn(&mut EventContext, f64) + 'static,
    on_drag_end: impl Fn(&mut EventContext) + 'static,
) {
    const INTRINSIC_SIZE: Size = Size::splat(24.0);
    cx.intrinsic_size(INTRINSIC_SIZE);
    cx.style(style);
    cx.handler(
        Capabilities::FOCUSABLE | Capabilities::DRAGGABLE | Capabilities::MOUSE_WHEEL,
        KnobHandler {
            value,
            mapper,
            on_drag_start,
            on_drag_end,
            on_value_changed,
        },
    );
}

pub struct KnobHandler<FDragStart, FDragEnd, FOnChange, FValue> {
    value: FValue,
    mapper: ValueMapper,
    on_drag_start: FDragStart,
    on_drag_end: FDragEnd,
    on_value_changed: FOnChange,
}

impl<FDragStart, FDragEnd, FOnChange, FValue> WidgetHandler
    for KnobHandler<FDragStart, FDragEnd, FOnChange, FValue>
where
    FDragStart: Fn(&mut EventContext) + 'static,
    FDragEnd: Fn(&mut EventContext) + 'static,
    FOnChange: Fn(&mut EventContext, f64) + 'static,
    FValue: Fn(&mut ReadContext) -> f64 + 'static,
{
    fn event(&mut self, cx: &mut EventContext, event: &Event) -> EventResult {
        match event {
            Event::Drag(drag_event) => {
                match drag_event {
                    MouseDrag::Started { .. } => {
                        (self.on_drag_start)(cx);
                    }
                    MouseDrag::Moved {
                        modifiers, delta, ..
                    } => {
                        let delta_value = if modifiers.contains(Modifiers::SHIFT) {
                            delta.y * 0.001
                        } else {
                            delta.y * 0.01
                        };
                        let old_value = self.mapper.normalize((self.value)(&mut cx.read_context()));
                        let new_value = self
                            .mapper
                            .round((old_value - delta_value as f64).clamp(0.0, 1.0));

                        if new_value != old_value {
                            (self.on_value_changed)(cx, self.mapper.denormalize(new_value));
                        }
                    }
                    MouseDrag::Ended { .. } | MouseDrag::Cancelled { .. } => {
                        (self.on_drag_end)(cx);
                    }
                };
                EventResult::Stop
            }
            Event::Wheel(MouseWheelEvent { delta, .. }) => {
                let delta_y = delta.y as f64;
                let old_value = self.mapper.normalize((self.value)(&mut cx.read_context()));

                let new_value = self
                    .mapper
                    .round((old_value - 0.2 * delta_y).clamp(0.0, 1.0));
                if new_value != old_value {
                    (self.on_value_changed)(cx, self.mapper.denormalize(new_value));
                }
                EventResult::Stop
            }
            _ => EventResult::Continue,
        }
    }

    fn render(&mut self, cx: &mut RenderContext) {
        const MIN_ANGLE: f32 = -1.25 * std::f32::consts::PI;
        const MAX_ANGLE: f32 = 0.25 * std::f32::consts::PI;

        let bounds = cx.bounds();
        let shape = Circle::from_rect(bounds);
        let normalized_value = self.mapper.normalize((self.value)(&mut cx.read_context())) as f32;
        let angle = MIN_ANGLE * (1.0 - normalized_value) + MAX_ANGLE * normalized_value;

        let dot_pos = shape.center + Vec2::new(angle.cos(), angle.sin()).scale(0.7 * shape.radius);
        cx.fill(shape, Color::GREEN);
        cx.fill(Circle::new(dot_pos, 0.15 * shape.radius), Color::BLACK);
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
    fn build(self, cx: &mut BuildContext) {
        let editor = self.editor;
        let mapper = ValueMapper {
            min: editor.info(cx).min_value().0,
            max: editor.info(cx).max_value().0,
            steps: editor.info(cx).step_count(),
        };

        build_knob(
            cx,
            ViewStyle::default(),
            mapper,
            move |cx| self.signal.get(cx).0,
            move |cx| editor.begin_edit(cx),
            move |cx, value| editor.set_value_plain(cx, PlainValue(value)),
            move |cx| editor.end_edit(cx),
        );
    }
}
