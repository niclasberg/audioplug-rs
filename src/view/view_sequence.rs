use crate::Id;

use super::{BuildContext, View, Widget, WidgetNode};

pub trait ViewSequence: Sized {
    fn len(&self) -> usize;
    fn build(self, ctx: &mut BuildContext);
}

macro_rules! impl_view_seq_tuple {
    ( $n: tt; $( $t: ident),* ; $( $s: tt),*) => {
        impl<$( $t: View, )*> ViewSequence for ($( $t, )*) {
            fn build(self, ctx: &mut BuildContext) {
                (
                    $( ctx.add_child(self.$s), )*
                );
            }
        
            fn len(&self) -> usize {
                $n
            }
        }
    }
}

impl_view_seq_tuple!(1; V; 0);
impl_view_seq_tuple!(2; V1, V2; 0, 1);
impl_view_seq_tuple!(3; V1, V2, V3; 0, 1, 2);
impl_view_seq_tuple!(4; V1, V2, V3, V4; 0, 1, 2, 3);
impl_view_seq_tuple!(5; V1, V2, V3, V4, V5; 0, 1, 2, 3, 4);
impl_view_seq_tuple!(6; V1, V2, V3, V4, V5, V6; 0, 1, 2, 3, 4, 5);

impl<V: View> ViewSequence for Vec<V> {
    fn build(self, ctx: &mut BuildContext) {
        for child in self {
            ctx.add_child(child);
        }
    }

    fn len(&self) -> usize {
        self.len()
    }
}