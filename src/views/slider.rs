use crate::{core::{Color, Point, Rectangle, Shape, Size}, event::MouseButton, Event, LayoutHint, MouseEvent, View, Widget};

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

    pub fn on_value_changed(mut self, f: impl Fn()) -> Self {
        self.on_value_changed = Box::new(f);
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

    fn build(&mut self, _ctx: &mut crate::BuildContext) -> Self::Element {
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
}

impl Widget for SliderWidget {
    fn event(&mut self, event: Event, ctx: &mut crate::EventContext<()>) {
        match event {
            Event::Mouse(mouse_event) => {
                match mouse_event {
                    MouseEvent::Down { button, position } => {
                        if self.knob_shape(ctx.local_bounds()).hit_test(position) {
                            ctx.set_handled();
                            if button == MouseButton::LEFT && self.state != State::Dragging {
                                ctx.request_render();
                                if let Some(f) = self.on_drag_start.as_ref() {
                                    f();
                                }
                                self.state = State::Dragging;
                            }
                        }
                    },
                    MouseEvent::Moved { position } => {
                        match self.state {
                            State::Idle => {
                                if self.knob_shape(ctx.local_bounds()).hit_test(position) {
                                    ctx.request_render();
                                    self.state = State::KnobHover;
                                }
                            },
                            State::KnobHover => {
                                if !self.knob_shape(ctx.local_bounds()).hit_test(position) {
                                    ctx.request_render();
                                    self.state = State::Idle;
                                }
                            },
                            State::Dragging => {
                                let normalized_position = ((position.x - ctx.local_bounds().left()) / ctx.local_bounds().width()).clamp(0.0, 1.0);
                                println!("{}", normalized_position);
                                if normalized_position != self.position_normalized {
                                    self.position_normalized = normalized_position;
                                    ctx.request_render();
                                    if let Some(f) = self.on_value_changed.as_ref() {
                                        f(self.min + (self.max - self.min) * self.position_normalized);
                                    }
                                }
                            },
                        }
                    },
                    MouseEvent::Up { button, position } => {
                        ctx.set_handled();
                        if button == MouseButton::LEFT {
                            if self.state == State::Dragging {
                                if let Some(f) = self.on_drag_end.as_ref() {
                                }
                            }
                            ctx.request_render();
                            self.state = if self.knob_shape(ctx.local_bounds()).hit_test(position) {
                                State::KnobHover
                            } else {
                                State::Idle
                            };
                        }
                    }
                    _ => {}
                }
            },
            _ => {}
        };
    }

    fn layout(&mut self, constraint: crate::core::Constraint, _ctx: &mut crate::LayoutContext) -> Size {
        constraint.clamp(Size::new(100.0, 20.0))
    }

    fn render(&mut self, ctx: &mut crate::RenderContext) {
        let bounds = ctx.local_bounds();
        let center = bounds.center();
        let width = bounds.width();
        let knob_color = match self.state {
            State::Idle => Color::BLACK,
            State::KnobHover => Color::from_rgb(0.5, 0.5, 0.5),
            State::Dragging => Color::from_rgb(0.75, 0.75, 0.75),
        };

        ctx.fill(Rectangle::from_center(center, Size::new(width, 2.0)), Color::BLACK);
        ctx.fill(self.knob_shape(bounds), knob_color);
    }

    fn layout_hint(&self) -> (LayoutHint, LayoutHint) {
        (LayoutHint::Flexible, LayoutHint::Fixed)
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