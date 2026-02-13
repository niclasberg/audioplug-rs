use std::{cell::Cell, collections::VecDeque, ops::{Deref, DerefMut}};

use slotmap::{Key, SecondaryMap, SlotMap};

use crate::{
    core::{FxIndexSet, Point},
    ui::{Widget, WidgetData, WidgetFlags, WidgetId, WidgetRef, WindowId},
};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum WidgetPos {
    Before(WidgetId),
    After(WidgetId),
    FirstChild(WidgetId),
    LastChild(WidgetId),
    FirstOverlay(WidgetId),
    LastOverlay(WidgetId),
}

#[derive(Default)]
pub struct Widgets {
    /// Data (e.g. parent, layout, render scene etc.) associated with each widget
    pub(super) data: SlotMap<WidgetId, WidgetData>,
    /// Widget implementation. Should exist for each widget data.
    pub(super) widgets: SecondaryMap<WidgetId, Box<dyn Widget>>,
    /// (Lazy) cache of child ids. Taffy requires random access during layout.
    child_id_cache: SecondaryMap<WidgetId, Vec<WidgetId>>,
    /// Ids of all widgets that have had their child list changed. Cleared during call to rebuild_children.
    child_layout_changed: FxIndexSet<WidgetId>,
    /// Ids of all widgets that have requested animation. Cleared during each call to [drive_animations]
    pending_animations: FxIndexSet<WidgetId>,
    /// Ids of all widgets that have requested render.
    pub(super) needing_render: FxIndexSet<WidgetId>,
    /// Temporary cache used to avoid allocations while performing traversals
    id_buffer: Cell<VecDeque<WidgetId>>,
}

