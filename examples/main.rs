use audioplug::core::{Rectangle, Color, Constraint, Size, Vector};
use audioplug::views::zstack;
use audioplug::{window::*, View, Event, EventContext, MouseEvent, View, BuildContext, Id, ChangeFlags, Application, Message, IdPath};

struct MyWidget {
    active: bool
}

#[derive(Debug)]
enum MyMessage {
    Clicked
}

impl View for MyWidget {
    type Message = MyMessage;

    fn render(&self, bounds: Rectangle, ctx: &mut Renderer) {
        let color = if self.active { Color::RED } else { Color::WHITE };
        let r = Rectangle::new(bounds.position() + Vector::new(40.0, 40.0), bounds.size().scale(0.5));
        ctx.fill_rectangle(r, color);
        ctx.draw_text("hello howudoin?", bounds);
    }

    fn event(&mut self, event: Event, ctx: &mut EventContext<Self::Message>) {
        //println!("{:?}", event);
        match event {
            Event::Mouse(mouse_event) => match mouse_event { 
                MouseEvent::Down { .. } => { ctx.publish_message(MyMessage::Clicked) },
                MouseEvent::Up { .. } => { self.active = false; },
                _ => {}
            },
            _ => {}
        }
    }

    fn layout(&mut self, constraint: Constraint) -> Size {
        Size::new(100.0, 100.0)
    }
}

struct MyView {
    
}

impl View for MyView {
    type Element = MyWidget;
    type State = ();

    fn build(&self, id_path: &IdPath) -> (Self::State, Self::Element) {
        ((), MyWidget { active: false })
    }

    fn rebuild(&self, id_path: &IdPath, prev: &Self, state: &mut Self::State, widget: &mut Self::Element) -> ChangeFlags {
        ChangeFlags::empty()
    }

    fn message(&mut self, msg: &Message<MyMessage>) {
        match msg {
            Message::Widget(msg) => {
                println!("{:?}", msg);
            }
        }        
    }
}

fn main() {
    //let device = Device::new()?;
    //println!("name: {}, id: {}", device.name()?, device.id()?);

    let mut app = Application::new();
    let _ = Window::open(zstack((
        Color::RED,
        MyView{}
    )));

    app.run();
}