use crate::{
    app::{
        Accessor, BuildContext, CallbackContext, EventContext, EventStatus, LinearGradient,
        MouseEventContext, ParamSetter, RenderContext, StatusChange, View, Widget,
    },
    core::{Circle, Color, Key, Point, Rectangle, RoundedRectangle, Size, UnitPoint},
    event::MouseButton,
    param::{AnyParameter, NormalizedValue, PlainValue},
    style::{AvailableSpace, DisplayStyle, Length, Measure, Style},
    KeyEvent, MouseEvent,
};

use super::util::{denormalize_value, normalize_value};

#[derive(Default)]
enum Direction {
    #[default]
    Horizontal,
    Vertical,
}

pub struct Slider {
    min: f64,
    max: f64,
    value: Option<Accessor<f64>>,
    on_drag_start: Option<Box<dyn Fn(&mut CallbackContext)>>,
    on_drag_end: Option<Box<dyn Fn(&mut CallbackContext)>>,
    on_value_changed: Option<Box<dyn Fn(&mut CallbackContext, f64)>>,
    direction: Direction,
}

impl Slider {
    pub fn new() -> Self {
        Self {
            min: 0.0,
            max: 1.0,
            value: None,
            on_drag_start: None,
            on_drag_end: None,
            on_value_changed: None,
            direction: Default::default(),
        }
    }

    pub fn vertical(mut self) -> Self {
        self.direction = Direction::Vertical;
        self
    }

    pub fn on_value_changed(mut self, f: impl Fn(&mut CallbackContext, f64) + 'static) -> Self {
        self.on_value_changed = Some(Box::new(f));
        self
    }

    pub fn on_drag_start(mut self, f: impl Fn(&mut CallbackContext) + 'static) -> Self {
        self.on_drag_start = Some(Box::new(f));
        self
    }

    pub fn on_drag_end(mut self, f: impl Fn(&mut CallbackContext) + 'static) -> Self {
        self.on_drag_end = Some(Box::new(f));
        self
    }

    pub fn value(mut self, value: impl Into<Accessor<f64>>) -> Self {
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
        ctx.set_default_style(Style {
            size: Size::new(Length::Auto, Length::Px(10.0)),
            ..Default::default()
        });

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
            on_value_changed: self.on_value_changed,
            direction: self.direction,
            ..Default::default()
        }
    }
}

pub struct ParameterSlider<P: AnyParameter> {
    editor: ParamSetter<P>,
    signal: Accessor<NormalizedValue>,
    direction: Direction,
}

impl<P: AnyParameter> ParameterSlider<P> {
    pub fn new(parameter: &P) -> Self {
        let signal = parameter.as_signal_normalized().into();
        let editor = ParamSetter::new(parameter);
        Self {
            editor,
            signal,
            direction: Default::default(),
        }
    }

    pub fn vertical(mut self) -> Self {
        self.direction = Direction::Vertical;
        self
    }
}

impl<P: AnyParameter> View for ParameterSlider<P> {
    type Element = SliderWidget;

