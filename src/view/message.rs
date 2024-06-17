use crate::{view::IdPath, MouseEvent};

use super::{EventContext, Widget};

pub struct ViewMessage {
    pub destination: IdPath,
    pub body: ViewMessageBody
}

impl ViewMessage {
	pub(crate) fn handle(&mut self, widget: &mut dyn Widget, ctx: &mut EventContext) {
		self.destination.pop_root();
		self.handle_impl(widget, ctx)
	}

    fn handle_impl(&mut self, widget: &mut dyn Widget, ctx: &mut EventContext) {
        if let Some(child_id) = self.destination.pop_root() {
			let child = widget.get_child_mut(child_id.0);
			ctx.with_child(&mut child.data, |ctx| {
				self.handle_impl(&mut child.widget, ctx);
				ctx.view_flags()
			});
        } else {
            match self.body {
                ViewMessageBody::Mouse(mouse_event) => {
                    widget.mouse_event(mouse_event, ctx);
                },
                ViewMessageBody::FocusChanged(has_focus) => 
                    widget.focus_changed(has_focus, ctx)
            };
        }
    }
}

pub enum ViewMessageBody {
    Mouse(MouseEvent),
    FocusChanged(bool)
}

