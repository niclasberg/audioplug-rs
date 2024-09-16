use std::marker::PhantomData;

use crate::view::View;

use super::{widget_node::{WidgetFlags, WidgetId, WidgetMut}, Accessor, AppState, ParamContext, SignalGet, Widget};

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

    pub(crate) fn build<V: View>(&mut self, view: V) -> V::Element {
        let mut ctx = BuildContext {
            id: self.id,
            app_state: self.app_state,
            _phantom: PhantomData
        };
        view.build(&mut ctx)
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

    fn get_parameter_ref<'a>(&'a self, id: crate::param::ParameterId) -> Option<crate::param::ParamRef<'a>> {
        self.app_state.get_parameter_ref(id)
    }

    fn get_parameter_as<'a, P: crate::param::AnyParameter>(&'a self, param: &super::ParamEditor<P>) -> &'a P {
        self.app_state.get_parameter_as(param)
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


