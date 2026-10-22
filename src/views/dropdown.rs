use crate::{
    KeyEvent,
    core::{Align, Key},
    ui::{
        BuildContext, Sense, Event, EventHandlerExt, EventHandlers, EventResult,
        OverlayAnchor, OverlayOptions, View, reactive::Trigger,
    },
};

pub struct Dropdown<VTrigger, FMenu> {
    trigger_view: VTrigger,
    menu_fn: FMenu,
    event_handlers: EventHandlers,
}

impl<VTrigger: View, VMenu: View, FMenu: Fn() -> VMenu + 'static> Dropdown<VTrigger, FMenu> {
    pub fn new(trigger_view: VTrigger, menu_fn: FMenu) -> Self {
        Self {
            trigger_view,
            menu_fn,
            event_handlers: Default::default(),
        }
    }
}

impl<VTrigger: View, VMenu: View, FMenu: Fn() -> VMenu + 'static> View
    for Dropdown<VTrigger, FMenu>
{
    fn build(self, cx: &mut BuildContext) {
        let is_open = Trigger::new(cx);
        let trigger_id = cx.id();
        let mut overlay_id = None;
        is_open.watch(cx, move |cx| {
            if let Some(overlay_id) = overlay_id.take() {
                cx.widget_mut(overlay_id).remove();
            } else {
                let id = cx.widget_mut(trigger_id).add_overlay(
                    (self.menu_fn)(),
                    OverlayOptions {
                        anchor: OverlayAnchor::OutsideParent,
                        align: Align::Bottom,
                        z_index: 50000,
                        ..Default::default()
                    },
                );
                overlay_id = Some(id);
            }
        });

        self.trigger_view.build(cx);
        let event_handlers = self.event_handlers.on_event(
            Sense::FOCUSABLE | Sense::CLICKABLE,
            move |cx, event| match event {
                Event::Click(mouse_down) => {
                    //if mouse_down.button == MouseButton::LEFT {
                    is_open.notify(cx);
                    //}
                    EventResult::Stop
                }
                Event::Key(KeyEvent::KeyDown {
                    key: Key::Escape, ..
                }) => {
                    is_open.notify(cx);
                    EventResult::Stop
                }
                _ => EventResult::Continue,
            },
        );
        cx.event_handlers(event_handlers);
    }
}
