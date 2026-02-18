use std::{
    cell::Cell,
    collections::VecDeque,
    ops::{Deref, DerefMut},
};

use slotmap::{Key, SecondaryMap, SlotMap};

use crate::{
    core::{FxIndexSet, Point},
    platform,
    ui::{
        OverlayOptions, Widget, WidgetData, WidgetFlags, WidgetId, WidgetRef, WindowId,
        overlay::OverlayContainer,
        render::{GpuScene, WGPUSurface},
    },
};

type ChildCache = SecondaryMap<WidgetId, Vec<WidgetId>>;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum WidgetPos {
    Before(WidgetId),
    After(WidgetId),
    FirstChild(WidgetId),
    LastChild(WidgetId),
    Overlay(WidgetId, OverlayOptions),
}

pub(super) struct WindowState {
    pub(super) handle: platform::Handle,
    pub(super) wgpu_surface: WGPUSurface,
    pub(super) gpu_scene: GpuScene,
    pub(super) root_widget: WidgetId,
    pub(super) focus_widget: Option<WidgetId>,
    pub(super) overlays: OverlayContainer,
}

#[derive(Default)]
pub struct Widgets {
    /// Data (e.g. parent, layout, render scene etc.) associated with each widget
    pub(super) data: SlotMap<WidgetId, WidgetData>,
    /// Widget implementation. Should exist for each widget data.
    pub(super) widgets: SecondaryMap<WidgetId, Box<dyn Widget>>,
    pub(super) windows: SlotMap<WindowId, WindowState>,
    /// (Lazy) cache of child ids. Taffy requires random access during layout.
    child_id_cache: ChildCache,
    /// Ids of all widgets that have had their child list changed. Cleared during call to rebuild_children.
    child_layout_changed: FxIndexSet<WidgetId>,
    /// Ids of all widgets that have requested animation. Cleared during each call to [drive_animations]
    pending_animations: FxIndexSet<WidgetId>,
    /// Ids of all widgets that have requested render.
    pub(super) needing_render: FxIndexSet<WidgetId>,
    /// Temporary cache used to avoid allocations while performing traversals
    id_buffer: Cell<VecDeque<WidgetId>>,
    /// The widget that currently has mouse capture
    pub(super) mouse_capture_widget: Option<WidgetId>,
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

    pub(super) fn allocate_window(
        &mut self,
        handle: platform::Handle,
        wgpu_surface: WGPUSurface,
    ) -> WindowId {
        let window_id = self.windows.insert_with_key(|window_id| {
            let root_widget = self
                .data
                .insert_with_key(|id| WidgetData::new(window_id, id));
            self.child_id_cache.insert(root_widget, Vec::new());
            WindowState {
                handle,
                wgpu_surface,
                root_widget,
                focus_widget: None,
                gpu_scene: GpuScene::new(),
                overlays: OverlayContainer::default(),
            }
        });

        window_id
    }

    /// Allocate a new widget which does not have an implementation.
    /// Make sure to call [set_widget_impl] afterwards (or there will be panics later)
    pub(super) fn allocate_widget(&mut self, position: WidgetPos) -> WidgetId {
        let parent_id = match position {
            WidgetPos::Before(widget_id) => self.data[widget_id].parent_id,
            WidgetPos::After(widget_id) => self.data[widget_id].parent_id,
            WidgetPos::FirstChild(widget_id) => widget_id,
            WidgetPos::LastChild(widget_id) => widget_id,
            WidgetPos::Overlay(widget_id, _) => widget_id,
        };
        let window_id = self.data[parent_id].window_id;

        let id = self
            .data
            .insert_with_key(|id| WidgetData::new(window_id, id).with_parent(parent_id));
        self.child_id_cache.insert(id, Vec::new());
        self.move_widget(id, position);
        id
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
        let current = self
            .data
            .get_mut(widget_id)
            .expect("Widget should exist when being moved");
        let old_parent_id = current.parent_id;
        assert!(!old_parent_id.is_null(), "Cannot move a root widget");

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
            WidgetPos::Overlay(parent_id, overlay_options) => {
                let parent = &mut self.data[parent_id];
                let first_overlay_id = parent.first_overlay_id;
                self.windows[parent.window_id]
                    .overlays
                    .insert_or_update(widget_id, overlay_options);
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

    pub(super) fn window(&self, id: WindowId) -> &WindowState {
        self.windows.get(id).expect("Window handle not found")
    }

    pub(super) fn window_for_widget(&self, id: WidgetId) -> &WindowState {
        self.windows
            .get(self.data[id].window_id)
            .expect("Window handle not found")
    }

    pub(super) fn window_mut(&mut self, id: WindowId) -> &mut WindowState {
        self.windows.get_mut(id).expect("Window handle not found")
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
        let data = &self.data[widget_id];
        let is_overlay = data.is_overlay();
        let window_id = data.window_id;
        let parent_id = data.parent_id;
        if !parent_id.is_null() {
            let parent = &mut self.data[parent_id];
            if parent.first_overlay_id == widget_id {}
        }
        self.overlays[window_id].remove(widget_id);
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
