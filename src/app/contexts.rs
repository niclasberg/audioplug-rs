use std::{any::Any, marker::PhantomData};

use crate::{param::{ParamRef, ParameterId}, style::Style, view::View};

use super::{effect::EffectState, WidgetFlags, WidgetId, WidgetMut, Accessor, AppState, NodeId, ParamContext, Signal, SignalContext, SignalCreator, SignalGetContext, Widget};

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

    pub fn get_and_track<T: Clone + 'static>(&mut self, accessor: Accessor<T>, f: impl Fn(&T, WidgetMut<'_, W>) + 'static) -> T {
        let value = accessor.get(self.app_state);
		self.track(accessor, f);
        value
    }

	pub fn track<T: 'static>(&mut self, accessor: Accessor<T>, f: impl Fn(&T, WidgetMut<'_, W>) + 'static) {
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
        self.app_state.add_widget(self.id, view)
    }

	pub fn add_child_with<V: View>(&mut self, view_factory: impl FnOnce(&mut ViewContext) -> V) -> WidgetId {
        let view = view_factory(&mut ViewContext::new(&mut self.app_state));
		self.app_state.add_widget(self.id, view)
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
		let mut app_context = ViewContext {
			app_state: &mut self.app_state,
		};
		let view = view_factory(&mut app_context);
		self.build(view)
	}

	pub fn set_style(&mut self, style: Style) {
		self.app_state.widget_data_mut(self.id).style = style;
	}

	pub fn update_style(&mut self, f: impl FnOnce(&mut Style)) {
		f(&mut self.app_state.widget_data_mut(self.id).style);
	}

    pub(crate) fn as_view_context(self) -> ViewContext<'a> {
        ViewContext::new(self.app_state)
    }
}

impl<'s, W: Widget> ParamContext for BuildContext<'s, W> {
    fn host_handle(&self) -> &dyn super::HostHandle {
        self.app_state.host_handle()
    }
}

impl<'b, W: Widget> SignalGetContext for BuildContext<'b, W> {
    fn get_node_value_ref<'a>(&'a mut self, signal_id: NodeId) -> &'a dyn Any {
        self.app_state.get_node_value_ref(signal_id)
    }
	
	fn get_parameter_ref_untracked(&self, parameter_id: ParameterId) -> ParamRef {
		self.app_state.get_parameter_ref_untracked(parameter_id)
	}
	
	fn get_parameter_ref(&mut self, parameter_id: ParameterId) -> ParamRef {
		self.app_state.get_parameter_ref(parameter_id)
	}
}

pub struct ViewContext<'a> {
    app_state: &'a mut AppState,
}

impl<'a> ViewContext<'a> {
    pub(super) fn new(app_state: &'a mut AppState) -> Self {
        Self { app_state}
    }

	pub fn app_state(&self) -> &AppState {
		&self.app_state
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
}

impl<'b> SignalGetContext for ViewContext<'b> {
    fn get_node_value_ref<'a>(&'a mut self, signal_id: NodeId) -> &'a dyn Any {
        self.app_state.get_node_value_ref(signal_id)
    }
	
	fn get_parameter_ref_untracked(&self, parameter_id: ParameterId) -> ParamRef {
		self.app_state.get_parameter_ref_untracked(parameter_id)
	}
	
	fn get_parameter_ref(&mut self, parameter_id: ParameterId) -> ParamRef {
		self.app_state.get_parameter_ref(parameter_id)
	}
}

impl<'b> SignalContext for ViewContext<'b> {
    fn set_signal_value<T: Any>(&mut self, signal: &Signal<T>, value: T) {
        self.app_state.set_signal_value(signal, value)
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


