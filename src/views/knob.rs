use crate::{
    core::{Circle, Color, Size, Vec2},
    param::{AnyParameter, NormalizedValue},
    ui::{
        BuildContext, Sense, EventContext, KnobEvent, Prop, RenderContext, View,
        reactive::{ParamSetter, ReactiveValue, ReadSignal, Var},
    },
    views::util::ValueMapper,
};

const INTRINSIC_SIZE: Size = Size::splat(24.0);
const CAPABILITIES: Sense = Sense::DRAGGABLE
    .union(Sense::FOCUSABLE)
    .union(Sense::MOUSE_WHEEL);

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
            {
                let value = value.clone();
                cx.knob(move |cx, event| match event {
                    KnobEvent::DragStart => {}
                    KnobEvent::DragEnd => {}
                    KnobEvent::ValueOffset { normalized_offset } => {
                        let old_value = mapper.normalize(value.get_untracked(cx));
                        let new_value = (old_value + normalized_offset).clamp(0.0, 1.0);
                        if old_value != new_value {
                            (self.on_value_changed)(cx, mapper.denormalize(new_value));
                        }
                    }
                });
            }
            build_knob(cx, move |cx| mapper.normalize(value.get(cx)));
        } else {
            let normalized_value = Var::new(cx, 0.0);
            cx.knob(move |cx, event| match event {
                KnobEvent::DragStart => {}
                KnobEvent::DragEnd => {}
                KnobEvent::ValueOffset { normalized_offset } => {
                    let old_value = normalized_value.get_untracked(cx);
                    let new_value = (old_value + normalized_offset).clamp(0.0, 1.0);
                    if old_value != new_value {
                        normalized_value.set(cx, new_value);
                        (self.on_value_changed)(cx, mapper.denormalize(new_value));
                    }
                }
            });
            build_knob(cx, move |cx| normalized_value.get(cx));
        }
    }
}

fn build_knob(
    cx: &mut BuildContext,
    f_normalized_value: impl Fn(&mut RenderContext) -> f64 + 'static,
) {
    cx.intrinsic_size(INTRINSIC_SIZE).render(move |cx| {
        const MIN_ANGLE: f32 = -1.25 * std::f32::consts::PI;
        const MAX_ANGLE: f32 = 0.25 * std::f32::consts::PI;

        let bounds = cx.bounds();
        let shape = Circle::from_rect(bounds);
        let normalized_value = f_normalized_value(cx) as f32;
        let angle = MIN_ANGLE * (1.0 - normalized_value) + MAX_ANGLE * normalized_value;

        let dot_pos = shape.center + Vec2::new(angle.cos(), angle.sin()).scale(0.7 * shape.radius);
        cx.fill(shape, Color::GREEN);
        cx.fill(Circle::new(dot_pos, 0.15 * shape.radius), Color::BLACK);
    });
}

pub struct ParameterKnob<P> {
    editor: ParamSetter<P>,
    signal: ReadSignal<NormalizedValue>,
}

impl<P: AnyParameter> ParameterKnob<P> {
    pub fn new(parameter: &P) -> Self {
        Self {
            signal: parameter.as_signal_normalized(),
            editor: ParamSetter::new(parameter),
        }
    }
}

impl<P: AnyParameter> View for ParameterKnob<P> {
    fn build(self, cx: &mut BuildContext) {
        let editor = self.editor;
        let signal = self.signal;
        cx.knob(move |cx, event| match event {
            KnobEvent::DragStart => editor.begin_edit(cx),
            KnobEvent::DragEnd => editor.end_edit(cx),
            KnobEvent::ValueOffset { normalized_offset } => {
                let old_value = signal.get_untracked(cx).0;
                let new_value = (old_value + normalized_offset).clamp(0.0, 1.0);
                if old_value != new_value {
                    editor.set_value_normalized(cx, NormalizedValue(new_value))
                }
            }
        });
        build_knob(cx, move |cx| signal.get(cx).0);
    }
}