    fn build(self, ctx: &mut BuildContext<Self::Element>) -> Self::Element {
        let editor = self.editor;
        ctx.set_focusable(true);
        ctx.set_default_style(Style {
            size: match self.direction {
                Direction::Horizontal => Size::new(Length::Auto, Length::Px(10.0)),
                Direction::Vertical => Size::new(Length::Px(10.0), Length::Auto),
            },
            ..Default::default()
        });

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

pub struct SliderWidget {
    /// Normalized position, between 0 and 1
    position_normalized: f64,
    state: State,
    min: f64,
    max: f64,
    on_drag_start: Option<Box<dyn Fn(&mut CallbackContext)>>,
    on_drag_end: Option<Box<dyn Fn(&mut CallbackContext)>>,
    on_value_changed: Option<Box<dyn Fn(&mut CallbackContext, f64)>>,
    direction: Direction,
    knob_gradient_up: LinearGradient,
    knob_gradient_down: LinearGradient,
    background_gradient: LinearGradient,
}

#[derive(Debug, PartialEq)]
enum State {
    Idle,
    KnobHover,
    Dragging,
}

impl SliderWidget {
    fn slider_position(&self, bounds: Rectangle) -> Point {
        let slider_bounds = self.inner_bounds(bounds);
        match self.direction {
            Direction::Horizontal => Point {
                x: slider_bounds.left() + self.position_normalized * slider_bounds.width(),
                y: slider_bounds.center().y,
            },
            Direction::Vertical => Point {
                x: slider_bounds.center().x,
                y: slider_bounds.top() + self.position_normalized * slider_bounds.height(),
            },
        }
    }

    fn inner_bounds(&self, bounds: Rectangle) -> Rectangle {
        match self.direction {
            Direction::Horizontal => bounds.shrink_x(self.knob_radius(bounds)),
            Direction::Vertical => bounds.shrink_y(self.knob_radius(bounds)),
        }
    }

    fn knob_shape(&self, bounds: Rectangle) -> Circle {
        Circle::new(self.slider_position(bounds), self.knob_radius(bounds))
    }

    fn knob_radius(&self, bounds: Rectangle) -> f64 {
        bounds.height().min(bounds.width()) / 2.0
    }

    fn absolute_to_normalized_position(&self, position: Point, bounds: Rectangle) -> f64 {
        match self.direction {
            Direction::Horizontal => {
                ((position.x - bounds.left() - 2.5) / (bounds.width() - 5.0)).clamp(0.0, 1.0)
            }
            Direction::Vertical => {
                ((position.y - bounds.top() - 2.5) / (bounds.height() - 5.0)).clamp(0.0, 1.0)
            }
        }
    }

    fn set_position(&mut self, cx: &mut CallbackContext, normalized_position: f64) -> bool {
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
            state: State::Idle,
            min: 0.0,
            max: 1.0,
            on_drag_start: None,
            on_drag_end: None,
            on_value_changed: None,
            direction: Default::default(),
            knob_gradient_up: LinearGradient::new(
                (
                    Color::from_rgb8(0xA7, 0xA7, 0xA7),
                    Color::from_rgb8(0xDA, 0xDA, 0xDA),
                ),
                UnitPoint::TOP_CENTER,
                UnitPoint::BOTTOM_CENTER,
            ),
            knob_gradient_down: LinearGradient::new(
                (
                    Color::from_rgb8(0xA7, 0xA7, 0xA7),
                    Color::from_rgb8(0xDA, 0xDA, 0xDA),
                ),
                UnitPoint::BOTTOM_CENTER,
                UnitPoint::TOP_CENTER,
            ),
            background_gradient: LinearGradient::new(
                (Color::BLACK.with_alpha(0.2), Color::WHITE.with_alpha(0.2)),
                UnitPoint::TOP_CENTER,
                UnitPoint::BOTTOM_CENTER,
            ),
        }
    }
}

impl Measure for SliderWidget {
    fn measure(&self, _style: &Style, width: AvailableSpace, height: AvailableSpace) -> Size<f64> {
        let width = match width {
            AvailableSpace::Exact(x) => x,
            AvailableSpace::MinContent => 5.0,
            AvailableSpace::MaxContent => 500.0,
        };
        let height = height.unwrap_or(5.0);

        match self.direction {
            Direction::Horizontal => Size::new(width, height),
            Direction::Vertical => Size::new(height, width),
        }
    }
}

impl Widget for SliderWidget {
    fn debug_label(&self) -> &'static str {
        "Slider"
    }

