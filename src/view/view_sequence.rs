use std::{any::Any, cell::Cell, collections::HashMap, hash::Hash, ops::Deref, rc::Rc};

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

        let f = self.view_factory;
		cx.track(self.count, move |value, mut widget| {
            if widget.child_count() < value {
                for i in widget.child_count()..value {
                    widget.add_child_with(|cx| f(cx, i));
				}
            } else if value < widget.child_count() {
                for i in value..widget.child_count() {
                    widget.remove_child(i);
				}
            }
		});
	}
}

pub struct ForEach<C, F, FKey> {
	values: Accessor<C>,
	view_fn: F,
	key_fn: FKey
}

impl<C, K, T, V, F, FKey> ViewSequence for ForEach<C, F, FKey> 
where 
	C: 'static,
	for <'a> &'a C: IntoIterator<Item = &'a T>,
	K: Hash + Eq + 'static,
	T: Clone + 'static,
	V: View,
	F: Fn(&mut ViewContext, &T) -> V + 'static,
	FKey: Fn(&T) -> K
{
    fn build_seq<W: Widget>(self, cx: &mut BuildContext<W>) {
		let items: Vec<_> = self.values.with_ref(cx, |values| {
			values.into_iter().map(T::clone).collect()
		});

		cx.track_mapped(self.values, 
			|values| { values.into_iter().map(T::clone).collect::<Vec<T>>() },
			|values, mut widget| {

			});

		/*
		view_indices.insert((self.key_fn)(&value), i);
				cx.add_child_with(|cx| (self.view_fn)(cx, value)); */
		
		/*let view_indices = Cell::new(Some(view_indices));
		cx.track(self.values, move |values, mut widget| {
			let old_view_indices = view_indices.take();


		});*/
    }
}

pub fn for_each_keyed<C, K, T, V, F, FKey>(
	values: impl Into<Accessor<C>>,
	key_fn: FKey,
	view_fn: F
) -> ForEach<C, F, FKey> 
where 
	K: Hash + Eq,
	T: Any,
	C: IntoIterator<Item = T>,
	V: View,
	F: Fn(&mut ViewContext, T) -> V + 'static,
	FKey: Fn(&T) -> K
{
	ForEach {
		values: values.into(),
		key_fn,
		view_fn
	}
}


pub enum VecDiff<T> {
	Removed { index: usize, len: usize},
	Changed { index: usize, new_value: T },
	Inserted { index: usize, value: T }
}

/*fn diff_slices<T: Eq>(a: &[T], b: &[T]) {
	let (mut a_start, mut a_end) = (0, a.len());
	let (mut b_start, mut b_end) = (0, b.len());

	while a_start < a_end && b_start < b_end {
		// Matching prefix
		if a[a_start] == b[b_start] {
			a_start += 1;
			b_start += 1;
		} else if a[a_end - 1] == b[b_end - 1] {
			a_end -= 1;
			b_end -= 1;
		} else if 
	}
}*/