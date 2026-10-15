use crate::{
    KeyEvent, MouseButton,
    core::{Align, Key},
    ui::{
        BuildContext, Capabilities, Event, EventResult, OverlayAnchor, OverlayOptions, View,
        ViewEventHandlers, WidgetHandler,
        reactive::{ReactiveValue, Var},
    },
};

pub struct Dropdown<VTrigger, FMenu> {
    trigger_view: VTrigger,
    menu_fn: FMenu,
    event_handlers: ViewEventHandlers,
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
        let is_open = Var::new(cx, false);
        let trigger_id = cx.id();
        let mut overlay_id = None;
        is_open.watch(cx, move |cx, is_open| {
            if *is_open {
                if overlay_id.is_none() {
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
            } else {
                if let Some(overlay_id) = overlay_id.take() {
                    cx.widget_mut(overlay_id).remove();
                }
            }
        });

        cx.handler(Capabilities::FOCUSABLE, DropdownHandler { is_open });

        self.trigger_view.build(cx);
    }
}

struct DropdownHandler {
    is_open: Var<bool>,
}

impl WidgetHandler for DropdownHandler {
    fn event(&mut self, cx: &mut crate::ui::EventContext, event: &Event) -> EventResult {
        match event {
            Event::MouseDown(mouse_down) => {
                if mouse_down.button == MouseButton::LEFT {
                    self.is_open.update(cx, |_, val| *val = !*val);
                }
                EventResult::Stop
            }
            Event::Key(KeyEvent::KeyDown {
                key: Key::Escape, ..
            }) => {
                self.is_open.update(cx, |_, val| *val = !*val);
                EventResult::Stop
            }
            _ => EventResult::Continue,
        }
    }

    fn render(&mut self, _cx: &mut crate::ui::RenderContext) {}
}
