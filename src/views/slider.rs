use crate::{
    KeyEvent, MouseEvent,
    core::{Axis, Capsule, Circle, Color, Key, LinearGradient, Point, Rect, Size, UnitPoint},
    event::{MouseButton, MouseDownEvent, MouseDragEvent},
    param::{AnyParameter, NormalizedValue, PlainValue},
    ui::{
        BuildContext, EventContext, EventResult, Prop, RenderContext, StyleExt, View, ViewStyle,
        Widget, reactive::ParamSetter,
    },
};

use super::util::{denormalize_value, normalize_value};

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
            on_drag_start: None,
            on_drag_end: None,
            on_value_changed: Box::new(value_change_fn),
            direction: Axis::Horizontal,
            style: ViewStyle::DEFAULT,
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
    type Element = SliderWidget;

    fn build(self, ctx: &mut BuildContext<Self::Element>) -> Self::Element {
        ctx.set_focusable(true);
        ctx.set_draggable(true);
        ctx.apply_style(self.style);
        ctx.set_intrinsic_size(intrinsic_size(self.direction));

        let position_normalized = if let Some(value) = self.value {
            let position = value.get_and_bind(ctx, move |value, mut widget| {
                widget.position_normalized = normalize_value(widget.min, widget.max, value);
                widget.request_render();
            });
            normalize_value(self.min, self.max, position)
        } else {
            0.0
        };

        SliderWidget {
            position_normalized,
            min: self.min,
            max: self.max,
            on_drag_start: self.on_drag_start,
            on_drag_end: self.on_drag_end,
            on_value_changed: Some(self.on_value_changed),
            direction: self.direction,
            ..Default::default()
        }
    }
}

fn intrinsic_size(direction: Axis) -> Size {
    let main_size = 120.0;
    let cross_size = 16.0;
    match direction {
        Axis::Horizontal => Size::new(main_size, cross_size),
        Axis::Vertical => Size::new(cross_size, main_size),
    }
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
    type Element = SliderWidget;

    fn build(self, ctx: &mut BuildContext<Self::Element>) -> Self::Element {
        let editor = self.editor;
        ctx.set_focusable(true);
        ctx.set_draggable(true);
        ctx.apply_style(self.style);
        ctx.set_intrinsic_size(intrinsic_size(self.direction));
        /*ctx.set_default_style(Style {
            size: match self.direction {
                Direction::Horizontal => Size::new(Length::Auto, Length::Px(10.0)),
                Direction::Vertical => Size::new(Length::Px(10.0), Length::Auto),
            },
            ..Default::default()
        });*/

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
            on_drag_start: Some(Box::new(move |cx| {
                editor.begin_edit(cx);
            })),
            on_drag_end: Some(Box::new(move |cx| {
                editor.end_edit(cx);
            })),
            on_value_changed: Some(Box::new(move |cx, value| {
                editor.set_value_plain(cx, PlainValue::new(value));
            })),
            direction: self.direction,
            ..Default::default()
        }
    }
}

impl<P: AnyParameter> StyleExt for ParameterSlider<P> {
    fn style_mut(&mut self) -> &mut ViewStyle {
        &mut self.style
    }
}

pub struct SliderWidget {
    /// Normalized position, between 0 and 1
    position_normalized: f64,
    min: f64,
    max: f64,
    on_drag_start: Option<Box<OnDragCallback>>,
    on_drag_end: Option<Box<OnDragCallback>>,
    on_value_changed: Option<Box<OnValueChangeCallback>>,
    direction: Axis,
}

impl SliderWidget {
    fn slider_position(&self, bounds: Rect) -> Point {
        let slider_bounds = self.inner_bounds(bounds);
        match self.direction {
            Axis::Horizontal => {
                slider_bounds.get_relative_point(self.position_normalized as f32, 0.5)
            }
            Axis::Vertical => {
                slider_bounds.get_relative_point(0.5, self.position_normalized as f32)
            }
        }
    }

