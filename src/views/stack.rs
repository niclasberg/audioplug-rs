use std::marker::PhantomData;

use crate::{view::{View, EventContext}, core::{Size, Rectangle, Constraint, Alignment}, ViewSequence, IdPath, ViewVisitor, Event, LayoutContext};

pub enum StackType {
    Vertical,
    Horizontal,
    Depth
}

pub struct StackWidget<Msg, VS: ViewSequence<Msg>> {
    view_seq: VS,
    alignment: Alignment,
    _phantom: PhantomData<Msg>
}

impl<Msg, VS: ViewSequence<Msg>> StackWidget<Msg, VS> {
    pub fn new(views: VS) -> Self {
        Self {
            view_seq: views,
            alignment: Alignment::Center,
            _phantom: PhantomData,
        }
    }

    pub fn with_alignment(mut self, alignment: Alignment) -> Self {
        self.alignment = alignment;
        self
    }
}

impl<Msg, VS: ViewSequence<Msg>> View for StackWidget<Msg, VS> {
    type Message = Msg;
	type State = VS::State;

    fn build(&mut self, id_path: &IdPath) -> Self::State {
        self.view_seq.build(id_path)
    }

    fn rebuild(&mut self, id_path: &IdPath, prev: &Self, state: &mut Self::State) {
        todo!()
    }

    fn event<'a, 'b>(&mut self, state: &mut Self::State, event: Event, ctx: &mut EventContext<'a, 'b, Msg>) {
        struct Visitor<'a, 'b, Msg> {
            event: Event,
            _phantom: PhantomData<fn(&'a Msg) -> &'b ()>
        }

        impl<'a, 'b, Msg> ViewVisitor<Msg, EventContext<'a, 'b, Msg>> for Visitor<'a, 'b, Msg> {
            fn visit<V: View<Message = Msg>>(&mut self, context: &mut EventContext<'a, 'b, Msg>, view: &mut V, state: &mut V::State) {
                view.event(state, self.event, context);
            }
        }

        self.view_seq.for_each(ctx, state, &mut Visitor { event, _phantom: PhantomData });
    }

    fn layout(&self, state: &Self::State, constraint: Constraint, ctx: &mut LayoutContext) -> Size {
        /*struct Visitor<'a> {
            constraint: Constraint
        }

        impl ViewVisitor<LayoutContext> for Visitor {
            fn visit<V: View>(&mut self, context: &mut LayoutContext, view: &mut V, state: &mut V::State) {
                todo!()
            }
        }



        let mut max_size = Size::ZERO;
        for child in self.children.iter_mut() {
            let size = child.layout(constraint);
            child.state.set_size(max_size);
            max_size = max_size.max(&size);
        }

        for child in self.children.iter_mut() {
            child.state.set_origin(self.alignment.compute_offset(child.state.size, max_size));
        }

        max_size*/
        todo!()
    }

    fn render(&self, state: &Self::State, bounds: Rectangle, ctx: &mut crate::window::Renderer) {
        /*println!("Stack render");
        for child in self.children.iter() {
            child.render(bounds, ctx);
        }*/
    }
}


pub fn zstack<Message, Vs: ViewSequence<Message>>(view_seq: Vs) -> StackWidget<Message, Vs> {
    StackWidget::new(view_seq)
}