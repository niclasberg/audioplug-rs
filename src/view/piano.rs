use crate::app::Widget;

use super::View;

pub enum PianoKeyEvent {
    NoteDown,
    NoteUp,
}

pub struct Piano {
    
}

impl View for Piano {
    type Element = Piano;

    fn build(self, ctx: &mut crate::app::BuildContext) -> Self::Element {
        todo!()
    }
}

impl Widget for Piano {
    fn debug_label(&self) -> &'static str {
        "Piano"
    }

    fn style(&self) -> taffy::Style {
        taffy::Style::default()
    }

    fn render(&mut self, ctx: &mut crate::app::RenderContext) {
        todo!()
    }
}