use std::{ops::Deref, rc::Rc};

use crate::app::{Accessor, BuildContext, ViewContext, Widget};

use super::View;

pub trait ViewSequence: Sized {
    fn build_seq<W: Widget>(self, ctx: &mut BuildContext<W>);
}

impl<V: View + Sized> ViewSequence for V {
    fn build_seq<W: Widget>(self, ctx: &mut BuildContext<W>) {
        ctx.add_child(self);
    }
}

macro_rules! impl_view_seq_tuple {
    ($( $t: ident),* ; $( $s: tt),*) => {
        impl<$( $t: ViewSequence, )*> ViewSequence for ($( $t, )*) {
            fn build_seq<W: Widget>(self, ctx: &mut BuildContext<W>) {
                (
                    $( self.$s.build_seq(ctx), )*
                );
            }
        }
    }
}

impl_view_seq_tuple!(V; 0);
impl_view_seq_tuple!(V1, V2; 0, 1);
impl_view_seq_tuple!(V1, V2, V3; 0, 1, 2);
impl_view_seq_tuple!(V1, V2, V3, V4; 0, 1, 2, 3);
impl_view_seq_tuple!(V1, V2, V3, V4, V5; 0, 1, 2, 3, 4);
impl_view_seq_tuple!(V1, V2, V3, V4, V5, V6; 0, 1, 2, 3, 4, 5);
impl_view_seq_tuple!(V1, V2, V3, V4, V5, V6, V7; 0, 1, 2, 3, 4, 5, 6);
impl_view_seq_tuple!(V1, V2, V3, V4, V5, V6, V7, V8; 0, 1, 2, 3, 4, 5, 6, 7);
impl_view_seq_tuple!(V1, V2, V3, V4, V5, V6, V7, V8, V9; 0, 1, 2, 3, 4, 5, 6, 7, 8);
impl_view_seq_tuple!(V1, V2, V3, V4, V5, V6, V7, V8, V9, V10; 0, 1, 2, 3, 4, 5, 6, 7, 8, 9);
impl_view_seq_tuple!(V1, V2, V3, V4, V5, V6, V7, V8, V9, V10, V11; 0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10);
impl_view_seq_tuple!(V1, V2, V3, V4, V5, V6, V7, V8, V9, V10, V11, V12; 0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11);

impl<V: View> ViewSequence for Vec<V> {
    fn build_seq<W: Widget>(self, ctx: &mut BuildContext<W>) {
        for child in self {
            ctx.add_child(child);
        }
    }
}

impl<V: View> ViewSequence for Option<V> {
    fn build_seq<W: Widget>(self, ctx: &mut BuildContext<W>) {
        if let Some(view) = self {
            ctx.add_child(view);
        }
    }
}

pub struct IndexedViewSeq<F> {
	count: Accessor<usize>,
	view_factory: F
}

impl<V: View, F: Fn(&mut ViewContext, usize) -> V> IndexedViewSeq<F> {
	pub fn new(count: impl Into<Accessor<usize>>, view_factory: F) -> Self {
		Self {
			count: count.into(),
			view_factory
		}
	}
}

impl<V: View, F: Fn(&mut ViewContext, usize) -> V + 'static> ViewSequence for IndexedViewSeq<F> {
	fn build_seq<W: Widget>(self, cx: &mut BuildContext<W>) {
		let child_count = self.count.get(cx);
		for i in 0..child_count {
			cx.add_child_with(|cx| (self.view_factory)(cx, i));
		}

        let f = Rc::new(self.view_factory);

		cx.track(self.count, move |value, mut widget| {
            if widget.child_count() < value {
                for i in widget.child_count()..value {
                    let f = f.clone();
                    widget.add_child_with(move |cx| (f.deref())(cx, i));
				}
            } else if value < widget.child_count() {
                for i in value..widget.child_count() {
                    widget.remove_child(i);
				}
            }
		});
	}
}

pub struct ForEach<T, F> {
	values: Accessor<Vec<T>>,
	view_factory: F
}

impl<T, V: View, F: Fn(&mut ViewContext, &T) -> V + 'static> ForEach<T, F> {
    pub fn new(values: impl Into<Accessor<Vec<T>>>, view_factory: F) -> Self {
        Self {
            values: values.into(),
            view_factory
        }
    }
}

impl<T, V: View, F: Fn(&mut ViewContext, &T) -> V + 'static> ViewSequence for ForEach<T, F> {
    fn build_seq<W: Widget>(self, ctx: &mut BuildContext<W>) {
        todo!()
    }
}