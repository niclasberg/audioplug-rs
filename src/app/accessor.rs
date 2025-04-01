use std::rc::Rc;

use super::{
    effect::BindingState, signal::ReadSignal, Brush, BuildContext, EffectState, LinearGradient,
    Mapped, Memo, NodeId, Owner, ParamSignal, ReactiveContext, ReadContext, Readable, Signal,
    Widget, WidgetMut,
};
use crate::{core::Color, param::ParameterId};

pub trait MappedAccessor<T> {
    fn get_source_id(&self) -> SourceId;
    fn evaluate(&self, ctx: &mut dyn ReadContext) -> T;
}

#[derive(Clone, Copy)]
pub enum SourceId {
    Parameter(ParameterId),
    Node(NodeId),
}

type ComputedFn<T> = dyn Fn(&mut dyn ReadContext) -> T;

#[derive(Clone)]
pub struct Computed<T> {
    f: Rc<ComputedFn<T>>,
}

impl<T> Computed<T> {
    pub fn new(f: impl Fn(&mut dyn ReadContext) -> T + 'static) -> Self {
        Self { f: Rc::new(f) }
    }
}

#[derive(Clone)]
pub enum Accessor<T> {
    Signal(Signal<T>),
    ReadSignal(ReadSignal<T>),
    Memo(Memo<T>),
    Parameter(ParamSignal<T>),
    Const(T),
    Mapped(Rc<dyn MappedAccessor<T>>),
    Computed(Computed<T>),
}

impl<T: 'static> Accessor<T> {
    pub fn get(&self, cx: &mut dyn ReadContext) -> T
    where
        T: Clone,
    {
        match self {
            Self::Signal(signal) => signal.get(cx),
            Self::ReadSignal(signal) => signal.get(cx),
            Self::Memo(memo) => memo.get(cx),
            Self::Const(value) => value.clone(),
            Self::Parameter(param) => param.get(cx),
            Self::Mapped(mapped) => mapped.evaluate(cx),
            Self::Computed(computed) => (computed.f)(cx),
        }
    }

    pub fn with_ref<R>(&self, cx: &mut dyn ReadContext, f: impl FnOnce(&T) -> R) -> R {
        match self {
            Self::Signal(signal) => signal.with_ref(cx, f),
            Self::ReadSignal(signal) => signal.with_ref(cx, f),
            Self::Memo(memo) => memo.with_ref(cx, f),
            Self::Const(value) => f(value),
            Self::Parameter(param) => param.with_ref(cx, f),
            Self::Mapped(mapped) => f(&mapped.evaluate(cx)),
            Self::Computed(computed) => f(&(computed.f)(cx)),
        }
    }

    pub fn get_and_bind<W: Widget>(
        self,
        cx: &mut BuildContext<W>,
        f: impl Fn(T, WidgetMut<'_, W>) + 'static,
    ) -> T
    where
        T: Clone,
    {
        self.get_and_bind_mapped(cx, T::clone, f)
    }

    pub fn get_and_bind_mapped<W: Widget, U: 'static>(
        self,
        cx: &mut BuildContext<W>,
        f_map: fn(&T) -> U,
        f: impl Fn(U, WidgetMut<'_, W>) + 'static,
    ) -> U {
        let value = self.with_ref(cx, f_map);
        self.bind_mapped(cx, f_map, f);
        value
    }

    pub fn bind<W: Widget>(
        self,
        cx: &mut BuildContext<W>,
        f: impl Fn(T, WidgetMut<'_, W>) + 'static,
    ) where
        T: Clone,
    {
        self.bind_mapped(cx, T::clone, f);
    }

    pub fn bind_mapped<W: Widget, U: 'static, F>(
        self,
        cx: &mut BuildContext<W>,
        f_map: fn(&T) -> U,
        f: F,
    ) where
        F: Fn(U, WidgetMut<'_, W>) + 'static,
    {
        let create_binding = move |_self: Self, f: F, cx: &mut BuildContext<W>, source_id| {
            let widget_id = cx.id();
            let state = BindingState::new(move |app_state| {
                let value = _self.with_ref(app_state, f_map);
                // Widget might have been removed
                if app_state.widgets.contains_key(widget_id.id) {
                    let node = WidgetMut::new(app_state, widget_id.id);
                    f(value, node);
                }
                if app_state.widgets.contains_key(widget_id.id) {
                    app_state.merge_widget_flags(widget_id.id);
                }
            });

            cx.runtime_mut().create_binding_node(
                source_id,
                state,
                Some(Owner::Widget(widget_id.id)),
            );
        };

        match self {
            Self::Signal(signal) => create_binding(self, f, cx, SourceId::Node(signal.id)),
            Self::ReadSignal(signal) => create_binding(self, f, cx, SourceId::Node(signal.id)),
            Self::Memo(memo) => create_binding(self, f, cx, SourceId::Node(memo.id)),
            Self::Parameter(param) => create_binding(self, f, cx, SourceId::Parameter(param.id)),
            Self::Mapped(ref mapped) => {
                let id = mapped.get_source_id();
                create_binding(self, f, cx, id)
            }
            Self::Computed(computed) => {
                let value_fn = computed.f.clone();
                let widget_id = cx.id();
                let state = EffectState::new(move |cx| {
                    let value = f_map(&value_fn(cx));
                    let widget = cx.widget_mut(widget_id);
                    f(value, widget);
                });
                cx.runtime_mut().create_effect_node(
                    state,
                    Some(Owner::Widget(widget_id.id)),
                    false,
                );
            }
            Accessor::Const(_) => {}
        }
    }
}

impl<T: Default> Default for Accessor<T> {
    fn default() -> Self {
        Self::Const(T::default())
    }
}

impl<T> From<T> for Accessor<T> {
    fn from(value: T) -> Self {
        Self::Const(value)
    }
}

impl<T> From<Signal<T>> for Accessor<T> {
    fn from(value: Signal<T>) -> Self {
        Self::Signal(value)
    }
}

impl<T> From<ReadSignal<T>> for Accessor<T> {
    fn from(value: ReadSignal<T>) -> Self {
        Self::ReadSignal(value)
    }
}

impl<T> From<Memo<T>> for Accessor<T> {
    fn from(value: Memo<T>) -> Self {
        Self::Memo(value)
    }
}

impl<T> From<ParamSignal<T>> for Accessor<T> {
    fn from(value: ParamSignal<T>) -> Self {
        Self::Parameter(value)
    }
}

impl<S, T, R, F> From<Mapped<S, T, R, F>> for Accessor<R>
where
    S: Readable,
    Mapped<S, T, R, F>: MappedAccessor<R> + 'static,
{
    fn from(value: Mapped<S, T, R, F>) -> Self {
        Self::Mapped(Rc::new(value))
    }
}

impl From<&str> for Accessor<String> {
    fn from(value: &str) -> Self {
        Self::Const(value.to_string())
    }
}

impl From<Color> for Accessor<Brush> {
    fn from(value: Color) -> Self {
        Self::Const(value.into())
    }
}

impl From<LinearGradient> for Accessor<Brush> {
    fn from(value: LinearGradient) -> Self {
        Self::Const(value.into())
    }
}

impl<T> From<Computed<T>> for Accessor<T> {
    fn from(value: Computed<T>) -> Self {
        Self::Computed(value)
    }
}
