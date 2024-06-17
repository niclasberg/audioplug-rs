use crate::{view::IdPath, MouseEvent};

use super::{EventContext, Widget};

pub struct ViewMessage {
    pub destination: IdPath,
    pub body: ViewMessageBody
}

impl ViewMessage {
    pub(crate) fn handle(&mut self, widget: &mut dyn Widget, ctx: &mut EventContext) {
        if let Some(child_id) = self.destination.pop_root() {
            if child_id.0 < widget.child_count() {
                self.handle(&mut widget.get_child_mut(child_id.0).widget, ctx)
            }
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

