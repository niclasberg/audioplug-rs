use crate::{View, IdPath, Id, view::AnyView};

use super::contexts::Context;

pub trait ViewVisitor<Msg, C: Context> {
    fn visit<V: View<Message = Msg>>(&mut self, context: &mut C, view: &mut V, state: &mut V::State);
}

pub trait ViewSequence<Msg> {
    type State;

    fn for_each<C: Context>(&mut self, context: &mut C, state: &mut Self::State, f: &mut impl ViewVisitor<Msg, C>);
    fn len(&self) -> usize;

    fn build(&mut self, id_path: &IdPath) -> Self::State;
}

impl<Msg, V: View<Message = Msg>> ViewSequence<Msg> for V {
    type State = V::State;

    fn build(&mut self, id_path: &IdPath) -> Self::State {
        <Self>::build(self, id_path)
    }

    fn for_each<C: Context>(&mut self, context: &mut C, state: &mut Self::State, f: &mut impl ViewVisitor<Msg, C>) {
        f.visit(context, self, state)
    }

    fn len(&self) -> usize {
        1
    }
}

impl<Msg, V1: View<Message = Msg>, V2: View<Message = Msg>> ViewSequence<Msg> for (V1, V2) {
    type State = (V1::State, V2::State);

    fn build(&mut self, id_path: &IdPath) -> Self::State {
        let mut id_path = id_path.clone();
        (
            id_path.with_child_id(Id(0), |id_path| self.0.build(&id_path)),
            id_path.with_child_id(Id(1), |id_path| self.1.build(&id_path))
        )
    }

    fn for_each<C: Context>(&mut self, context: &mut C, state: &mut Self::State, f: &mut impl ViewVisitor<Msg, C>) {
        context.with_child(Id(0), |c| f.visit(c, &mut self.0, &mut state.0));
        context.with_child(Id(1), |c| self.1.for_each(c, &mut state.1, f));
    }

    fn len(&self) -> usize {
        self.0.len() + self.1.len()
    }
}

/*impl<Msg, V: ViewSequence<Msg>> ViewSequence<Msg> for Option<VS> {
    type State = Option<VS::State>;

    fn fold<T>(&self, id_path: &IdPath, init: T, f: &impl FnMut(T, &IdPath, &dyn AnyView<Msg>) -> T) -> T {
        match self {
            Some(vs) => vs.fold(id_path, init, f),
            None => init
        }
    }

    fn build(&self, id_path: &IdPath) -> Self::State {
        self.as_ref().map(|vs| vs.build(id_path))
    }

    fn for_each<C: Context>(&mut self, context: &mut C, state: &mut Self::State, f: &impl ViewVisitor<Msg, C>) {
        
    }

    fn len(&self) -> usize {
        if self.is_some() { 1 } else { 0 }
    }
}*/

impl<Msg, V: View<Message = Msg>> ViewSequence<Msg> for Vec<V> {
    type State = Vec<V::State>;

    fn build(&mut self, id_path: &IdPath) -> Self::State {
        let mut id_path = id_path.clone();
        self.iter_mut().enumerate().map(|(i, vs)| {
            id_path.with_child_id(Id(i), |id_path| vs.build(id_path))
        }).collect()
    }

    fn for_each<C: Context>(&mut self, context: &mut C, state: &mut Self::State, f: &mut impl ViewVisitor<Msg, C>) {
        for (i, (v, state)) in self.iter_mut().zip(state.iter_mut()).enumerate() {
            context.with_child(Id(i), |c| f.visit(c, v, state));
        }
    }

    fn len(&self) -> usize {
        self.iter().fold(0, |len, view| { len + view.len() })
    }
}