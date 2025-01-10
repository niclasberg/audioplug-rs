use std::{any::Any, marker::PhantomData};

use crate::{param::{ParamRef, ParameterId}, style::Style, view::View};

use super::{effect::EffectState, signal::SignalContext, Accessor, AppState, Node, NodeId, ParamContext, Path, ReactiveContext, Signal, SignalCreator, Widget, WidgetFlags, WidgetId, WidgetMut};

pub struct BuildContext<'a, W: Widget> {
    id: WidgetId,
    pub(crate) app_state: &'a mut AppState,
    _phantom: PhantomData<W>
}

impl<'a, W: Widget> BuildContext<'a, W> {
    pub fn new(id: WidgetId, app_state: &'a mut AppState) -> Self {
        Self {
            id,
            app_state,
            _phantom: PhantomData
        }
    }

    pub fn get_and_track<T: Clone + 'static>(&mut self, accessor: Accessor<T>, f: impl Fn(T, WidgetMut<'_, W>) + 'static) -> T {
        let value = accessor.get(self.app_state);
		self.track(accessor, f);
        value
    }

	pub fn track<T: Clone + 'static>(&mut self, accessor: Accessor<T>, f: impl Fn(T, WidgetMut<'_, W>) + 'static) {
		self.app_state.create_binding(accessor, self.id, f);
	}

    pub fn id(&self) -> WidgetId {
        self.id
    }

    pub fn set_focusable(&mut self, focusable: bool) {
        if focusable {
            self.app_state.widget_data_mut(self.id).set_flag(WidgetFlags::FOCUSABLE)
        } else {
            self.app_state.widget_data_mut(self.id).clear_flag(WidgetFlags::FOCUSABLE)
        }
    }

    pub fn add_child<V: View>(&mut self, view: V) -> WidgetId {
        self.app_state.add_widget(self.id, move |cx| {
            Box::new(view.build(&mut BuildContext::new(cx.id, &mut cx.app_state)))
        })
    }

	pub fn add_child_with<V: View>(&mut self, view_factory: impl FnOnce(&mut ViewContext) -> V) -> WidgetId {
		self.app_state.add_widget(self.id, move |cx| {
            let view = view_factory(cx);
            Box::new(view.build(&mut BuildContext::new(cx.id, &mut cx.app_state)))
        })
	}

    pub(crate) fn build<V: View>(&mut self, view: V) -> V::Element {
        let mut ctx = BuildContext {
            id: self.id,
            app_state: self.app_state,
            _phantom: PhantomData
        };
        view.build(&mut ctx)
    }

	pub(crate) fn build_with<V: View>(&mut self, view_factory: impl FnOnce(&mut ViewContext) -> V) -> V::Element {
		let view = view_factory(&mut ViewContext::new(self.id, &mut self.app_state));
		self.build(view)
	}

	pub fn set_style(&mut self, style: Style) {
		self.app_state.widget_data_mut(self.id).style = style;
	}

	pub fn update_style(&mut self, f: impl FnOnce(&mut Style)) {
		f(&mut self.app_state.widget_data_mut(self.id).style);
	}

    pub(crate) fn as_view_context(self) -> ViewContext<'a> {
        ViewContext::new(self.id, self.app_state)
    }
}

impl<'s, W: Widget> ParamContext for BuildContext<'s, W> {
    fn host_handle(&self) -> &dyn super::HostHandle {
        self.app_state.host_handle()
    }
}

impl<'b, W: Widget> ReactiveContext for BuildContext<'b, W> {
    fn get_node_mut(&mut self, signal_id: NodeId) -> &mut Node {
        self.app_state.get_node_mut(signal_id)
    }
	
	fn get_parameter_ref(&self, parameter_id: ParameterId) -> ParamRef {
		self.app_state.get_parameter_ref(parameter_id)
	}
}

pub struct ViewContext<'a> {
    app_state: &'a mut AppState,
    id: WidgetId,
}

impl<'a> ViewContext<'a> {
    pub(super) fn new(id: WidgetId, app_state: &'a mut AppState) -> Self {
        Self { id, app_state }
    }

	pub fn app_state(&self) -> &AppState {
		&self.app_state
	}

    pub fn as_build_context<'b, W: Widget>(&'b mut self) -> BuildContext<'b, W> {
        BuildContext::new(self.id, self.app_state)
    }
}

impl<'a> SignalCreator for ViewContext<'a> {
    fn create_signal_node(&mut self, state: super::signal::SignalState) -> NodeId {
        self.app_state.create_signal_node(state)
    }

    fn create_effect_node(&mut self, state: EffectState) -> NodeId {
        let id = self.app_state.create_effect_node(state);
        // Run the effect directly
        self.app_state.run_effects();
        id
    }
    
    fn create_memo_node(&mut self, state: super::memo::MemoState) -> NodeId {
        self.app_state.create_memo_node(state)
    }
	
	fn create_trigger(&mut self) -> NodeId {
		self.app_state.create_trigger()
	}
}

impl<'b> ReactiveContext for ViewContext<'b> {
    fn get_node_mut(&mut self, signal_id: NodeId) -> &mut Node {
        self.app_state.get_node_mut(signal_id)
    }

	fn get_parameter_ref(&self, parameter_id: ParameterId) -> ParamRef {
		self.app_state.get_parameter_ref(parameter_id)
	}
}

impl<'b> SignalContext for ViewContext<'b> {
    fn notify(&mut self, node_id: NodeId) {
		self.app_state.notify(node_id);
	}
	
	fn get_or_insert_field_trigger(&mut self, node_id: NodeId, path: Path) -> super::Trigger {
		todo!()
	}
}


/*pub struct WidgetContext<'a> {
    widget_data: &'a mut WidgetData
}

impl<'a> WidgetContext<'a> {
    pub fn request_layout(&mut self) {
        self.widget_data.set_flag(ViewFlags::NEEDS_LAYOUT);
    }

    pub fn request_render(&mut self) {
        self.widget_data.set_flag(ViewFlags::NEEDS_RENDER);
    }
}*/


