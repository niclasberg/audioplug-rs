use crate::app::{diff::DiffOp, Effect, ReadableSeq};

use super::{
    effect::BindingState, Accessor, BuildContext, NodeId, Owner, ReactiveContext, Readable, View,
    Widget,
};
use std::{cell::Cell, collections::HashMap, hash::Hash, ops::Range};

pub trait ViewSequence: Sized + 'static {
    fn build_seq<W: Widget>(self, cx: &mut BuildContext<W>);
}

impl<V: View + Sized> ViewSequence for V {
    fn build_seq<W: Widget>(self, cx: &mut BuildContext<W>) {
        cx.add_child(self);
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
    fn build_seq<W: Widget>(self, cx: &mut BuildContext<W>) {
        for child in self {
            cx.add_child(child);
        }
    }
}

impl<V: View> ViewSequence for Option<V> {
    fn build_seq<W: Widget>(self, cx: &mut BuildContext<W>) {
        if let Some(view) = self {
            cx.add_child(view);
        }
    }
}

impl<const N: usize, V: View> ViewSequence for [V; N] {
    fn build_seq<W: Widget>(self, cx: &mut BuildContext<W>) {
        for child in self {
            cx.add_child(child);
        }
    }
}

pub trait MapToViews {
    type Element;
    fn map_to_views<V: View, F: Fn(&Self::Element) -> V + 'static>(self, f: F)
        -> impl ViewSequence;
}

struct MapArrayToViews<R, F> {
    readable: R,
    f: F,
}

impl<const N: usize, T, R> MapToViews for R
where
    T: PartialEq + Clone + 'static,
    R: Readable<Value = [T; N]> + 'static,
{
    type Element = T;

    fn map_to_views<V: View, F: Fn(&Self::Element) -> V + 'static>(
        self,
        f: F,
    ) -> impl ViewSequence {
        MapArrayToViews { readable: self, f }
    }
}

impl<const N: usize, T, V, R, F> ViewSequence for MapArrayToViews<R, F>
where
    T: PartialEq + Clone + 'static,
    R: Readable<Value = [T; N]> + 'static,
    F: Fn(&T) -> V + 'static,
    V: View,
{
    fn build_seq<W: Widget>(self, cx: &mut BuildContext<W>) {
        let values = self.readable.get(cx);

        for value in values.iter() {
            cx.add_child((self.f)(value));
        }

        let id = cx.id();
        let old_values = values;
        Effect::watch(cx, self.readable, move |cx, values| {
            let mut widget = cx.widget_mut(id);
            for i in 0..N {
                if values[i] != old_values[i] {}
            }
        });
    }
}

pub struct MapToViews2<R, F> {
    readable_seq: R,
    f_map: F,
}

impl<V, R, F> ViewSequence for MapToViews2<R, F>
where
    V: View,
    R: 'static + ReadableSeq,
    R::Value: 'static,
    F: Fn(&R::Element) -> V + 'static,
{
    fn build_seq<W: Widget>(self, cx: &mut BuildContext<W>) {
        let views: Vec<V> = self
            .readable_seq
            .with_ref_iter(cx, |iter| iter.map(self.f_map).collect());

        for view in views {
            cx.add_child(view);
        }

        let id = cx.id();
        self.readable_seq.subscribe_seq_diff(cx, move |cx, diff| {
            let mut widget_mut = cx.widget_mut(id);
            match diff {
                DiffOp::Remove { index, len } => {
                    for i in *index..(*index + *len) {
                        widget_mut.remove_child(i);
                    }
                }
                DiffOp::Change { index, new_value } => todo!(),
                DiffOp::Insert { index, values } => todo!(),
                DiffOp::Move {
                    old_index,
                    new_index,
                } => todo!(),
            }
        });
    }
}

pub struct ForRange<Idx, F> {
    start: Accessor<Idx>,
    end: Accessor<Idx>,
    view_fn: F,
}

impl<Idx, V, F> ViewSequence for ForRange<Idx, F>
where
    Idx: num::Integer + Clone + 'static,
    V: View,
    F: Fn(Idx) -> V + 'static,
{
    fn build_seq<W: Widget>(self, cx: &mut BuildContext<W>) {
        let mut start = self.start.get(cx);
        let mut end = self.end.get(cx);
        //let mut ids = Vec::new();
        let view_fn = self.view_fn;
        let mut i = start;
        /*while i != end {
            let id = cx.add_child(view_fn(i));
            ids.push(id);
            if start < end {
                i = i.add(Idx::one());
            } else {
                i = i.sub(Idx::one());
            };
        }*/
    }
}

