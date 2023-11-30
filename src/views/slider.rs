use crate::{View, core::{Point, Color, Size, Rectangle, Shape}, Event, MouseEvent, event::MouseButton, LayoutHint};

#[derive(Debug, PartialEq)]
pub enum SliderMessage {
    DragStarted,
    DragEnded,
    ValueChanged { current: f64 }
}

pub struct SliderState {
    /// Normalized position, between 0 and 1
    position_normalized: f64,
    state: State
}

#[derive(Debug, PartialEq)]
enum State {
    Idle,
    KnobHover,
    Dragging
}

impl SliderState {
    pub fn slider_position(&self, bounds: Rectangle) -> Point {
        let knob_x = bounds.left() + self.position_normalized * bounds.width();
        let knob_y = bounds.center().y;
        Point::new(knob_x, knob_y)
    }

    fn knob_shape(&self, bounds: Rectangle) -> Shape {
        Shape::circle(self.slider_position(bounds), 5.0)
    }
}

pub struct Slider {
    min: f64,
    max: f64
}

impl Slider {
    pub fn new() -> Self {
        Self { min: 0.0, max: 1.0 }
    }

    pub fn with_range(mut self, min: f64, max: f64) -> Self {
        self.min = min;
        self.max = max;
        self
    }
}

impl View for Slider {
    type Message = SliderMessage;
    type State = SliderState;

    fn build(&mut self, _ctx: &mut crate::BuildContext) -> Self::State {
        SliderState {
            position_normalized: 0.4,
            state: State::Idle
        }
    }

    fn rebuild(&mut self, _state: &mut Self::State, _ctx: &mut crate::BuildContext) {
        
    }

    fn event(&mut self, state: &mut Self::State, event: Event, ctx: &mut crate::EventContext<Self::Message>) {
        match event {
            Event::Mouse(mouse_event) => {
                match mouse_event {
                    MouseEvent::Down { button, position } => {
                        if state.knob_shape(ctx.local_bounds()).hit_test(position) {
                            ctx.set_handled();
                            if button == MouseButton::LEFT && state.state != State::Dragging {
                                ctx.request_render();
                                ctx.publish_message(SliderMessage::DragStarted);
                                state.state = State::Dragging;
                            }
                        }
                    },
                    MouseEvent::Moved { position } => {
                        match state.state {
                            State::Idle => {
                                if state.knob_shape(ctx.local_bounds()).hit_test(position) {
                                    ctx.request_render();
                                    state.state = State::KnobHover;
                                }
                            },
                            State::KnobHover => {
                                if !state.knob_shape(ctx.local_bounds()).hit_test(position) {
                                    ctx.request_render();
                                    state.state = State::Idle;
                                }
                            },
                            State::Dragging => {
                                let normalized_position = ((position.x - ctx.local_bounds().left()) / ctx.local_bounds().width()).clamp(0.0, 1.0);
                                println!("{}", normalized_position);
                                if normalized_position != state.position_normalized {
                                    state.position_normalized = normalized_position;
                                    ctx.request_render();
                                    ctx.publish_message(SliderMessage::ValueChanged { 
                                        current: self.min + (self.max - self.min) * state.position_normalized
                                    });
                                }
                            },
                        }
                    },
                    MouseEvent::Up { button, position } => {
                        ctx.set_handled();
                        if button == MouseButton::LEFT {
                            if state.state == State::Dragging {
                                ctx.publish_message(SliderMessage::DragEnded);
                            }
                            ctx.request_render();
                            state.state = if state.knob_shape(ctx.local_bounds()).hit_test(position) {
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

    fn layout(&self, _state: &mut Self::State, constraint: crate::core::Constraint, _ctx: &mut crate::LayoutContext) -> Size {
        constraint.clamp(Size::new(100.0, 20.0))
    }

    fn render(&self, state: &Self::State, ctx: &mut crate::RenderContext) {
        let bounds = ctx.local_bounds();
        let center = bounds.center();
        let width = bounds.width();
        let knob_color = match state.state {
            State::Idle => Color::BLACK,
            State::KnobHover => Color::from_rgb(0.5, 0.5, 0.5),
            State::Dragging => Color::from_rgb(0.75, 0.75, 0.75),
        };

        ctx.fill(Rectangle::from_center(center, Size::new(width, 2.0)), Color::BLACK);
        ctx.fill(state.knob_shape(bounds), knob_color);
    }

    fn layout_hint(&self, _state: &Self::State) -> (LayoutHint, LayoutHint) {
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