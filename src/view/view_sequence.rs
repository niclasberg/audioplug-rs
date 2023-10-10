use crate::core::{Constraint, Size};
use crate::{View, Id, BuildContext, Event, EventContext, LayoutContext, RenderContext};

pub struct LayoutProxy<Msg, VS: ViewSequence<Msg>> {
    pub layout: fn(&VS, &VS::State, Constraint, &mut LayoutContext) -> Size
}

pub trait ViewSequence<Msg>: Sized {
    type State;

    fn len(&self) -> usize;
    fn layout_proxies(&self) -> Vec<LayoutProxy<Msg, Self>>;
    fn build(&mut self, ctx: &mut BuildContext) -> Self::State;
    fn rebuild(&mut self, state: &mut Self::State, ctx: &mut BuildContext);
    fn event(&mut self, state: &mut Self::State, event: Event, ctx: &mut EventContext<Msg>);
    fn layout(&self, state: &mut Self::State, constraint: Constraint, ctx: &mut LayoutContext) -> Vec<Size>;
    fn render(&self, state: &Self::State, ctx: &mut RenderContext);
}

macro_rules! impl_view_seq_tuple {
    ( $n: tt; $( $t: ident),* ; $( $s: tt),* ; $( $s_rev: tt),*) => {
        impl<Msg, $( $t: View<Message = Msg>, )* > ViewSequence<Msg> for ($( $t, )*) {
            type State = ($($t::State,)*);

            fn layout_proxies(&self) -> Vec<LayoutProxy<Msg, Self>> {
                todo!()
            }

            fn build(&mut self, ctx: &mut BuildContext) -> Self::State {
                ctx.set_number_of_children($n);
                (
                    $( ctx.with_child(Id($s), |c| self.$s.build(c)), )*
                )
            }
        
            fn rebuild(&mut self, state: &mut Self::State, ctx: &mut BuildContext) {
                $( ctx.with_child(Id($s), |c| self.$s.rebuild(&mut state.$s, c)); )*
            }
        
            fn event(&mut self, state: &mut Self::State, event: Event, ctx: &mut EventContext<Msg>) {
                $( ctx.forward_to_child(Id($s_rev), event, |c, event| self.$s_rev.event(&mut state.$s_rev, event, c)); )*
            }
        
            fn layout(&self, state: &mut Self::State, constraint: Constraint, ctx: &mut LayoutContext) -> Vec<Size> {
                vec![
                    $( ctx.with_child(Id($s), |c| self.$s.layout(&mut state.$s, constraint, c)), )*
                ]
            }
        
            fn render(&self, state: &Self::State, ctx: &mut RenderContext) {
                $( ctx.with_child(Id($s), |c| self.$s.render(&state.$s, c)); )*
            }
        
            fn len(&self) -> usize {
                $n
            }
        }
    }
}

impl_view_seq_tuple!(1; V; 0; 0);
impl_view_seq_tuple!(2; V1, V2; 0, 1; 1, 0);
impl_view_seq_tuple!(3; V1, V2, V3; 0, 1, 2; 2, 1, 0);
impl_view_seq_tuple!(4; V1, V2, V3, V4; 0, 1, 2, 3; 3, 2, 1, 0);

impl<Msg, V: View<Message = Msg>> ViewSequence<Msg> for Vec<V> {
    type State = Vec<V::State>;

    fn build(&mut self, ctx: &mut BuildContext) -> Self::State {
        ctx.set_number_of_children(self.len());
        self.iter_mut().zip(ctx.child_iter()).map(|(view, mut ctx)| {
            view.build(&mut ctx)
        }).collect()
    }

    fn layout_proxies(&self) -> Vec<LayoutProxy<Msg, Self>> {
        todo!()
    }

    fn rebuild(&mut self, state: &mut Self::State, ctx: &mut BuildContext) {
        let mut child_ctx_iter = ctx.child_iter();
        todo!()
    }

    fn event(&mut self, state: &mut Self::State, event: Event, ctx: &mut EventContext<Msg>) {
        for ((view, state), mut ctx) in self.iter_mut().rev().zip(state.iter_mut().rev()).zip(ctx.child_iter().rev()) {
            view.event(state, event, &mut ctx);
        }
    }

    fn layout(&self, state: &mut Self::State, constraint: Constraint, ctx: &mut LayoutContext) -> Vec<Size> {
        let mut child_ctx_iter = ctx.child_iter();
        todo!()
    }

    fn render(&self, state: &Self::State, ctx: &mut RenderContext) {
        todo!()
    }

    fn len(&self) -> usize {
        self.len()
    }
}