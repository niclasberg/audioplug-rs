use std::cell::Cell;

use slotmap::{Key, SecondaryMap, SlotMap};

use crate::{
    core::{FxIndexSet, Point},
    ui::{Widget, WidgetData, WidgetFlags, WidgetId, WidgetRef},
};


#[derive(Default)]
pub struct Widgets {
    /// Data (e.g. parent, layout, render scene etc.) associated with each widget
    pub(super) data: SlotMap<WidgetId, WidgetData>,
    /// Widget implementation. Should exist for each widget data.
    pub(super) widgets: SecondaryMap<WidgetId, Box<dyn Widget>>,
    /// (Lazy) cache of child ids. Taffy requires random access during layout.
    children: SecondaryMap<WidgetId, Vec<WidgetId>>,
    /// Ids of all widgets that have requested animation. Cleared during each call to [drive_animations]
    pub(super) pending_animations: FxIndexSet<WidgetId>,
    /// Ids of all widgets that have requested render.
    pub(super) needing_render: FxIndexSet<WidgetId>,
    /// Temporary cache used to avoid allocations while performing traversals
    id_buffer: Cell<Vec<WidgetId>>,
}

impl Widgets {
    pub fn get(&self, widget_id: WidgetId) -> WidgetRef<'_, dyn Widget> {
        WidgetRef::new(self, widget_id)
    }

    pub fn contains(&self, widget_id: WidgetId) -> bool {
        self.data.contains_key(widget_id)
    }

    pub(crate) fn children_as_vec(&self, widget_id: WidgetId) -> &Vec<WidgetId> {
        let widget_data = &self.data[widget_id];
        if widget_data.flag_is_set(WidgetFlags::CHILDREN_CHANGED) {

            widget_data.clear_flag(WidgetFlags::CHILDREN_CHANGED);
        }
        &self.children[widget_id]
    }

    pub fn sibling_id_iter(&self, widget_id: WidgetId) -> ChildIdIter<'_> {
        ChildIdIter { inner: WidgetIdIter::all_siblings(&self, widget_id), widgets: &self }
    }

    pub fn child_id_iter(&self, widget_id: WidgetId) -> ChildIdIter<'_> {
        ChildIdIter { inner: WidgetIdIter::all_children(&self, widget_id), widgets: &self }
    }

    pub fn overlay_id_iter(&self, widget_id: WidgetId) -> ChildIdIter<'_> {
        ChildIdIter { inner: WidgetIdIter::all_children(&self, widget_id), widgets: &self }
    }

    pub(super) fn remove(&mut self, widget_id: WidgetId) {
        
    }

    #[inline(always)]
    pub fn get_parent(&self, widget_id: WidgetId) -> WidgetId {
        self.data[widget_id].parent_id
    }

    pub fn has_parent(&self, child_id: WidgetId, parent_id: WidgetId) -> bool {
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

    pub(crate) fn request_layout(&mut self, widget_id: WidgetId) {
        let mut current = widget_id;
        // Merge until we hit the root, an overlay or another node that
        // already have marked as needing layout
        loop {
            if current.is_null() {
                break;
            }
            let data = &mut self.data[current];
            if data.is_overlay() || data.flag_is_set(WidgetFlags::NEEDS_LAYOUT) {
                break;
            }

            data.set_flag(WidgetFlags::NEEDS_LAYOUT);
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

pub struct ChildIdIter<'a> {
    inner: WidgetIdIter,
    widgets: &'a Widgets
}

impl<'a> Iterator for ChildIdIter<'a> {
    type Item = WidgetId;

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next_id(self.widgets)
    }
}

impl<'a> DoubleEndedIterator for ChildIdIter<'a> {
    fn next_back(&mut self) -> Option<Self::Item> {
        self.inner.next_back_id(self.widgets)
    }
}

// Keeps track of the current iterable range. The Widgets have to be passed on each
// call to next_id in order to avoid borrowing problems.
pub(super) struct WidgetIdIter {
    first: WidgetId,
    last: WidgetId,
    done: bool,
}

impl WidgetIdIter {
    pub(super) fn all_siblings(widgets: &Widgets, first: WidgetId) -> Self {
        if first.is_null() {
            Self {
                first: WidgetId::null(),
                last: WidgetId::null(),
                done: true,
            }
        } else {
            let last = widgets.data[first].prev_sibling_id;
            Self {
                first,
                last,
                done: false,
            }
        }
    }

    pub(super) fn all_children(widgets: &Widgets, parent_id: WidgetId) -> Self {
        let first = widgets.data[parent_id].first_child_id;
        Self::all_siblings(widgets, first)
    }

    pub(super) fn all_overlays(widgets: &Widgets, parent_id: WidgetId) -> Self {
        let first = widgets.data[parent_id].first_overlay_id;
        Self::all_siblings(widgets, first)
    }

    pub(super) fn next_id(&mut self, widgets: &Widgets) -> Option<WidgetId> {
        if self.done {
            None
        } else {
            let first = self.first;
            self.first = widgets.data[first].next_sibling_id;
            if self.first == self.last {
                self.done = true;
            }
            Some(first)
        }
    }

    pub(super) fn next_back_id(&mut self, widgets: &Widgets) -> Option<WidgetId> {
        if self.done {
            None
        } else {
            let last = self.last;
            self.last = widgets.data[last].prev_sibling_id;
            if self.first == self.last {
                self.done = true;
            }
            Some(last)
        }
    }
}
