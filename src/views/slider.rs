use crate::{
    core::{Axis, Capsule, Circle, Color, LinearGradient, Rect, Size},
    param::{AnyParameter, NormalizedValue},
    ui::{
        BuildContext, EventContext, Prop, RenderContext, SliderEvent, StyleExt, View, ViewExt,
        WidgetProps,
        reactive::{ParamSetter, ReactiveValue, ReadSignal, Var},
    },
    views::util::ValueMapper,
};

type OnDragCallback = dyn Fn(&mut EventContext);
type OnValueChangeCallback = dyn Fn(&mut EventContext, f64);

pub struct Slider {
    min: f64,
    max: f64,
    value: Option<Prop<f64>>,
    on_drag_start: Option<Box<OnDragCallback>>,
    on_drag_end: Option<Box<OnDragCallback>>,
    on_value_changed: Box<OnValueChangeCallback>,
    direction: Axis,
    props: WidgetProps,
}

impl Slider {
    pub fn new(value_change_fn: impl Fn(&mut EventContext, f64) + 'static) -> Self {
        Self {
            min: 0.0,
            max: 1.0,
            value: None,
            on_value_changed: Box::new(value_change_fn),
            direction: Axis::Horizontal,
            props: Default::default(),
            on_drag_start: None,
            on_drag_end: None,
        }
    }

    pub fn vertical(mut self) -> Self {
        self.direction = Axis::Vertical;
        self
    }

    pub fn on_drag_start(mut self, f: impl Fn(&mut EventContext) + 'static) -> Self {
        self.on_drag_start = Some(Box::new(f));
        self
    }

    pub fn on_drag_end(mut self, f: impl Fn(&mut EventContext) + 'static) -> Self {
        self.on_drag_end = Some(Box::new(f));
        self
    }

    pub fn value(mut self, value: impl Into<Prop<f64>>) -> Self {
        self.value = Some(value.into());
        self
    }

    pub fn range(mut self, min: f64, max: f64) -> Self {
        self.min = min;
        self.max = max;
        self
    }
}

impl View for Slider {
    fn build(self, cx: &mut BuildContext) {
        let mapper = ValueMapper {
            min: self.min,
            max: self.max,
            steps: 0,
        };

        let on_drag_start = move |cx: &mut EventContext| {
            if let Some(drag_started) = self.on_drag_start.as_ref() {
                drag_started(cx);
            }
        };

        let on_drag_end = move |cx: &mut EventContext| {
            if let Some(drag_ended) = self.on_drag_end.as_ref() {
                drag_ended(cx);
            }
        };

        if let Some(value) = self.value {
            cx.slider(
                move |cx, event| match event {
                    SliderEvent::DragStart => on_drag_start(cx),
                    SliderEvent::DragEnd => on_drag_end(cx),
                    SliderEvent::ValueOffset { normalized_offset } => {}
                    SliderEvent::ValueTo { normalized_value } => todo!(),
                },
                self.direction,
            );
            build_slider(cx, self.props, self.direction, move |cx| {
                mapper.normalize(value.get(cx))
            });
        } else {
            let value = Var::new(cx, mapper.min);

            build_slider(cx, self.props, self.direction, move |cx| {
                mapper.normalize(value.get(cx))
            });
        };
    }
}

