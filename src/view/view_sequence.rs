use crate::app::{BuildContext, Widget};

use super::View;

pub trait ViewSequence: Sized {
    fn len(&self) -> usize;
    fn build<W: Widget>(self, ctx: &mut BuildContext<W>);
}

macro_rules! impl_view_seq_tuple {
    ( $n: tt; $( $t: ident),* ; $( $s: tt),*) => {
        impl<$( $t: View, )*> ViewSequence for ($( $t, )*) {
            fn build<W: Widget>(self, ctx: &mut BuildContext<W>) {
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

impl_view_seq_tuple!( 1; V; 0);
impl_view_seq_tuple!( 2; V1, V2; 0, 1);
impl_view_seq_tuple!( 3; V1, V2, V3; 0, 1, 2);
impl_view_seq_tuple!( 4; V1, V2, V3, V4; 0, 1, 2, 3);
impl_view_seq_tuple!( 5; V1, V2, V3, V4, V5; 0, 1, 2, 3, 4);
impl_view_seq_tuple!( 6; V1, V2, V3, V4, V5, V6; 0, 1, 2, 3, 4, 5);
impl_view_seq_tuple!( 7; V1, V2, V3, V4, V5, V6, V7; 0, 1, 2, 3, 4, 5, 6);
impl_view_seq_tuple!( 8; V1, V2, V3, V4, V5, V6, V7, V8; 0, 1, 2, 3, 4, 5, 6, 7);
impl_view_seq_tuple!( 9; V1, V2, V3, V4, V5, V6, V7, V8, V9; 0, 1, 2, 3, 4, 5, 6, 7, 8);
impl_view_seq_tuple!(10; V1, V2, V3, V4, V5, V6, V7, V8, V9, V10; 0, 1, 2, 3, 4, 5, 6, 7, 8, 9);
impl_view_seq_tuple!(11; V1, V2, V3, V4, V5, V6, V7, V8, V9, V10, V11; 0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10);
impl_view_seq_tuple!(12; V1, V2, V3, V4, V5, V6, V7, V8, V9, V10, V11, V12; 0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11);

impl<V: View> ViewSequence for Vec<V> {
    fn build<W: Widget>(self, ctx: &mut BuildContext<W>) {
        for child in self {
            ctx.add_child(child);
        }
    }

    fn len(&self) -> usize {
        self.len()
    }
}