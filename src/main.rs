use crate::core::{Rectangle, Color};

use application::Application;
use event::{Event, MouseEvent};
use widget::Widget;

#[cfg(target_os = "windows")]
mod win;

#[cfg(target_os = "macos")]
mod mac;

mod core;
mod widget;
mod window;
mod event;
mod application;
use crate::window::*;

struct MyWidget {
    active: bool
}

impl Widget for MyWidget {
    fn render(&self, ctx: &mut Renderer) {
        let color = if self.active { Color::RED } else { Color::WHITE };
        ctx.fill_rectangle(Rectangle::from_xywh(45.0, 45.0, 100.0, 100.0), color);
    }

    fn event(&mut self, event: Event) {
        println!("{:?}", event);
        match event {
            Event::Mouse(mouse_event) => match mouse_event { 
                MouseEvent::Down { .. } => { self.active = true; },
                MouseEvent::Up { .. } => { self.active = false; },
                _ => {}
            },
            _ => {}
        }
    }

    fn layout(&self, bounds: Rectangle) {
        todo!()
    }
}

fn main() {
    //let device = Device::new()?;
    //println!("name: {}, id: {}", device.name()?, device.id()?);

    let mut app = Application::new();
    let _ = Window::new(MyWidget{ active: false});

    app.run();
}