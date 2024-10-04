use std::{any::Any, marker::PhantomData};

use crate::{param::{ParamRef, ParameterId}, view::View};

use super::{widget_node::{WidgetFlags, WidgetId, WidgetMut}, Accessor, AppState, NodeId, ParamContext, Runtime, Signal, SignalContext, SignalGet, SignalGetContext, Widget};

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
        let value = accessor.get_untracked(self.app_state);
        self.app_state.create_binding(accessor, self.id, f);
        value
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

    pub fn add_child<V: View>(&mut self, view: V) {
        self.app_state.add_widget(self.id, move |ctx| -> V::Element {
            view.build(ctx)
        });
    }

	pub fn add_child_with<V: View>(&mut self, view_factory: impl FnOnce(&mut AppContext) -> V) {
		self.app_state.add_widget(self.id, move |ctx| -> V::Element {
            ctx.build_with(view_factory)
        });
	}

    pub(crate) fn build<V: View>(&mut self, view: V) -> V::Element {
        let mut ctx = BuildContext {
            id: self.id,
            app_state: self.app_state,
            _phantom: PhantomData
        };
        view.build(&mut ctx)
    }

	pub(crate) fn build_with<V: View>(&mut self, view_factory: impl FnOnce(&mut AppContext) -> V) -> V::Element {
		let mut app_context = AppContext {
			app_state: &mut self.app_state,
		};
		let view = view_factory(&mut app_context);
		self.build(view)
	}

	pub fn set_style(&mut self, style: taffy::Style) {
		self.app_state.widget_data_mut(self.id).style = style;
	}

	pub fn update_style(&mut self, f: impl FnOnce(&mut taffy::Style)) {
		f(&mut self.app_state.widget_data_mut(self.id).style);
	}
}

impl<'s, W: Widget> ParamContext for BuildContext<'s, W> {
    fn host_handle(&self) -> &dyn super::HostHandle {
        self.app_state.host_handle()
    }
}

impl<'b, W: Widget> SignalGetContext for BuildContext<'b, W> {
    fn get_signal_value_ref_untracked<'a>(&'a self, signal_id: NodeId) -> &'a dyn Any {
        self.app_state.get_signal_value_ref_untracked(signal_id)
    }

    fn get_signal_value_ref<'a>(&'a mut self, signal_id: NodeId) -> &'a dyn Any {
        self.app_state.get_signal_value_ref(signal_id)
    }
	
	fn get_parameter_ref_untracked(&self, parameter_id: ParameterId) -> ParamRef {
		self.app_state.get_parameter_ref_untracked(parameter_id)
	}
	
	fn get_parameter_ref(&mut self, parameter_id: ParameterId) -> ParamRef {
		self.app_state.get_parameter_ref(parameter_id)
	}
}

pub struct AppContext<'a> {
    app_state: &'a mut AppState,
}

impl<'a> AppContext<'a> {
    pub fn create_signal<T: Any>(&mut self, value: T) -> Signal<T> {
        self.app_state.create_signal(value)
    }

    pub fn create_effect(&mut self, f: impl Fn(&mut Runtime) + 'static) {
        self.app_state.create_effect(f)
    }

	pub fn app_state(&self) -> &AppState {
		&self.app_state
	}
}

impl<'b> SignalGetContext for AppContext<'b> {
    fn get_signal_value_ref_untracked<'a>(&'a self, signal_id: NodeId) -> &'a dyn Any {
        self.app_state.get_signal_value_ref_untracked(signal_id)
    }

    fn get_signal_value_ref<'a>(&'a mut self, signal_id: NodeId) -> &'a dyn Any {
        self.app_state.get_signal_value_ref(signal_id)
    }
	
	fn get_parameter_ref_untracked(&self, parameter_id: ParameterId) -> ParamRef {
		self.app_state.get_parameter_ref_untracked(parameter_id)
	}
	
	fn get_parameter_ref(&mut self, parameter_id: ParameterId) -> ParamRef {
		self.app_state.get_parameter_ref(parameter_id)
	}
}

impl<'b> SignalContext for AppContext<'b> {
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


