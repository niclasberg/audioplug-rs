use crate::{
    app::{
        EventContext, EventStatus, MouseEventContext, OverlayAnchor, OverlayOptions, ReadContext,
        View, Widget, WidgetId, WidgetMut, WrappedWidget,
    },
    core::{Align, Key},
    KeyEvent, MouseEvent,
};

pub struct Dropdown<VTrigger, FMenu> {
    trigger_view: VTrigger,
    menu_fn: FMenu,
}

impl<VTrigger: View, VMenu: View, FMenu: Fn() -> VMenu + 'static> Dropdown<VTrigger, FMenu> {
    pub fn new(trigger_view: VTrigger, menu_fn: FMenu) -> Self {
        Self {
            trigger_view,
            menu_fn,
        }
    }
}

impl<VTrigger: View, VMenu: View, FMenu: Fn() -> VMenu + 'static> View
    for Dropdown<VTrigger, FMenu>
{
    type Element = DropdownWidget<VTrigger::Element, FMenu>;

    fn build(self, cx: &mut crate::app::BuildContext<Self::Element>) -> Self::Element {
        let trigger_widget = cx.build_inner(self.trigger_view);
        cx.set_focusable(true);

        DropdownWidget {
            trigger_widget,
            menu_fn: self.menu_fn,
            overlay_id: None,
        }
    }
}

pub struct DropdownWidget<WTrigger, FMenu> {
    trigger_widget: WTrigger,
    menu_fn: FMenu,
    overlay_id: Option<WidgetId>,
}

impl<WTrigger: Widget, V: View, FMenu: Fn() -> V + 'static> DropdownWidget<WTrigger, FMenu> {
    fn is_dropdown_open(&self) -> bool {
        self.overlay_id.is_some()
    }

    fn close(mut widget: WidgetMut<Self>) {
        if let Some(overlay_id) = widget.overlay_id.take() {
            widget.remove_child_by_id(overlay_id);
        }
    }
}

impl<WTrigger: Widget, V: View, FMenu: Fn() -> V + 'static> WrappedWidget
    for DropdownWidget<WTrigger, FMenu>
{
    type Inner = WTrigger;

    fn inner(&self) -> &Self::Inner {
        &self.trigger_widget
    }

    fn inner_mut(&mut self) -> &mut Self::Inner {
        &mut self.trigger_widget
    }

    fn mouse_event(&mut self, event: MouseEvent, cx: &mut MouseEventContext) -> EventStatus {
        match event {
            MouseEvent::Down { .. } => {
                let view = (self.menu_fn)();
                cx.defer_update(self, move |mut widget| {
                    if !widget.is_dropdown_open() {
                        let id = widget.add_overlay(
                            view,
                            OverlayOptions {
                                anchor: OverlayAnchor::OutsideParent,
                                align: Align::Bottom,
                                z_index: 50000,
                                ..Default::default()
                            },
                        );
                        widget.overlay_id = Some(id);
                    }
                });
                EventStatus::Handled
            }
            _ => EventStatus::Ignored,
        }
    }

    fn key_event(&mut self, event: KeyEvent, cx: &mut EventContext) -> EventStatus {
        match event {
            KeyEvent::KeyDown { key, .. } if key == Key::Escape => {
                if self.is_dropdown_open() {
                    cx.defer_update(self, Self::close);
                    EventStatus::Handled
                } else {
                    EventStatus::Ignored
                }
            }
            _ => EventStatus::Ignored,
        }
    }
}
