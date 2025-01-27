use std::rc::Rc;

use crate::param::ParameterId;
use super::{BindingState, BuildContext, Mapped, Memo, NodeId, ParamSignal, ReactiveContext, ReadContext, Readable, Runtime, Signal, Widget, WidgetMut};

pub trait MappedAccessor<T> {
    fn get_source_id(&self) -> SourceId;
    fn evaluate(&self, ctx: &mut dyn ReadContext) -> T;
}

#[derive(Clone, Copy)]
pub enum SourceId {
	Parameter(ParameterId),
	Node(NodeId)
}

#[derive(Clone)]
pub enum Accessor<T> {
    Signal(Signal<T>),
    Memo(Memo<T>),
	Parameter(ParamSignal<T>),
    Const(T),
    Mapped(Rc<dyn MappedAccessor<T>>)
}

impl<T: 'static> Accessor<T> {
    pub fn get_source_id(&self) -> Option<SourceId> {
		match self {
			Accessor::Signal(signal) => Some(SourceId::Node(signal.id)),
            Accessor::Memo(memo) => Some(SourceId::Node(memo.id)),
			Accessor::Parameter(param) => Some(SourceId::Parameter(param.id)),
            Accessor::Const(_) => None,
            Accessor::Mapped(mapped) => Some(mapped.get_source_id())
		}
	}

	pub fn get(&self, cx: &mut dyn ReadContext) -> T where T: Clone {
		match self {
            Accessor::Signal(signal) => signal.get(cx),
            Accessor::Memo(memo) => memo.get(cx),
            Accessor::Const(value) => value.clone(),
			Accessor::Parameter(param) => param.get(cx),
            Accessor::Mapped(mapped) => mapped.evaluate(cx)
        }
	}

    pub fn with_ref<R>(&self, cx: &mut dyn ReadContext, f: impl FnOnce(&T) -> R) -> R {
        match self {
            Accessor::Signal(signal) => signal.with_ref(cx, f),
            Accessor::Memo(memo) => memo.with_ref(cx, f),
            Accessor::Const(value) => f(value),
			Accessor::Parameter(param) => param.with_ref(cx, f),
            Accessor::Mapped(mapped) => f(&mapped.evaluate(cx))
        }
    }

	pub fn get_and_track<W: Widget>(self, cx: &mut BuildContext<W>, f: impl Fn(T, WidgetMut<'_, W>) + 'static) -> T where T: Clone{
        self.get_and_track_mapped(cx, T::clone, f)
    }

	pub fn get_and_track_mapped<W: Widget, U: 'static>(self, cx: &mut BuildContext<W>, f_map: fn(&T) -> U, f: impl Fn(U, WidgetMut<'_, W>) + 'static) -> U {
		let value = self.with_ref(cx, f_map);
		self.track_mapped(cx, f_map, f);
        value
	}

	pub fn track<W: Widget>(self, cx: &mut BuildContext<W>, f: impl Fn(T, WidgetMut<'_, W>) + 'static) where T: Clone{
		self.track_mapped(cx, T::clone, f);
	}

    pub fn track_mapped<W: Widget, U: 'static>(self, cx: &mut BuildContext<W>, f_map: fn(&T) -> U, f: impl Fn(U, WidgetMut<'_, W>) + 'static) {
		if let Some(source_id) = self.get_source_id() {
			let widget_id = cx.id();
            let state = BindingState::new(move |app_state| {
                let value = self.with_ref(app_state, f_map);
				// Widget might have been removed
				if app_state.widgets.contains_key(widget_id) {
					let node = WidgetMut::new(app_state, widget_id);
					f(value, node);
				}
				if app_state.widgets.contains_key(widget_id) {
					app_state.merge_widget_flags(widget_id);
				}
            });

            cx.runtime_mut().create_binding_node(source_id, state, Some(super::Owner::Widget(widget_id)));
		}
    }
}

impl<T> From<Signal<T>> for Accessor<T> {
    fn from(value: Signal<T>) -> Self {
        Self::Signal(value)
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
    Mapped<S, T, R, F>: MappedAccessor<R> + 'static
{
    fn from(value: Mapped<S, T, R, F>) -> Self {
        Self::Mapped(Rc::new(value))
    }
}

impl<T> From<T> for Accessor<T> {
    fn from(value: T) -> Self {
        Self::Const(value)
    }
}

impl From<&str> for Accessor<String> {
    fn from(value: &str) -> Self {
        Self::Const(value.to_string())
    }
}