    fn mouse_event(&mut self, event: MouseEvent, ctx: &mut MouseEventContext) -> EventStatus {
        match event {
            MouseEvent::Down {
                button, position, ..
            } => {
                if button == MouseButton::LEFT && self.state != State::Dragging {
                    if !self.knob_shape(ctx.bounds()).contains(position) {
                        let normalized_position =
                            self.absolute_to_normalized_position(position, ctx.bounds());
                        if self.set_position(&mut ctx.as_callback_context(), normalized_position) {
                            ctx.request_render();
                        }
                    }
                    ctx.capture_mouse();
                    ctx.request_render();
                    if let Some(f) = self.on_drag_start.as_ref() {
                        f(&mut ctx.as_callback_context());
                    }
                    self.state = State::Dragging;
                }
                EventStatus::Handled
            }
            MouseEvent::Moved { position, .. } => {
                match self.state {
                    State::Idle => {
                        if self.knob_shape(ctx.bounds()).contains(position) {
                            ctx.request_render();
                            self.state = State::KnobHover;
                        }
                    }
                    State::KnobHover => {
                        if !self.knob_shape(ctx.bounds()).contains(position) {
                            ctx.request_render();
                            self.state = State::Idle;
                        }
                    }
                    State::Dragging => {
                        let normalized_position =
                            self.absolute_to_normalized_position(position, ctx.bounds());
                        if self.set_position(&mut ctx.as_callback_context(), normalized_position) {
                            ctx.request_render();
                        }
                    }
                }
                EventStatus::Handled
            }
            MouseEvent::Up {
                button, position, ..
            } => {
                if button == MouseButton::LEFT {
                    if self.state == State::Dragging {
                        if let Some(f) = self.on_drag_end.as_ref() {
                            f(&mut ctx.as_callback_context())
                        }
                    }
                    ctx.release_capture();
                    ctx.request_render();
                    self.state = if self.knob_shape(ctx.bounds()).contains(position) {
                        State::KnobHover
                    } else {
                        State::Idle
                    };
                }
                EventStatus::Handled
            }
            _ => EventStatus::Ignored,
        }
    }

    fn key_event(&mut self, event: KeyEvent, ctx: &mut EventContext) -> EventStatus {
        match event {
            crate::KeyEvent::KeyDown { key, .. } => match key {
                Key::Left | Key::Down => {
                    let new_position = (self.position_normalized - 0.1).clamp(0.0, 1.0);
                    if self.set_position(&mut ctx.as_callback_context(), new_position) {
                        ctx.request_render();
                    }
                    EventStatus::Handled
                }
                Key::Right | Key::Up => {
                    let new_position = (self.position_normalized + 0.1).clamp(0.0, 1.0);
                    if self.set_position(&mut ctx.as_callback_context(), new_position) {
                        ctx.request_render();
                    }
                    EventStatus::Handled
                }
                _ => EventStatus::Ignored,
            },
            _ => EventStatus::Ignored,
        }
    }

    fn status_change(&mut self, event: StatusChange, ctx: &mut EventContext) {
        match event {
            StatusChange::FocusGained | StatusChange::FocusLost => ctx.request_render(),
            StatusChange::MouseCaptureLost => {
                if self.state == State::Dragging {
                    self.state = State::Idle;
                    if let Some(f) = self.on_drag_end.as_ref() {
                        f(&mut ctx.as_callback_context())
                    }
                }
            }
            _ => {}
        }
    }

    fn render(&mut self, ctx: &mut RenderContext) {
        let bounds = ctx.content_bounds();
        let center = bounds.center();
        let knob_shape = self.knob_shape(bounds);
        let knob_radius = self.knob_radius(bounds);

        if ctx.has_focus() {
            //ctx.stroke(bounds, Color::RED, 1.0);
        }

        let indent_rect = match self.direction {
            Direction::Horizontal => Rectangle::from_center(center, bounds.size.scale_y(0.3)),
            Direction::Vertical => Rectangle::from_center(center, bounds.size.scale_x(0.3)),
        };
        let corner_radius = indent_rect.height().min(indent_rect.width()) / 2.0;
        let background_rect = RoundedRectangle::new(indent_rect, Size::splat(corner_radius));
        /*let range_indicator_rect = Rectangle::from_ltrb(
        bounds.left(),
        center.y - bounds.height() / 5.0,
        slider_position.x,
        center.y + bounds.height() / 5.0);*/

        ctx.stroke(background_rect, &self.background_gradient, 1.0);
        ctx.fill(background_rect, Color::BLACK.with_alpha(0.3));
        //ctx.fill(RoundedRectangle::new(range_indicator_rect, Size::new(1.0, 1.0)), Color::NEON_GREEN);
        ctx.fill(knob_shape, &self.knob_gradient_down);
        ctx.fill(
            knob_shape.with_radius(4.0 * knob_radius / 5.0),
            &self.knob_gradient_up,
        );
    }

    fn display_style(&self) -> DisplayStyle {
        DisplayStyle::Leaf(self)
    }
}
