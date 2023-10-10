use crate::{View, core::{Point, Color}, Shape};

pub enum SliderMessage {
    DragStarted,
    DragEnded,
    ValueChanged { current: f64 }
}

pub struct SliderState {
    /// Normalized position, between 0 and 1
    position_normalized: f64,
    last_mouse_position: Option<Point>
}

pub struct Slider {
    min: f64,
    max: f64,
    steps: Option<usize>
}

impl Slider {
    pub fn new() -> Self {
        Self { min: 0.0, max: 1.0, steps: None }
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
            position_normalized: 0.0,
            last_mouse_position: None
        }
    }

    fn rebuild(&mut self, _state: &mut Self::State, _ctx: &mut crate::BuildContext) {
        
    }

    fn event(&mut self, state: &mut Self::State, event: crate::Event, ctx: &mut crate::EventContext<Self::Message>) {
        
    }

    fn layout(&self, state: &mut Self::State, constraint: crate::core::Constraint, ctx: &mut crate::LayoutContext) -> crate::core::Size {
        constraint.max_size
    }

    fn render(&self, state: &Self::State, ctx: &mut crate::RenderContext) {
        let bounds = ctx.local_bounds();
        ctx.fill(&Shape::rect(bounds.size().scale_y(0.1)), bounds.position(), Color::BLACK);

    }
}