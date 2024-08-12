use crate::view::View;

use super::{widget_node::{WidgetFlags, WidgetId, WidgetMut}, Accessor, AppState, SignalGet, Widget};

pub struct BuildContext<'a> {
    pub(crate) id: WidgetId,
    pub(crate) app_state: &'a mut AppState
}

impl<'a> BuildContext<'a> {
    pub fn get_and_track<W: Widget, T: Clone + 'static>(&mut self, accessor: Accessor<T>, f: impl Fn(&T, WidgetMut<'_, W>) + 'static) -> T {
        let value = accessor.get_untracked(self.app_state);
        let binding = self.app_state.create_binding(accessor, self.id, move |value, widget_node| {
            /*let mut ctx = WidgetContext {
                widget_data: &mut widget_node.data,
            };
            // SAFETY: The window class guarantees that the message is dispatched to *this* widget
            let widget = unsafe { &mut *(widget_node.widget.as_mut() as *mut dyn Widget as *mut W) };
            f(value, &mut ctx, widget)*/
            todo!()
        });
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

    pub fn add_child(&mut self, view: impl View) {
        self.app_state.add_widget(self.id, move |ctx| {
            view.build(ctx)
        });
    }

    /*pub fn build_child<'s, V: View>(&'s mut self, id: Id, view: V) -> WidgetNode {
		let mut id_path = self.id_path.clone();
        id_path.push_child(id);
        let mut data = WidgetData::new(id_path.clone());
        let widget = Box::new(view.build(&mut BuildContext {
            id_path,
            widget_data: &mut data,
            app_state: &mut self.app_state
        }));
        data.style = widget.style();
        WidgetNode {
            widget,
            data
        }
    }*/
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


