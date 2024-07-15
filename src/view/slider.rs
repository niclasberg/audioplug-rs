use crate::{core::{Color, Point, Rectangle, Shape, Size}, event::MouseButton, keyboard::Key, KeyEvent, MouseEvent};

use super::{BuildContext, EventContext, EventStatus, LayoutContext, RenderContext, View, Widget};

pub struct Slider {
    min: f64,
    max: f64,
    on_drag_start: Option<Box<dyn Fn()>>, 
    on_drag_end: Option<Box<dyn Fn()>>, 
    on_value_changed: Option<Box<dyn Fn(f64)>>
}

impl Slider {
    pub fn new() -> Self {
        Self { 
            min: 0.0, 
            max: 1.0, 
            on_drag_start: None,
            on_drag_end: None,
            on_value_changed: None 
        }
    }

    pub fn on_value_changed(mut self, f: impl Fn(f64) + 'static) -> Self {
        self.on_value_changed = Some(Box::new(f));
        self
    }

    pub fn with_range(mut self, min: f64, max: f64) -> Self {
        self.min = min;
        self.max = max;
        self
    }
}

impl View for Slider {
    type Element = SliderWidget;

    fn build(self, ctx: &mut BuildContext) -> Self::Element {
        ctx.set_focusable(true);
        SliderWidget {
            position_normalized: 0.4,
            state: State::Idle,
            min: self.min,
            max: self.max, 
            on_drag_start: self.on_drag_start,
            on_drag_end: self.on_drag_end,
            on_value_changed: self.on_value_changed
        }
    }
}

pub struct SliderWidget {
    /// Normalized position, between 0 and 1
    position_normalized: f64,
    state: State,
    min: f64,
    max: f64,
    on_drag_start: Option<Box<dyn Fn()>>, 
    on_drag_end: Option<Box<dyn Fn()>>, 
    on_value_changed: Option<Box<dyn Fn(f64)>>
}

#[derive(Debug, PartialEq)]
enum State {
    Idle,
    KnobHover,
    Dragging
}

impl SliderWidget {
    fn slider_position(&self, bounds: Rectangle) -> Point {
        let knob_x = bounds.left() + self.position_normalized * bounds.width();
        let knob_y = bounds.center().y;
        Point::new(knob_x, knob_y)
    }

    fn knob_shape(&self, bounds: Rectangle) -> Shape {
        Shape::circle(self.slider_position(bounds), 5.0)
    }

    fn set_position(&mut self, normalized_position: f64, ctx: &mut EventContext) {
        if normalized_position != self.position_normalized {
            self.position_normalized = normalized_position;
            ctx.request_render();
            if let Some(f) = self.on_value_changed.as_ref() {
                f(self.min + (self.max - self.min) * self.position_normalized);
            }
        }
    }
}

impl Widget for SliderWidget {
    fn mouse_event(&mut self, event: MouseEvent, ctx: &mut EventContext) -> EventStatus {
        match event {
            MouseEvent::Down { button, position } => {
                if self.knob_shape(ctx.bounds()).hit_test(position) {
                    if button == MouseButton::LEFT && self.state != State::Dragging {
                        ctx.capture_mouse();
                        ctx.request_render();
                        if let Some(f) = self.on_drag_start.as_ref() {
                            f();
                        }
                        self.state = State::Dragging;
                    }
                }
                EventStatus::Handled
            },
            MouseEvent::Moved { position } => {
                match self.state {
                    State::Idle => {
                        if self.knob_shape(ctx.bounds()).hit_test(position) {
                            ctx.request_render();
                            self.state = State::KnobHover;
                        }
                    },
                    State::KnobHover => {
                        if !self.knob_shape(ctx.bounds()).hit_test(position) {
                            ctx.request_render();
                            self.state = State::Idle;
                        }
                    },
                    State::Dragging => {
                        let normalized_position = ((position.x - ctx.bounds().left()) / ctx.bounds().width()).clamp(0.0, 1.0);
                        self.set_position(normalized_position, ctx);
                    },
                }
                EventStatus::Handled
            },
            MouseEvent::Up { button, position } => {
                if button == MouseButton::LEFT {
                    if self.state == State::Dragging {
                        if let Some(f) = self.on_drag_end.as_ref() {
                            f()
                        }
                    }
                    ctx.request_render();
                    self.state = if self.knob_shape(ctx.bounds()).hit_test(position) {
                        State::KnobHover
                    } else {
                        State::Idle
                    };
                }
                EventStatus::Handled
            }
            _ => EventStatus::Ignored
        }
    }

    fn key_event(&mut self, event: KeyEvent, ctx: &mut EventContext) -> EventStatus {
        match event {
            crate::KeyEvent::KeyDown { key, .. } => {
                match key {
                    Key::Left => {
                        let new_position = (self.position_normalized - 0.1).clamp(0.0, 1.0);
                        self.set_position(new_position, ctx);
                        EventStatus::Handled
                    },
                    Key::Right => {
                        let new_position = (self.position_normalized + 0.1).clamp(0.0, 1.0);
                        self.set_position(new_position, ctx);
                        EventStatus::Handled
                    },
                    _ => EventStatus::Ignored
                }
            },
            _ => EventStatus::Ignored
        }
    }

    fn layout(&mut self, inputs: taffy::LayoutInput, ctx: &mut LayoutContext) -> taffy::LayoutOutput {
        ctx.compute_leaf_layout(inputs, |known_size, available_space| {
            known_size.unwrap_or(available_space.map(|x| match x {
                taffy::AvailableSpace::Definite(x) => x,
                taffy::AvailableSpace::MinContent => 5.0,
                taffy::AvailableSpace::MaxContent => 100.0,
            }))
        })
    }

    fn focus_changed(&mut self, _has_focus: bool, ctx: &mut EventContext) {
        ctx.request_render()
    }

    fn render(&mut self, ctx: &mut RenderContext) {
        let bounds = ctx.global_bounds();
        let center = bounds.center();
        let width = bounds.width();
        let knob_color = match self.state {
            State::Idle => Color::BLACK,
            State::KnobHover => Color::from_rgb(0.5, 0.5, 0.5),
            State::Dragging => Color::from_rgb(0.75, 0.75, 0.75),
        };

        if ctx.has_focus() {
            ctx.stroke(bounds, Color::RED, 1.0);
        }
        
        ctx.fill(Rectangle::from_center(center, Size::new(width, 2.0)), Color::BLACK);
        ctx.fill(self.knob_shape(bounds), knob_color);
    }
    
    fn style(&self) -> taffy::Style {
        taffy::Style {
            size: taffy::Size { width: taffy::Dimension::Auto, height: taffy::Dimension::Length(5.0) },
            ..Default::default()
        }
    }
}

/*pub struct ParameterSlider {
    slider: Slider,
}

impl View for ParameterSlider {
    type Message = ();
    type State = <Slider as View>::State;

    fn build(&mut self, ctx: &mut crate::BuildContext) -> Self::State {
        self.slider.build(ctx)
    }

    fn rebuild(&mut self, state: &mut Self::State, ctx: &mut crate::BuildContext) {
        todo!()
    }

    fn event(&mut self, state: &mut Self::State, event: Event, ctx: &mut crate::EventContext<Self::Message>) {
        todo!()
    }

    fn layout(&self, state: &mut Self::State, constraint: crate::core::Constraint, ctx: &mut crate::LayoutContext) -> Size {
        todo!()
    }

    fn render(&self, state: &Self::State, ctx: &mut crate::RenderContext) {
        todo!()
    }
}*/