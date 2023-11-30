use audioplug::core::{Color, Constraint, Size, Vector, Alignment, Shape, Rectangle, Point};
use audioplug::views::{use_state, Row, Label, Button, Slider, Column, TextBox, Fill};
use audioplug::{window::*, View, Event, EventContext, MouseEvent, Application, LayoutContext, BuildContext, RenderContext, LayoutHint};

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
        ctx.fill(bounds.scale(0.5).offset(Vector::new(40.0, 40.0)), color);
    }

    fn event(&mut self, _state: &mut Self::State, event: Event, ctx: &mut EventContext<Self::Message>) {
        match event {
            Event::Mouse(mouse_event) => match mouse_event { 
                MouseEvent::Down { position, .. } if ctx.local_bounds().contains(position) => { ctx.publish_message(MyMessage::Clicked) },
                MouseEvent::Up { .. } => { self.active = false; },
                _ => {}
            },
            _ => {}
        }
    }

    fn layout(&self, _state: &mut Self::State, constraint: Constraint, _ctx: &mut LayoutContext) -> Size {
        constraint.clamp(Size::new(300.0, 100.0))
    }

    fn layout_hint(&self, _state: &Self::State) -> (audioplug::LayoutHint, audioplug::LayoutHint) {
        (LayoutHint::Fixed, LayoutHint::Fixed)
    }
}

fn main() {
    //let device = Device::new()?;
    //println!("name: {}, id: {}", device.name()?, device.id()?);

    let mut app = Application::new();
    let _ = Window::open(
        Row::new((
            Column::new((
                Row::new((
                    Label::new("Slider"),
                    Slider::new().with_range(1.0, 10.0).map(|ev| println!("{:?}", ev))
                )).with_spacing(5.0),
                Row::new((
                    Label::new("Button"),
                    Button::new(Label::new("Filled")).map(|_| ())
                )).with_spacing(5.0),
                Row::new((
                    Label::new("Text input"),
                    TextBox::new().map(|_| ())
                ))
            )).with_alignment(Alignment::Leading)
            .with_spacing(5.0),
            Column::new((
                Rectangle::new(Point::ZERO, Size::new(40.0, 40.0)).fill(Color::RED),
                use_state(
                    || true, 
                    |state| { MyWidget { active: *state } }, 
                    |_msg, state| { *state = !*state; }),
            )),
            Shape::circle(Point::new(40.0, 40.0), 40.0).fill(Color::GREEN)
        )).with_alignment(audioplug::core::Alignment::TopLeading)
        .with_spacing(5.0));

    app.run();
}