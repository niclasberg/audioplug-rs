use std::cell::Cell;

use slotmap::{Key, SecondaryMap, SlotMap};

use crate::{
    core::{FxIndexSet, Point},
    ui::{Widget, WidgetData, WidgetFlags, WidgetId},
};

#[derive(Default)]
pub struct Widgets {
    /// Data (e.g. parent, layout, render scene etc.) associated with each widget
    pub(super) data: SlotMap<WidgetId, WidgetData>,
    /// Widget implementation. Should exist for each widget data.
    pub(super) widgets: SecondaryMap<WidgetId, Box<dyn Widget>>,
    /// (Lazy) cache of child ids. Taffy requires random access during layout.
    pub(super) children: SlotMap<WidgetId, Vec<WidgetId>>,
    pub(super) overlays: SlotMap<WidgetId, Vec<WidgetId>>,
    /// Ids of all widgets that have requested animation. Cleared during each call to [drive_animations]
    pub(super) pending_animations: FxIndexSet<WidgetId>,
    /// Ids of all widgets that have requested render.
    pub(super) needing_render: FxIndexSet<WidgetId>,
    /// Temporary cache used to avoid allocations while performing traversals
    id_buffer: Cell<Vec<WidgetId>>,
}

impl Widgets {
    pub fn child_count(&self, widget_id: WidgetId) -> usize {
        self.children.get(widget_id).map(|c| c.len()).unwrap_or(0)
    }

    pub fn contains(&self, widget_id: WidgetId) -> bool {
        self.data.contains_key(widget_id)
    }

    pub fn children(&self, widget_id: WidgetId) -> &Vec<WidgetId> {
        todo!()
    }

    pub fn overlay_count(&self, widget_id: WidgetId) -> usize {
        self.overlays.get(widget_id).map(|c| c.len()).unwrap_or(0)
    }

    #[inline(always)]
    pub fn get_parent(&self, widget_id: WidgetId) -> WidgetId {
        self.data[widget_id].parent_id
    }

    pub fn widget_has_parent(&self, child_id: WidgetId, parent_id: WidgetId) -> bool {
        let mut id = child_id;
        while !id.is_null() {
            id = self.data[id].parent_id;
            if id == parent_id {
                return true;
            }
        }
        false
    }

    pub(crate) fn request_animation(&mut self, widget_id: WidgetId) {
        self.pending_animations.insert(widget_id);
    }

    pub(super) fn merge_flags(&mut self, source: WidgetId) {
        let mut current = source;
        let mut flags_to_apply = WidgetFlags::empty();
        // Merge until we hit the root, or an overlay
        while !current.is_null() && !self.data[current].is_overlay() {
            let data = &mut self.data[current];
            data.flags |= flags_to_apply;
            flags_to_apply = data.flags & (WidgetFlags::NEEDS_LAYOUT);
            current = data.parent_id;
        }
    }

    pub(super) fn lease_widget(&mut self, id: WidgetId) -> Box<dyn Widget> {
        self.widgets.remove(id).unwrap()
    }

    pub(super) fn unlease_widget(&mut self, id: WidgetId, widget: Box<dyn Widget>) {
        self.widgets.insert(id, widget);
    }

    /// Gets children of a widget. The order is from the root and down the tree (draw order).
    pub(super) fn get_widgets_at(&self, root_id: WidgetId, pos: Point) -> Vec<WidgetId> {
        let mut widgets = Vec::new();
        let mut stack = self.id_buffer.take();
        stack.clear();

        if self.data[root_id].global_bounds().contains(pos) {
            stack.push(root_id);
            while let Some(current) = stack.pop() {
                widgets.push(current);

                for &child in self.children[current].iter().rev() {
                    let data = &self.data[child];
                    if data.global_bounds().contains(pos) && !data.is_overlay() {
                        stack.push(child)
                    }
                }
            }
        }
        self.id_buffer.set(stack);
        widgets
    }
}
