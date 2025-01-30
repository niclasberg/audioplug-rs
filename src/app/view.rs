use std::marker::PhantomData;
use crate::style::Style;
use super::{AppState, CreateContext, Owner, ParamContext, ReactiveContext, ReadContext, Scope, Widget, WidgetFlags, WidgetId};

pub type AnyView = Box<dyn FnOnce(&mut BuildContext<Box<dyn Widget>>) -> Box<dyn Widget>>;

pub trait View: Sized {
    type Element: Widget + 'static;

    fn build(self, ctx: &mut BuildContext<Self::Element>) -> Self::Element;

    fn as_any_view(self) -> AnyView
    where 
        Self: 'static 
    {
        Box::new(move |ctx| Box::new(ctx.build(self)))
    }
}


impl View for AnyView {
	type Element = Box<dyn Widget>;

	fn build(self, ctx: &mut BuildContext<Self::Element>) -> Self::Element {
		self(ctx)
	}
}
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

    pub fn add_child(&mut self, view: impl View) -> WidgetId {
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

	pub fn set_style(&mut self, style: Style) {
		self.app_state.widget_data_mut(self.id).style = style;
	}

	pub fn update_style(&mut self, f: impl FnOnce(&mut Style)) {
		f(&mut self.app_state.widget_data_mut(self.id).style);
	}
}

impl<'s, W: Widget> ParamContext for BuildContext<'s, W> {
    fn host_handle(&self) -> &dyn super::HostHandle {
        self.app_state.host_handle()
    }
}

impl<'s, W: Widget> ReadContext for BuildContext<'s, W> {
    fn scope(&self) -> Scope {
        Scope::Root
    }
}

impl<'b, W: Widget> ReactiveContext for BuildContext<'b, W> {
    fn runtime(&self) -> &super::Runtime {
        self.app_state.runtime()
    }

    fn runtime_mut(&mut self) -> &mut super::Runtime {
        self.app_state.runtime_mut()
    }
}

impl<'s, W: Widget> CreateContext for BuildContext<'s, W> {
    fn owner(&self) -> Option<Owner> {
        Some(Owner::Widget(self.id))
    }
}