use crate::{view::View, core::{Size, Rectangle}};

struct LabelWidget {
    pub text: String
}

/*impl Widget for LabelWidget {
    type Message = ();

    fn event(&mut self, event: crate::event::Event) {
        todo!()
    }

    fn layout(&mut self, constraint: crate::core::Constraint) -> Size {
        todo!()
    }

    fn render(&self, bounds: Rectangle, ctx: &mut crate::window::Renderer) {
        todo!()
    }
}

struct Label {
    text: String
}

impl View for Label {
    type Element = LabelWidget;
    type State = ();

    fn build(&self, context: &mut crate::view::BuildContext) -> (crate::view::Id, Self::State, Self::Element) {
        todo!()
    }

    fn rebuild(&self, context: &mut crate::view::BuildContext, prev: &Self, state: &mut Self::State, widget: &mut Self::Element) -> crate::view::ChangeFlags {
        if prev.text != self.text {
            todo!()
        }
        todo!()
    }
}*/