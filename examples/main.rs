use audioplug::core::{Color, Constraint, Size, Vector};
use audioplug::views::{use_state, Row, Label, Button};
use audioplug::{window::*, View, Event, EventContext, MouseEvent, Application, LayoutContext, BuildContext, RenderContext, Shape};

struct MyWidget {
    active: bool
}

#[derive(Debug)]
enum MyMessage {
    Clicked
}

impl View for MyWidget {
    type State = ();
    type Message = MyMessage;

    fn build(&mut self, _ctx: &mut BuildContext) -> Self::State { }
    fn rebuild(&mut self, _state: &mut Self::State, _ctx: &mut BuildContext) {}

    fn render(&self, _state: &Self::State, ctx: &mut RenderContext) {
        let color = if self.active { Color::BLACK } else { Color::WHITE };
        let bounds = ctx.local_bounds();
        ctx.fill(&Shape::rect(bounds.size().scale(0.5)), bounds.position() + Vector::new(40.0, 40.0), color);
    }

    fn event(&mut self, _state: &mut Self::State, event: Event, ctx: &mut EventContext<Self::Message>) {
        println!("{:?}", event);
        match event {
            Event::Mouse(mouse_event) => match mouse_event { 
                MouseEvent::Down { .. } => { ctx.publish_message(MyMessage::Clicked) },
                MouseEvent::Up { .. } => { self.active = false; },
                _ => {}
            },
            _ => {}
        }
    }

    fn layout(&self, _state: &mut Self::State, constraint: Constraint, _ctx: &mut LayoutContext) -> Size {
        constraint.clamp(Size::new(300.0, 100.0))
    }
}

fn main() {
    //let device = Device::new()?;
    //println!("name: {}, id: {}", device.name()?, device.id()?);

    let mut app = Application::new();
    let _ = Window::open(
        Row::new((
            Shape::rounded_rect(Size::new(40.0, 40.0), Size::new(5.0, 5.0)).fill(Color::RED),
            use_state(
                || true, 
                |state| { MyWidget { active: *state } }, 
                |_msg, state| { *state = !*state; }),
            Row::new((
                Button::new(Label::new("Babushka!")).map(|_| ()),
                Shape::rect(Size::new(40.0, 40.0)).fill(Color::GREEN)
            )).with_alignment(audioplug::core::Alignment::Center)
            .with_spacing(5.0)
        )).with_alignment(audioplug::core::Alignment::TopLeading)
        .with_spacing(5.0));

    app.run();
}