    fn inner_bounds(&self, bounds: Rect) -> Rect {
        match self.direction {
            Axis::Horizontal => bounds.shrink_x(self.knob_radius(bounds)),
            Axis::Vertical => bounds.shrink_y(self.knob_radius(bounds)),
        }
    }

    fn knob_shape(&self, bounds: Rect) -> Circle {
        Circle::new(self.slider_position(bounds), self.knob_radius(bounds))
    }

    fn knob_radius(&self, bounds: Rect) -> f32 {
        bounds.height().min(bounds.width()) / 2.0
    }

    fn absolute_to_normalized_position(&self, position: Point, bounds: Rect) -> f64 {
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

    fn set_position(&mut self, cx: &mut EventContext, normalized_position: f64) -> bool {
        if normalized_position != self.position_normalized {
            self.position_normalized = normalized_position;
            if let Some(f) = self.on_value_changed.as_ref() {
                f(
                    cx,
                    denormalize_value(self.min, self.max, self.position_normalized),
                );
            }
            true
        } else {
            false
        }
    }
}

impl Default for SliderWidget {
    fn default() -> Self {
        Self {
            position_normalized: 0.0,
            min: 0.0,
            max: 1.0,
            on_drag_start: None,
            on_drag_end: None,
            on_value_changed: None,
            direction: Axis::Horizontal,
        }
    }
}

impl Widget for SliderWidget {
    fn debug_label(&self) -> &'static str {
        "Slider"
    }

    fn mouse_event(&mut self, event: MouseEvent, ctx: &mut EventContext) {
        match event {
            MouseEvent::Down(MouseDownEvent {
                button, position, ..
            }) => {
                if button == MouseButton::LEFT {
                    if !self.knob_shape(ctx.bounds()).contains(position) {
                        let normalized_position =
                            self.absolute_to_normalized_position(position, ctx.bounds());
                        if self.set_position(ctx, normalized_position) {
                            ctx.request_render();
                        }
                    }
                    ctx.request_render();
                    if let Some(f) = self.on_drag_start.as_ref() {
                        f(ctx);
                    }
                }
            }
            MouseEvent::DragMoved(MouseDragEvent { position, .. }) => {
                let normalized_position =
                    self.absolute_to_normalized_position(position, ctx.bounds());
                if self.set_position(ctx, normalized_position) {
                    ctx.request_render();
                }
            }
            MouseEvent::DragEnded | MouseEvent::DragCancelled => {
                if let Some(f) = self.on_drag_end.as_ref() {
                    f(ctx)
                }
            }
            _ => {}
        }
    }

    fn key_event(&mut self, event: KeyEvent, ctx: &mut EventContext) -> EventResult {
        match event {
            crate::KeyEvent::KeyDown { key, .. } => match key {
                Key::Left | Key::Down => {
                    let new_position = (self.position_normalized - 0.1).clamp(0.0, 1.0);
                    if self.set_position(ctx, new_position) {
                        ctx.request_render();
                    }
                    EventResult::Stop
                }
                Key::Right | Key::Up => {
                    let new_position = (self.position_normalized + 0.1).clamp(0.0, 1.0);
                    if self.set_position(ctx, new_position) {
                        ctx.request_render();
                    }
                    EventResult::Stop
                }
                _ => EventResult::Continue,
            },
            _ => EventResult::Continue,
        }
    }

    fn render(&mut self, cx: &mut RenderContext) {
        let bounds = cx.bounds();
        let center = bounds.center();
        let knob_shape = self.knob_shape(bounds);
        let knob_bounds = knob_shape.bounds();
        let knob_radius = self.knob_radius(bounds);

        let indent_rect = match self.direction {
            Axis::Horizontal => Rect::from_center(center, bounds.size().scale_y(0.3)),
            Axis::Vertical => Rect::from_center(center, bounds.size().scale_x(0.3)),
        };
        let background_rect = Capsule::from_rect(indent_rect, self.direction);
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
