use rustc_hash::FxBuildHasher;

use super::reactive::{Effect, ReactiveValue, ReadContext};
use super::{BuildContext, View, ViewProp, Widget};
use crate::core::{FxIndexSet, diff};
use std::hash::Hash;

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
    count: ViewProp<usize>,
    view_factory: F,
}

impl<V: View, F: Fn(usize) -> V> IndexedViewSeq<F> {
    pub fn new(count: impl Into<ViewProp<usize>>, view_factory: F) -> Self {
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
            let mut child_index = 0;
            widget.for_each_child_mut(|child| {
                if child_index >= value {
                    child.remove();
                }
                child_index += 1;
            });
            for i in child_index..value {
                widget.push_child_back(f(i));
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
    T: PartialEq + 'static,
    FView: Fn(&T) -> V + 'static,
    V: View,
{
    fn build_seq(self, cx: &mut BuildContext<dyn Widget>) {
        let id = cx.id().into_any_widget_id();
        Effect::new_with_state(cx, move |cx, old_values: Option<Vec<T>>| {
            let new_values: Vec<T> = (self.values_fn)(cx).into_iter().collect();
            let mut widget = cx.widget_mut(id);
            if let Some(old_values) = old_values {
                for diff in diff::diff_slices(old_values.as_slice(), new_values.as_slice()) {
                    widget.apply_diff_to_children(diff, &self.view_fn);
                }
            } else {
                for value in new_values.iter() {
                    widget.push_child_back((self.view_fn)(value));
                }
            }
            widget.request_render();
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
    T: PartialEq + 'static,
    FView: Fn(&T) -> V + 'static,
    V: View,
{
    ViewForEach { values_fn, view_fn }
}

pub trait ReactiveValueExt: ReactiveValue {
    fn map_to_views_keyed<T, K, V, FKey, FView>(
        self,
        key_fn: FKey,
        view_fn: FView,
    ) -> impl ViewSequence
    where
        for<'a> &'a Self::Value: IntoIterator<Item = &'a T>,
        K: Hash + Eq + 'static,
        T: 'static,
        V: View,
        FView: Fn(&T) -> V + 'static,
        FKey: Fn(&T) -> K + 'static;
}

impl<R> ReactiveValueExt for R
where
    R: ReactiveValue + 'static,
{
    fn map_to_views_keyed<T, K, V, FKey, FView>(
        self,
        key_fn: FKey,
        view_fn: FView,
    ) -> impl ViewSequence
    where
        for<'a> &'a Self::Value: IntoIterator<Item = &'a T>,
        K: Hash + Eq + 'static,
        T: 'static,
        V: View,
        FView: Fn(&T) -> V + 'static,
        FKey: Fn(&T) -> K + 'static,
    {
        MapToViewsKeyedImpl {
            readable: self,
            view_fn,
            key_fn,
        }
    }
}

struct MapToViewsKeyedImpl<R, F, FKey> {
    readable: R,
    view_fn: F,
    key_fn: FKey,
}

impl<S, C: 'static, K, T, V, F, FKey> ViewSequence for MapToViewsKeyedImpl<S, F, FKey>
where
    S: ReactiveValue<Value = C> + 'static,
    for<'a> &'a C: IntoIterator<Item = &'a T>,
    K: Hash + Eq + 'static,
    T: 'static,
    V: View,
    F: Fn(&T) -> V + 'static,
    FKey: Fn(&T) -> K + 'static,
{
    fn build_seq(self, cx: &mut BuildContext<dyn Widget>) {
        let views_keys: Vec<_> = self.readable.with_ref(cx, |values| {
            values
                .into_iter()
                .map(|value| ((self.key_fn)(value), (self.view_fn)(value)))
                .collect()
        });

        let mut old_indices = FxIndexSet::with_capacity_and_hasher(views_keys.len(), FxBuildHasher);
        for (key, view) in views_keys.into_iter() {
            old_indices.insert(key);
            cx.add_child(view);
        }

        let widget_id = cx.id();
        self.readable.watch(cx, move |cx, values| {
            let new_indices: FxIndexSet<_> = values.into_iter().map(|x| (self.key_fn)(x)).collect();
            let value_vec: Vec<_> = values.into_iter().collect();
            let mut widget = cx.widget_mut(widget_id);

            diff::diff_keyed_with(&old_indices, &new_indices, &value_vec, |diff| {
                let f = |x: &&T| (self.view_fn)(*x);
                widget.apply_diff_to_children(diff, &f)
            });
            widget.request_render();

            old_indices = new_indices;
        });
    }
}