fn build_slider(
    cx: &mut BuildContext,
    props: WidgetProps,
    direction: Axis,
    normalized_value: impl Fn(&mut RenderContext) -> f64 + 'static,
) {
    const MAIN_SIZE: f32 = 120.0;
    const CROSS_SIZE: f32 = 16.0;
    let intrinsic_size = match direction {
        Axis::Horizontal => Size::new(MAIN_SIZE, CROSS_SIZE),
        Axis::Vertical => Size::new(CROSS_SIZE, MAIN_SIZE),
    };

    cx.intrinsic_size(intrinsic_size)
        .props(props)
        .render(move |cx| {
            let pos = normalized_value(cx) as f32;
            let bounds = cx.bounds();
            let knob_radius = 0.5 * bounds.size().min_element();
            let slider_position = match direction {
                Axis::Horizontal => bounds.shrink_x(knob_radius).get_relative_point(pos, 0.5),
                Axis::Vertical => bounds.shrink_y(knob_radius).get_relative_point(0.5, pos),
            };

            let center = bounds.center();
            let knob_shape = Circle::new(slider_position, knob_radius);
            let knob_bounds = knob_shape.bounds();

            let indent_rect = match direction {
                Axis::Horizontal => Rect::from_center(center, bounds.size().scale_y(0.3)),
                Axis::Vertical => Rect::from_center(center, bounds.size().scale_x(0.3)),
            };
            let background_rect = Capsule::from_rect(indent_rect, direction);
            let knob_gradient_up = LinearGradient::new(
                (
                    Color::from_rgb8(0xA7, 0xA7, 0xA7),
                    Color::from_rgb8(0xDA, 0xDA, 0xDA),
                ),
                knob_bounds.get_relative_point(0.5, 0.0),
                knob_bounds.get_relative_point(0.5, 1.0),
            );
            let knob_gradient_down = LinearGradient::new(
                (
                    Color::from_rgb8(0xA7, 0xA7, 0xA7),
                    Color::from_rgb8(0xDA, 0xDA, 0xDA),
                ),
                knob_bounds.get_relative_point(0.5, 1.0),
                knob_bounds.get_relative_point(0.5, 0.0),
            );
            let background_gradient = LinearGradient::new(
                (Color::BLACK.with_alpha(0.2), Color::WHITE.with_alpha(0.2)),
                indent_rect.get_relative_point(0.5, 0.0),
                indent_rect.get_relative_point(0.5, 1.0),
            );

            cx.stroke(background_rect, &background_gradient, 1.0);
            cx.fill(background_rect, Color::BLACK.with_alpha(0.3));
            //ctx.fill(RoundedRectangle::new(range_indicator_rect, Size::new(1.0, 1.0)), Color::NEON_GREEN);
            cx.fill(knob_shape, &knob_gradient_down);
            cx.fill(
                knob_shape.with_radius(4.0 * knob_radius / 5.0),
                &knob_gradient_up,
            );
        });
}

impl ViewExt for Slider {
    fn props_mut(&mut self) -> &mut WidgetProps {
        &mut self.props
    }
}
impl StyleExt for Slider {}

pub struct ParameterSlider<P: AnyParameter> {
    props: WidgetProps,
    editor: ParamSetter<P>,
    signal: ReadSignal<NormalizedValue>,
    direction: Axis,
}

impl<P: AnyParameter> ParameterSlider<P> {
    pub fn new(parameter: &P) -> Self {
        let signal = parameter.as_signal_normalized();
        let editor = ParamSetter::new(parameter);
        Self {
            editor,
            signal,
            direction: Axis::Horizontal,
            props: WidgetProps::default(),
        }
    }

    pub fn vertical(mut self) -> Self {
        self.direction = Axis::Vertical;
        self
    }
}

impl<P: AnyParameter> View for ParameterSlider<P> {
    fn build(self, cx: &mut BuildContext) {
        let editor = self.editor;
        let value = self.signal;
        build_slider(cx, self.props, self.direction, move |cx| value.get(cx).0);
        cx.slider(
            move |cx, event| match event {
                SliderEvent::DragStart => editor.begin_edit(cx),
                SliderEvent::DragEnd => editor.end_edit(cx),
                SliderEvent::ValueOffset { normalized_offset } => todo!(),
                SliderEvent::ValueTo { normalized_value } => todo!(),
            },
            self.direction,
        );
    }
}

impl<P: AnyParameter> ViewExt for ParameterSlider<P> {
    fn props_mut(&mut self) -> &mut WidgetProps {
        &mut self.props
    }
}

impl<P: AnyParameter> StyleExt for ParameterSlider<P> {}
