use crate::{
    KeyEvent,
    core::{Axis, Capsule, Circle, Color, Key, LinearGradient, Point, Rect, Size},
    event::MouseDrag,
    param::{AnyParameter, NormalizedValue, PlainValue},
    ui::{
        BuildContext, Capabilities, Event, EventContext, EventResult, Prop, StyleExt, View,
        ViewStyle, WidgetHandler,
        reactive::{ParamSetter, ReactiveValue},
    },
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
    style: ViewStyle,
}

impl Slider {
    pub fn new(value_change_fn: impl Fn(&mut EventContext, f64) + 'static) -> Self {
        Self {
            min: 0.0,
            max: 1.0,
            value: None,
            on_value_changed: Box::new(value_change_fn),
            direction: Axis::Horizontal,
            style: ViewStyle::DEFAULT,
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
        let position_normalized = if let Some(value) = self.value {
            let position = value.get_and_bind(cx, move |value, mut widget| {
                widget.position_normalized = normalize_value(widget.min, widget.max, value);
                widget.request_render();
            });
            normalize_value(self.min, self.max, position)
        } else {
            0.0
        };

        if let Some(value) = self.value {
            build_slider(
                cx,
                self.style,
                self.direction,
                |cx| {
                    if let Some(drag_started) = self.on_drag_start {
                        drag_started(cx)
                    }
                },
                on_value_changed,
                |cx| {
                    if let Some(drag_ended) = self.on_drag_end {
                        drag_ended(cx)
                    }
                },
            );
        }
    }
}

fn build_slider(
    cx: &mut BuildContext,
    style: ViewStyle,
    direction: Axis,
    value: Prop<f64>,
    set_value: impl Fn(&mut EventContext, f64) + 'static,
    offset_value: impl Fn(&mut EventContext, f64) + 'static,
    on_drag_start: impl Fn(&mut EventContext) + 'static,
    on_drag_end: impl Fn(&mut EventContext) + 'static,
) {
    cx.style(style).intrinsic_size(intrinsic_size(direction));

    cx.handler(
        Capabilities::DRAGGABLE | Capabilities::FOCUSABLE,
        SliderHandler {},
    );

    cx.render_fn(|cx| {});
}

fn intrinsic_size(direction: Axis) -> Size {
    let main_size = 120.0;
    let cross_size = 16.0;
    match direction {
        Axis::Horizontal => Size::new(main_size, cross_size),
        Axis::Vertical => Size::new(cross_size, main_size),
    }
}

struct SliderHandler {}

impl WidgetHandler for SliderHandler {
    fn event(&mut self, cx: &mut EventContext, event: &Event) -> EventResult {
        match event {
            Event::Drag(drag_event) => match drag_event {
                MouseDrag::Started { position, .. } => {
                    on_drag_start(cx);
                    set_value(
                        cx,
                        absolute_to_normalized_position(direction, cx.bounds(), *position),
                    );
                    EventResult::Stop
                }
                MouseDrag::Moved { position, .. } => {
                    set_value(
                        cx,
                        absolute_to_normalized_position(direction, cx.bounds(), *position),
                    );
                    EventResult::Stop
                }
                MouseDrag::Ended { .. } | MouseDrag::Cancelled { .. } => {
                    on_drag_end(cx);
                    EventResult::Stop
                }
            },
            Event::Key(key_event) => match key_event {
                KeyEvent::KeyDown { key, .. } => match key {
                    Key::Left | Key::Down => {
                        offset_value(cx, -0.1);
                        EventResult::Stop
                    }
                    Key::Right | Key::Up => {
                        offset_value(cx, 0.1);
                        EventResult::Stop
                    }
                    _ => EventResult::Continue,
                },
                _ => EventResult::Continue,
            },
            _ => EventResult::Continue,
        }
    }

    fn render(&mut self, cx: &mut crate::ui::RenderContext) {
        let bounds = cx.bounds();
        let center = bounds.center();
        let position_normalized = value.get(cx);
        let knob_shape = knob_shape(direction, bounds, position_normalized);
        let knob_bounds = knob_shape.bounds();
        let knob_radius = knob_radius(bounds);

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
    }
}

fn absolute_to_normalized_position(direction: Axis, bounds: Rect, position: Point) -> f64 {
    let normalized_position = match direction {
        Axis::Horizontal => {
            ((position.x - bounds.left - 2.5) / (bounds.width() - 5.0)).clamp(0.0, 1.0)
        }
        Axis::Vertical => {
            ((position.y - bounds.top - 2.5) / (bounds.height() - 5.0)).clamp(0.0, 1.0)
        }
    };
    normalized_position as f64
}

fn knob_shape(direction: Axis, bounds: Rect, pos: f64) -> Circle {
    Circle::new(slider_position(direction, bounds, pos), knob_radius(bounds))
}

fn slider_position(direction: Axis, bounds: Rect, pos: f64) -> Point {
    let slider_bounds = inner_bounds(direction, bounds);
    match direction {
        Axis::Horizontal => slider_bounds.get_relative_point(pos as f32, 0.5),
        Axis::Vertical => slider_bounds.get_relative_point(0.5, pos as f32),
    }
}

fn inner_bounds(direction: Axis, bounds: Rect) -> Rect {
    match direction {
        Axis::Horizontal => bounds.shrink_x(knob_radius(bounds)),
        Axis::Vertical => bounds.shrink_y(knob_radius(bounds)),
    }
}

fn knob_radius(bounds: Rect) -> f32 {
    0.5 * bounds.size().min_element()
}

impl StyleExt for Slider {
    fn style_mut(&mut self) -> &mut ViewStyle {
        &mut self.style
    }
}

pub struct ParameterSlider<P: AnyParameter> {
    style: ViewStyle,
    editor: ParamSetter<P>,
    signal: Prop<NormalizedValue>,
    direction: Axis,
}

impl<P: AnyParameter> ParameterSlider<P> {
    pub fn new(parameter: &P) -> Self {
        let signal = parameter.as_signal_normalized().into();
        let editor = ParamSetter::new(parameter);
        Self {
            editor,
            signal,
            direction: Axis::Horizontal,
            style: ViewStyle::DEFAULT,
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

        build_slider(
            cx,
            self.style,
            self.direction,
            move |cx| editor.begin_edit(cx),
            on_value_changed,
            move |cx| editor.end_edit(cx),
        );

        SliderWidget {
            position_normalized: self.signal.get_and_bind_mapped(
                ctx,
                |value| value.0,
                |value, mut widget| {
                    widget.position_normalized = value;
                    widget.request_render();
                },
            ),
            min: editor.info(ctx).min_value().into(),
            max: editor.info(ctx).max_value().into(),
            on_value_changed: Some(Box::new(move |cx, value| {
                editor.set_value_plain(cx, PlainValue::new(value));
            })),
            ..Default::default()
        }
    }
}

impl<P: AnyParameter> StyleExt for ParameterSlider<P> {
    fn style_mut(&mut self) -> &mut ViewStyle {
        &mut self.style
    }
}
