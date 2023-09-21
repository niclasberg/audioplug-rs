use crate::{View, Event, MouseEvent, event::MouseButton};

enum ButtonMessage {
    Clicked
}

struct ButtonWidget {

}

impl ButtonWidget {
    pub fn new() -> Self {
        Self {}
    }
}

impl View<ButtonMessage> for ButtonWidget {
    type State = ();

    fn build(&self, view_id: &crate::IdPath) -> Self::State {
        todo!()
    }

    fn rebuild(&self, view_id: &crate::IdPath, prev: &Self, state: &mut Self::State) {
        todo!()
    }

    fn event(&mut self, state: &mut Self::State, event: crate::Event, ctx: &mut crate::EventContext<ButtonMessage>) {
        match event {
            Event::Mouse(mouse_event) => match mouse_event {
                MouseEvent::Down { button, .. } if button == MouseButton::LEFT => {
                    ctx.publish_message(ButtonMessage::Clicked)
                },
                _ => {}
            },
            _ => {}
        }
    }

    fn layout(&mut self, state: &Self::State, constraint: crate::core::Constraint) -> crate::core::Size {
        todo!()
    }

    fn render(&self, state: &Self::State, bounds: crate::core::Rectangle, ctx: &mut crate::window::Renderer) {
        todo!()
    }
}