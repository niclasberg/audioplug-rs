use crate::app::{diff::DiffOp, Effect, ReadContext};

use super::{
    effect::BindingState, Accessor, BuildContext, Owner, ReactiveContext, Readable, View, Widget,
};
use std::{collections::HashMap, hash::Hash};

pub trait ViewSequence: Sized + 'static {
    fn build_seq(self, cx: &mut BuildContext<dyn Widget>);
}

impl<V: View + Sized> ViewSequence for V {
    fn build_seq(self, cx: &mut BuildContext<dyn Widget>) {
        cx.add_child(self);
    }
}

macro_rules! impl_view_seq_tuple {
    ($( $t: ident),* ; $( $s: tt),*) => {
        impl<$( $t: ViewSequence, )*> ViewSequence for ($( $t, )*) {
            fn build_seq(self, cx: &mut BuildContext<dyn Widget>) {
                (
                    $( self.$s.build_seq(cx), )*
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
    fn build_seq(self, cx: &mut BuildContext<dyn Widget>) {
        for child in self {
            cx.add_child(child);
        }
    }
}

impl<V: View> ViewSequence for Option<V> {
    fn build_seq(self, cx: &mut BuildContext<dyn Widget>) {
        if let Some(view) = self {
            cx.add_child(view);
        }
    }
}

impl<const N: usize, V: View> ViewSequence for [V; N] {
    fn build_seq(self, cx: &mut BuildContext<dyn Widget>) {
        for child in self {
            cx.add_child(child);
        }
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
    fn build_seq(self, cx: &mut BuildContext<dyn Widget>) {
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
                        widget.push_child(f(i));
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

struct ViewForEach<FValues, FView> {
    values_fn: FValues,
    view_fn: FView,
}

impl<T, C, V, FValues, FView> ViewSequence for ViewForEach<FValues, FView>
where
    FValues: Fn(&mut dyn ReadContext) -> C + 'static,
    C: IntoIterator<Item = T>,
    T: PartialEq + Clone + 'static,
    FView: Fn(&T) -> V + 'static,
    V: View,
{
    fn build_seq(self, cx: &mut BuildContext<dyn Widget>) {
        let id = cx.id().into_any_widget_id();
        Effect::new_with_state(cx, move |cx, old_values: Option<Vec<T>>| {
            let new_values: Vec<T> = (self.values_fn)(cx).into_iter().collect();
            let mut widget = cx.widget_mut(id);
            if let Some(old_values) = old_values {
                for diff in super::diff::diff_slices(old_values.as_slice(), new_values.as_slice()) {
                    match diff {
                        DiffOp::Remove { index, len } => {
                            for i in 0..len {
                                widget.remove_child(index + i)
                            }
                        }
                        DiffOp::Replace {
                            index,
                            source_index,
                        } => widget.replace_child(index, (self.view_fn)(&new_values[source_index])),
                        DiffOp::Insert {
                            index,
                            source_index,
                            len,
                        } => {
                            for i in 0..len {
                                widget.insert_child(
                                    (self.view_fn)(&new_values[i + source_index]),
                                    index,
                                );
                            }
                        }
                        DiffOp::Move { from, to } => widget.swap_children(from, to),
                    }
                }
            } else {
                for value in new_values.iter() {
                    widget.push_child((self.view_fn)(value));
                }
            }
            new_values
        });
    }
}

pub fn view_for_each<T, C, V, FValues, FView>(
    values_fn: FValues,
    view_fn: FView,
) -> impl ViewSequence
where
    FValues: Fn(&mut dyn ReadContext) -> C + 'static,
    C: IntoIterator<Item = T>,
    T: PartialEq + Clone + 'static,
    FView: Fn(&T) -> V + 'static,
    V: View,
{
    ViewForEach { values_fn, view_fn }
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
    fn build_seq(self, cx: &mut BuildContext<dyn Widget>) {
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
            let values = signal.with_ref_untracked(cx, |values| {
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