pub struct IndexedViewSeq<F> {
    count: Accessor<usize>,
    view_factory: F,
}

impl<V: View, F: Fn(usize) -> V> IndexedViewSeq<F> {
    pub fn new(count: impl Into<Accessor<usize>>, view_factory: F) -> Self {
        Self {
            count: count.into(),
            view_factory,
        }
    }
}

impl<V: View, F: Fn(usize) -> V + 'static> ViewSequence for IndexedViewSeq<F> {
    fn build_seq<W: Widget>(self, cx: &mut BuildContext<W>) {
        let child_count = self.count.get(cx);
        for i in 0..child_count {
            cx.add_child((self.view_factory)(i));
        }

        let f = self.view_factory;
        self.count.bind(cx, move |value, mut widget| {
            match widget.child_count().cmp(&value) {
                std::cmp::Ordering::Equal => {}
                std::cmp::Ordering::Less => {
                    for i in widget.child_count()..value {
                        widget.add_child(f(i));
                    }
                }
                std::cmp::Ordering::Greater => {
                    for i in value..widget.child_count() {
                        widget.remove_child(i);
                    }
                }
            }
        });
    }
}

pub struct RangeForViews<S, F> {
    signal: S,
    f: F,
}

impl<Idx, S, F> ViewSequence for RangeForViews<S, F>
where
    S: Readable<Value = Range<Idx>> + 'static,
    F: 'static,
{
    fn build_seq<W: Widget>(self, ctx: &mut BuildContext<W>) {}
}

pub struct ForEach<T, F> {
    values: Accessor<Vec<T>>,
    view_fn: F,
}

impl<T, V, F> ViewSequence for ForEach<T, F>
where
    T: Eq + Clone + 'static,
    V: View,
    F: Fn(Accessor<T>) -> V + 'static,
{
    fn build_seq<W: Widget>(self, ctx: &mut BuildContext<W>) {
        let triggers: Vec<NodeId> = Vec::new();
    }
}

pub struct ForEachKeyed<S, F, FKey> {
    signal: S,
    view_fn: F,
    key_fn: FKey,
}

impl<S, C, K, T, V, F, FKey> ViewSequence for ForEachKeyed<S, F, FKey>
where
    S: Readable<Value = C> + 'static,
    C: 'static,
    for<'a> &'a C: IntoIterator<Item = &'a T>,
    K: Hash + Eq + 'static,
    T: Clone + 'static,
    V: View,
    F: Fn(T) -> V + 'static,
    FKey: Fn(&T) -> K + 'static,
{
    fn build_seq<W: Widget>(self, cx: &mut BuildContext<W>) {
        let values = self.signal.with_ref(cx, |values| {
            values.into_iter().map(T::clone).collect::<Vec<T>>()
        });
        let mut indices = HashMap::new();
        for (i, value) in values.into_iter().enumerate() {
            indices.insert((self.key_fn)(&value), i);
            cx.add_child((self.view_fn)(value));
        }

        let signal = self.signal;
        let source_id = signal.get_source_id();
        let widget_id = cx.id();
        let state = BindingState::new(move |cx| {
            let values = signal.with_ref(cx, |values| {
                values.into_iter().map(T::clone).collect::<Vec<T>>()
            });
            let mut new_indices: HashMap<_, _> = values
                .iter()
                .enumerate()
                .map(|(i, value)| ((self.key_fn)(value), i))
                .collect();

            //cx.add_widget(widget_id, widget_factory)
            //cx.remove_widget(id);

            std::mem::swap(&mut indices, &mut new_indices);
        });

        cx.runtime_mut()
            .create_binding_node(source_id, state, Some(Owner::Widget(widget_id.id)));

        /*
        view_indices.insert((self.key_fn)(&value), i);
                cx.add_child_with(|cx| (self.view_fn)(cx, value)); */

        /*let view_indices = Cell::new(Some(view_indices));
        cx.track(self.values, move |values, mut widget| {
            let old_view_indices = view_indices.take();


        });*/
    }
}

pub fn for_each_keyed<S, C, K, T, V, F, FKey>(
    signal: S,
    key_fn: FKey,
    view_fn: F,
) -> ForEachKeyed<S, F, FKey>
where
    S: Readable<Value = C> + 'static,
    C: 'static,
    for<'a> &'a C: IntoIterator<Item = &'a T>,
    K: Hash + Eq + 'static,
    T: Clone + 'static,
    V: View,
    F: Fn(T) -> V + 'static,
    FKey: Fn(&T) -> K + 'static,
{
    ForEachKeyed {
        signal,
        key_fn,
        view_fn,
    }
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
