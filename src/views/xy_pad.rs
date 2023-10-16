use crate::{View, core::Point};

pub struct XyPad {
    position: Point
}

pub enum XyPadMessage {
    DragStarted,
    DragEnded,
    ValueChanged
}

pub struct XyPadState {

}

impl View for XyPad {
    type Message = XyPadMessage;
    type State = XyPadState;

    fn build(&mut self, ctx: &mut crate::BuildContext) -> Self::State {
        XyPadState {}
    }

    fn rebuild(&mut self, state: &mut Self::State, ctx: &mut crate::BuildContext) {
        
    }

    fn event(&mut self, state: &mut Self::State, event: crate::Event, ctx: &mut crate::EventContext<Self::Message>) {
        
    }

    fn layout(&self, state: &mut Self::State, constraint: crate::core::Constraint, ctx: &mut crate::LayoutContext) -> crate::core::Size {
        todo!()
    }

    fn render(&self, state: &Self::State, ctx: &mut crate::RenderContext) {
        todo!()
    }

    fn layout_hint(&self, state: &Self::State) -> (crate::LayoutHint, crate::LayoutHint) {
        todo!()
    }
}