impl Widgets {
    pub fn get(&self, widget_id: WidgetId) -> WidgetRef<'_, dyn Widget> {
        WidgetRef::new(self, widget_id)
    }

    pub fn contains(&self, widget_id: WidgetId) -> bool {
        self.data.contains_key(widget_id)
    }

    pub(super) fn rebuild_children(&mut self) {
        for widget_id in std::mem::take(&mut self.child_layout_changed) {
            let mut children = std::mem::take(&mut self.child_id_cache[widget_id]);
            children.clear();
            children.extend(self.child_id_iter(widget_id));
            self.child_id_cache[widget_id] = children;
        }
    }

    /// Get the (cached) child ids for a widget. You must call rebuild_children before using this method.
    pub(super) fn cached_child_ids(&self, widget_id: WidgetId) -> &Vec<WidgetId> {
        debug_assert!(self.child_layout_changed.is_empty());
        &self.child_id_cache[widget_id]
    }

    /// Iterator over the ids of all siblings of a node
    pub fn sibling_id_iter(&self, widget_id: WidgetId) -> ChildIdIter<'_> {
        ChildIdIter {
            inner: WidgetIdIter::all_siblings(&self, widget_id),
            widgets: &self,
        }
    }

    /// Iterator over the ids of all children of a node
    pub fn child_id_iter(&self, widget_id: WidgetId) -> ChildIdIter<'_> {
        ChildIdIter {
            inner: WidgetIdIter::all_children(&self, widget_id),
            widgets: &self,
        }
    }

    /// Iterator over the ids of all overlays of a node
    pub fn overlay_id_iter(&self, widget_id: WidgetId) -> ChildIdIter<'_> {
        ChildIdIter {
            inner: WidgetIdIter::all_children(&self, widget_id),
            widgets: &self,
        }
    }

    /// Allocate a new widget which does not have an implementation or a parent attached.
    /// Make sure to call [set_widget_impl] afterwards (or there will be panics later)
    /// For non-root widgets, you also need to call [move_widget] to assign siblings, window and parent
    pub(super) fn allocate_widget(&mut self, window_id: WindowId) -> WidgetId {
        let id = self
            .data
            .insert_with_key(|id| WidgetData::new(window_id, id));
        self.child_id_cache.insert(id, Vec::new());
        id
    }

    fn internal_remove(&mut self, widget_id: WidgetId) -> WidgetData {
        self.child_id_cache.remove(widget_id);
        self.widgets.remove(widget_id);
        // Maybe skip clearing these and update dependent code to not require the widgets to exist
        self.pending_animations.shift_remove(&widget_id);
        self.needing_render.shift_remove(&widget_id);

        self.data
            .remove(widget_id)
            .expect("Widget being removed should exist")
    }

    // Remove a widget and all its children. 
    pub(super) fn remove(&mut self, widget_id: WidgetId, f: &mut impl FnMut(WidgetData)) {
        let parent_id = self.data[widget_id].parent_id;
        if !parent_id.is_null() {
            let parent = &mut self.data[widget_id];
            if parent.first_overlay_id == widget_id {}
        }
        self.remove_children(widget_id, f);
        f(self.internal_remove(widget_id));
    }

    pub(super) fn remove_children(&mut self, widget_id: WidgetId, f: &mut impl FnMut(WidgetData)) {
        let mut children_to_remove = self.id_buffer.take();
        children_to_remove.clear();
        children_to_remove.extend(self.child_id_iter(widget_id));
        children_to_remove.extend(self.overlay_id_iter(widget_id));

        while let Some(child_id) = children_to_remove.back().copied() {
            children_to_remove.extend(self.child_id_iter(child_id));
            children_to_remove.extend(self.overlay_id_iter(child_id));
            f(self.internal_remove(child_id));
        }

        self.id_buffer.set(children_to_remove);
    }

    fn move_widget_before(&mut self, widget_id: WidgetId, next_id: WidgetId) -> WidgetId {
        let next = &mut self.data[next_id];
        let prev_id = std::mem::replace(&mut next.prev_sibling_id, widget_id);
        let new_parent_id = next.parent_id;
        assert!(!new_parent_id.is_null());
        self.data[prev_id].next_sibling_id = widget_id;

        let current = &mut self.data[widget_id];
        current.next_sibling_id = next_id;
        current.prev_sibling_id = prev_id;
        new_parent_id
    }

    /// Moves a widget to a new position in the tree by.
    pub fn move_widget(&mut self, widget_id: WidgetId, destination: WidgetPos) {
        let current = self.data.get_mut(widget_id).expect("Widget should exist when being moved");
        let old_parent_id = current.parent_id;
        if !old_parent_id.is_null() {
            self.child_layout_changed.insert(old_parent_id);
        }

        let new_parent_id = match destination {
            WidgetPos::Before(next_id) => self.move_widget_before(widget_id, next_id),
            WidgetPos::After(prev_id) => {
                self.move_widget_before(widget_id, self.data[prev_id].next_sibling_id)
            }
            WidgetPos::FirstChild(parent_id) => {
                let parent = &mut self.data[parent_id];
                let first_child_id = std::mem::replace(&mut parent.first_child_id, widget_id);
                if !first_child_id.is_null() {
                    self.move_widget_before(widget_id, first_child_id);
                }
                parent_id
            }
            WidgetPos::LastChild(parent_id) => {
                let parent = &mut self.data[parent_id];
                let first_child_id = parent.first_child_id;
                if first_child_id.is_null() {
                    parent.first_child_id = widget_id;
                } else {
                    self.move_widget_before(widget_id, first_child_id);
                }
                parent_id
            }
            WidgetPos::FirstOverlay(parent_id) => {
                let parent = &mut self.data[parent_id];
                let first_overlay_id = std::mem::replace(&mut parent.first_overlay_id, widget_id);
                if !first_overlay_id.is_null() {
                    self.move_widget_before(widget_id, first_overlay_id);
                }
                parent_id
            }
            WidgetPos::LastOverlay(parent_id) => {
                let parent = &mut self.data[parent_id];
                let first_overlay_id = parent.first_overlay_id;
                if first_overlay_id.is_null() {
                    parent.first_overlay_id = widget_id;
                } else {
                    self.move_widget_before(widget_id, first_overlay_id);
                }
                parent_id
            }
        };

        let new_parent = &mut self.data[new_parent_id];
        self.child_layout_changed.insert(new_parent_id);
        let window_id = new_parent.window_id;

        let current = &mut self.data[widget_id];
        current.parent_id = new_parent_id;
        current.window_id = window_id;
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

    pub(super) fn take_requested_animations(&mut self) -> FxIndexSet<WidgetId> {
        std::mem::take(&mut self.pending_animations)
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

    pub(super) fn lease_widget(&mut self, id: WidgetId) -> LeasedWidget {
        LeasedWidget(id, self.widgets.remove(id).unwrap())
    }

    pub(super) fn unlease_widget(&mut self, widget: LeasedWidget) {
        self.widgets.insert(widget.0, widget.1);
    }

    /// Gets children of a widget. The order is from the root and down the tree (draw order).
    pub(super) fn get_widgets_at(&self, root_id: WidgetId, pos: Point) -> Vec<WidgetId> {
        let mut widgets = Vec::new();
        let mut stack = self.id_buffer.take();
        stack.clear();

        if self.data[root_id].global_bounds().contains(pos) {
            stack.push_back(root_id);
            while let Some(current) = stack.pop_front() {
                widgets.push(current);

                let mut widget_id_iter = WidgetIdIter::all_children(&self, current);
                while let Some(child_id) = widget_id_iter.next_id(&self) {
                    let data = &self.data[child_id];
                    if data.global_bounds().contains(pos) && !data.is_overlay() {
                        stack.push_back(child_id)
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
    widgets: &'a Widgets,
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

pub struct LeasedWidget(WidgetId, Box<dyn Widget>);

impl Deref for LeasedWidget {
    type Target = dyn Widget;

    fn deref(&self) -> &Self::Target {
        self.1.deref()
    }
}

impl DerefMut for LeasedWidget {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.1.deref_mut()
    }
}