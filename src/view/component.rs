use super::View;

pub trait Component {
    type Input: 'static;
    type Output: 'static;
}

impl<C: Component> View for C {
    type Message = C::Output;
    type State = Self;

    fn build(&mut self, ctx: &mut crate::BuildContext) -> Self::State {
        todo!()
    }

    fn rebuild(&mut self, state: &mut Self::State, ctx: &mut crate::BuildContext) {
        todo!()
    }

    fn event(&mut self, state: &mut Self::State, event: crate::Event, ctx: &mut crate::EventContext<Self::Message>) {
        todo!()
    }

    fn layout(&self, state: &mut Self::State, constraint: crate::core::Constraint, ctx: &mut crate::LayoutContext) -> crate::core::Size {
        todo!()
    }

    fn layout_hint(&self, state: &Self::State) -> (crate::LayoutHint, crate::LayoutHint) {
        todo!()
    }

    fn render(&self, state: &Self::State, ctx: &mut crate::RenderContext) {
        todo!()
    }
}