use crate::{BuildContext, View, Widget, Id};

pub trait ViewSequence: Sized {
    fn len(&self) -> usize;
    fn build(self, ctx: &mut BuildContext) -> Vec<Box<dyn Widget>>;
}

macro_rules! impl_view_seq_tuple {
    ( $n: tt; $( $t: ident),* ; $( $s: tt),* ; $( $s_rev: tt),*) => {
        impl<$( $t: View, )*> ViewSequence for ($( $t, )*) {
            fn build(&mut self, ctx: &mut BuildContext) -> Vec<Box<dyn Widget>> {
                ctx.set_number_of_children($n);
                let mut widgets = Vec::new();
                (
                    $( widgets.push(Box::new(ctx.with_child(Id($s), |c| self.$s.build(c)))), )*
                );
                widgets
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

impl<V: View> ViewSequence for Vec<V> {
    fn build(&mut self, ctx: &mut BuildContext) -> Vec<Box<dyn Widget>> {
        /*ctx.set_number_of_children(self.len());
        self.iter_mut().zip(ctx.child_iter()).map(|(view, mut ctx)| {
            view.build(&mut ctx)
        }).collect()*/
        todo!()
    }

    fn len(&self) -> usize {
        self.len()
    }
}