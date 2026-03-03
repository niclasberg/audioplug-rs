use std::{
    cell::Cell,
    ops::{Index, IndexMut},
};

use bitflags::bitflags;
use slotmap::{Key, KeyData, SlotMap, new_key_type};

use crate::{
    core::{Point, Rect, RoundedRect, Shape, Size, Zero},
    ui::reactive::NodeId,
};

use super::{WindowId, style::Style};

new_key_type! {
    pub struct WidgetId;
}

impl From<taffy::NodeId> for WidgetId {
    fn from(value: taffy::NodeId) -> Self {
        Self::from(KeyData::from_ffi(value.into()))
    }
}

impl From<WidgetId> for taffy::NodeId {
    fn from(val: WidgetId) -> Self {
        val.0.as_ffi().into()
    }
}

bitflags!(
    #[derive(Debug, Clone, Copy)]
    pub struct WidgetFlags : u32 {
        const EMPTY = 0;
        // Dirty flags
        const NEEDS_LAYOUT = 1 << 1;
        const NEEDS_RENDER = 1 << 2;

        // Capability flags
        const FOCUSABLE = 1 << 4;
        const OVERLAY = 1 << 5;

        // Status flags
        const UNDER_MOUSE_CURSOR = 1 << 8;
    }
);

/// Data associated with each widget. The nodes form an intrusive tree,
/// and are all stored in a WidgetDataMap which handles node creation,
/// swapping etc. in order to ensure that link invariants are always kept.
pub struct WidgetData {
    pub(super) id: WidgetId,
    pub(super) window_id: WindowId,
    pub(super) parent_id: WidgetId,
    /// Id of the first child of the node (or null if )
    pub(super) first_child_id: WidgetId,
    pub(super) next_sibling_id: WidgetId,
    pub(super) prev_sibling_id: WidgetId,
    pub(crate) first_owned_node_id: NodeId,
    pub(super) style: Style,
    pub(super) layout: taffy::Layout,
    flags: Cell<WidgetFlags>,
    pub(super) origin: Point,
}

impl WidgetData {
    pub fn new(window_id: WindowId, id: WidgetId) -> Self {
        Self {
            id,
            window_id,
            parent_id: WidgetId::null(),
            first_child_id: WidgetId::null(),
            next_sibling_id: id,
            prev_sibling_id: id,
            first_owned_node_id: NodeId::null(),
            style: Default::default(),
            layout: Default::default(),
            flags: Cell::new(WidgetFlags::EMPTY),
            origin: Point::ZERO,
        }
    }

    pub fn reset(&mut self) {
        self.flags.set(WidgetFlags::EMPTY);
    }

    pub fn with_parent(mut self, parent_id: WidgetId) -> Self {
        self.parent_id = parent_id;
        self
    }

    pub fn with_siblings(mut self, prev_id: WidgetId, next_id: WidgetId) -> Self {
        self.prev_sibling_id = prev_id;
        self.next_sibling_id = next_id;
        self
    }

    pub fn id(&self) -> WidgetId {
        self.id
    }

    pub fn has_siblings(&self) -> bool {
        self.next_sibling_id != self.prev_sibling_id
    }

    /// Local bounds of the widget, relative to its parent
    pub fn local_bounds(&self) -> Rect {
        Rect::from_origin(self.offset(), self.size())
    }

    /// Bounds of the widget, in global coords, including borders and padding
    pub fn global_bounds(&self) -> Rect {
        Rect::from_origin(self.origin(), self.size())
    }

    fn subtract_padding_and_border(&self, rect: Rect) -> Rect {
        Rect {
            left: rect.left + (self.layout.border.left + self.layout.padding.left) as f64,
            top: rect.top + (self.layout.border.top + self.layout.padding.top) as f64,
            right: rect.right - (self.layout.border.right + self.layout.padding.right) as f64,
            bottom: rect.bottom - (self.layout.border.bottom + self.layout.padding.bottom) as f64,
        }
    }

    /// Bounds of the widget, in global coords, excluding borders and padding
    pub fn content_bounds(&self) -> Rect {
        self.subtract_padding_and_border(self.global_bounds())
    }

    pub fn border(&self) -> Rect {
        Rect {
            left: self.layout.border.left as f64,
            top: self.layout.border.top as f64,
            right: self.layout.border.right as f64,
            bottom: self.layout.border.bottom as f64,
        }
    }

    pub fn padding(&self) -> Rect {
        Rect {
            left: self.layout.padding.left as f64,
            top: self.layout.padding.top as f64,
            right: self.layout.padding.right as f64,
            bottom: self.layout.padding.bottom as f64,
        }
    }

    pub fn get_and_clear_flag(&self, flag: WidgetFlags) -> bool {
        let flags = self.flags.get();
        self.flags.set(flags & !flag);
        flags.contains(flag)
    }

    pub fn set_or_clear_flag(&self, flag: WidgetFlags, set: bool) {
        let mut flags = self.flags.get();
        if set {
            flags |= flag;
        } else {
            flags &= !flag;
        }
        self.flags.set(flags);
    }

    pub fn set_flag(&self, flag: WidgetFlags) {
        let mut flags = self.flags.get();
        flags |= flag;
        self.flags.set(flags);
    }

    pub fn clear_flag(&self, flag: WidgetFlags) {
        let mut flags = self.flags.get();
        flags &= !flag;
        self.flags.set(flags);
    }

    #[inline(always)]
    pub fn flag_is_set(&self, flag: WidgetFlags) -> bool {
        self.flags.get().contains(flag)
    }

    pub fn size(&self) -> Size {
        self.layout.size.map(|x| x as f64).into()
    }

    pub fn offset(&self) -> Point {
        self.layout.location.map(|x| x as f64).into()
    }

    pub fn origin(&self) -> Point {
        self.origin
    }

    pub fn with_style(mut self, f: impl Fn(&mut Style)) -> Self {
        f(&mut self.style);
        self
    }

    #[inline(always)]
    pub fn needs_layout(&self) -> bool {
        self.flag_is_set(WidgetFlags::NEEDS_LAYOUT)
    }

    pub fn is_hidden(&self) -> bool {
        self.style.hidden
    }

    #[inline(always)]
    pub fn is_overlay(&self) -> bool {
        self.flag_is_set(WidgetFlags::OVERLAY)
    }

    pub fn shape(&self) -> Shape {
        if self.style.corner_radius == Size::ZERO {
            self.global_bounds().into()
        } else {
            RoundedRect::new(self.global_bounds(), self.style.corner_radius).into()
        }
    }
}

#[derive(Default)]
pub struct WidgetTree {
    data: SlotMap<WidgetId, WidgetData>,
}

impl WidgetTree {
    #[inline(always)]
    pub fn get(&self, id: WidgetId) -> Option<&WidgetData> {
        self.data.get(id)
    }

    #[inline(always)]
    pub fn get_mut(&mut self, id: WidgetId) -> Option<&mut WidgetData> {
        self.data.get_mut(id)
    }

    pub fn contains(&self, widget_id: WidgetId) -> bool {
        self.data.contains_key(widget_id)
    }

    pub fn iter(&self) -> impl Iterator<Item = &WidgetData> {
        self.data.values()
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut WidgetData> {
        self.data.values_mut()
    }

    /// Iterator over the ids of all siblings of a node
    pub fn sibling_id_iter(&self, widget_id: WidgetId) -> ChildIdIter<'_> {
        ChildIdIter {
            inner: SiblingWalker::all_siblings(self, widget_id),
            widgets: self,
        }
    }

    pub fn sibling_id_walker(&self, widget_id: WidgetId) -> SiblingWalker {
        SiblingWalker::all_siblings(self, widget_id)
    }

    /// Iterator over the ids of all children of a node
    pub fn child_id_iter(&self, widget_id: WidgetId) -> ChildIdIter<'_> {
        ChildIdIter {
            inner: SiblingWalker::all_children(self, widget_id),
            widgets: self,
        }
    }

    pub fn child_id_walker(&self, widget_id: WidgetId) -> SiblingWalker {
        SiblingWalker::all_children(self, widget_id)
    }

    pub fn dfs_walker(&self, root_id: WidgetId) -> DFSWalker {
        DFSWalker::new(root_id)
    }

    pub fn dfs_iter(&self, root_id: WidgetId) -> DFSIterator<'_> {
        DFSIterator {
            tree: &self,
            walker: self.dfs_walker(root_id),
        }
    }

    /// Reset a widget, removing all children and reseting its data to its default state
    pub fn reset(&mut self, id: WidgetId, mut on_removed: impl FnMut(WidgetData)) {
        let mut current = id;
        loop {
            if let Some(first_child_id) = self.pop_first_child(current) {
                current = first_child_id;
            } else if current == id {
                break;
            } else {
                let data = self.data.remove(current).unwrap();
                let parent_id = data.parent_id;
                on_removed(data);
                current = parent_id;
            }
        }
        self.data[id].reset();
    }

    fn pop_first_child(&mut self, id: WidgetId) -> Option<WidgetId> {
        let first_child_id = self.data[id].first_child_id;
        if first_child_id.is_null() {
            return None;
        }

        let WidgetData {
            next_sibling_id,
            prev_sibling_id,
            ..
        } = self.data[first_child_id];

        if next_sibling_id == prev_sibling_id {
            // Only child
            self.data[id].first_child_id = WidgetId::null();
        } else {
            self.data[next_sibling_id].prev_sibling_id = prev_sibling_id;
            self.data[prev_sibling_id].next_sibling_id = next_sibling_id;
            self.data[id].first_child_id = next_sibling_id;
        }

        Some(first_child_id)
    }

    /// Removes a widget and all its children. The [on_removed] function is invoked for each widget
    /// that is removed
    pub fn remove(&mut self, id: WidgetId, mut on_removed: impl FnMut(WidgetData)) {
        self.detach(id);

        let mut current = id;
        loop {
            if let Some(first_child_id) = self.pop_first_child(current) {
                current = first_child_id;
            } else {
                let data = self.data.remove(current).unwrap();
                let parent_id = data.parent_id;
                on_removed(data);
                if parent_id.is_null() {
                    return;
                } else {
                    current = parent_id;
                }
            }
        }
    }

    /// Detaches a widget subtree from its parent and siblings, effectively making the widget into a new root
    pub fn detach(&mut self, id: WidgetId) {
        let WidgetData {
            parent_id,
            prev_sibling_id,
            next_sibling_id,
            ..
        } = self.data[id];

        if let Some(parent) = self.data.get_mut(parent_id) {
            if parent.first_child_id == id {
                parent.first_child_id = if next_sibling_id == id {
                    WidgetId::null()
                } else {
                    next_sibling_id
                };
            }

            self.data[prev_sibling_id].next_sibling_id = next_sibling_id;
            self.data[next_sibling_id].prev_sibling_id = prev_sibling_id;
        }
        self.data[id].parent_id = WidgetId::null();
    }

    pub fn insert_root(&mut self, window_id: WindowId) -> WidgetId {
        self.data
            .insert_with_key(|id| WidgetData::new(window_id, id))
    }

    fn insert_with_parent(&mut self, window_id: WindowId, parent_id: WidgetId) -> WidgetId {
        self.data
            .insert_with_key(|id| WidgetData::new(window_id, id).with_parent(parent_id))
    }

    fn insert_with_parent_and_siblings(
        &mut self,
        window_id: WindowId,
        parent_id: WidgetId,
        prev_sibling_id: WidgetId,
        next_sibling_id: WidgetId,
    ) -> WidgetId {
        let id = self.data.insert_with_key(|id| {
            WidgetData::new(window_id, id)
                .with_parent(parent_id)
                .with_siblings(prev_sibling_id, next_sibling_id)
        });

        self.data[next_sibling_id].prev_sibling_id = id;
        self.data[prev_sibling_id].next_sibling_id = id;

        id
    }

    pub fn insert_before(&mut self, next_sibling_id: WidgetId) -> WidgetId {
        let WidgetData {
            parent_id,
            window_id,
            prev_sibling_id,
            ..
        } = self.data[next_sibling_id];
        assert!(!parent_id.is_null(), "Cannot insert before the root widget");

        let id = self.insert_with_parent_and_siblings(
            window_id,
            parent_id,
            prev_sibling_id,
            next_sibling_id,
        );

        let parent = &mut self.data[parent_id];
        if parent.first_child_id == next_sibling_id {
            parent.first_child_id = id;
        }
        id
    }

    pub fn insert_after(&mut self, prev_sibling_id: WidgetId) -> WidgetId {
        let WidgetData {
            parent_id,
            window_id,
            next_sibling_id,
            ..
        } = self.data[prev_sibling_id];
        // Maybe also assert that the sibling is not an overlay?
        assert!(!parent_id.is_null(), "Cannot insert after the root widget");
        self.insert_with_parent_and_siblings(window_id, parent_id, prev_sibling_id, next_sibling_id)
    }

    pub fn insert_first_child(&mut self, parent_id: WidgetId) -> WidgetId {
        let WidgetData {
            window_id,
            first_child_id,
            ..
        } = self.data[parent_id];

        let id = if first_child_id.is_null() {
            self.insert_with_parent(window_id, parent_id)
        } else {
            let next_sibling_id = first_child_id;
            let prev_sibling_id = self.data[first_child_id].prev_sibling_id;
            self.insert_with_parent_and_siblings(
                window_id,
                parent_id,
                prev_sibling_id,
                next_sibling_id,
            )
        };
        self.data[parent_id].first_child_id = id;
        id
    }

    pub fn insert_last_child(&mut self, parent_id: WidgetId) -> WidgetId {
        let WidgetData {
            window_id,
            first_child_id,
            ..
        } = self.data[parent_id];

        if first_child_id.is_null() {
            let id = self.insert_with_parent(window_id, parent_id);
            self.data[parent_id].first_child_id = id;
            id
        } else {
            let next_sibling_id = first_child_id;
            let prev_sibling_id = self.data[first_child_id].prev_sibling_id;
            self.insert_with_parent_and_siblings(
                window_id,
                parent_id,
                prev_sibling_id,
                next_sibling_id,
            )
        }
    }

    // Swaps the position of two widgets in the tree
    pub fn swap(&mut self, src_id: WidgetId, dst_id: WidgetId) {
        let [src, dst] = self
            .data
            .get_disjoint_mut([src_id, dst_id])
            .expect("Source and destination widgets must exist when performing a swap");
        assert!(
            dst.window_id == src.window_id,
            "Cannot swap widgets between windows"
        );
        std::mem::swap(&mut dst.window_id, &mut src.window_id);
        std::mem::swap(&mut dst.parent_id, &mut src.parent_id);
        std::mem::swap(&mut dst.first_child_id, &mut src.first_child_id);
        std::mem::swap(&mut dst.next_sibling_id, &mut src.next_sibling_id);
        std::mem::swap(&mut dst.prev_sibling_id, &mut src.prev_sibling_id);
    }

    pub fn update_node_origins(&mut self, root_widget: WidgetId, position: Point) {
        let mut stack = vec![];
        self.data[root_widget].origin = position;
        for child in self.child_id_iter(root_widget) {
            stack.push((child, position));
        }

        while let Some((widget_id, parent_origin)) = stack.pop() {
            let origin = self.data[widget_id].offset() + parent_origin.into_vec2();
            for child in self.child_id_iter(widget_id) {
                stack.push((child, origin))
            }
            self.data[widget_id].origin = origin;
        }
    }
}

impl Index<WidgetId> for WidgetTree {
    type Output = WidgetData;

    #[inline(always)]
    fn index(&self, index: WidgetId) -> &Self::Output {
        &self.data[index]
    }
}

impl IndexMut<WidgetId> for WidgetTree {
    fn index_mut(&mut self, index: WidgetId) -> &mut Self::Output {
        &mut self.data[index]
    }
}

pub struct ChildIdIter<'a> {
    inner: SiblingWalker,
    widgets: &'a WidgetTree,
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
pub struct SiblingWalker {
    first: WidgetId,
    last: WidgetId,
    done: bool,
}

impl SiblingWalker {
    fn all_siblings(widgets: &WidgetTree, first: WidgetId) -> Self {
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

    fn all_children(widgets: &WidgetTree, parent_id: WidgetId) -> Self {
        let first = widgets.data[parent_id].first_child_id;
        Self::all_siblings(widgets, first)
    }

    pub fn next_id(&mut self, widgets: &WidgetTree) -> Option<WidgetId> {
        if self.done {
            None
        } else {
            let first = self.first;
            if self.first == self.last {
                self.done = true;
            }
            self.first = widgets.data[first].next_sibling_id;
            Some(first)
        }
    }

    pub fn next_back_id(&mut self, widgets: &WidgetTree) -> Option<WidgetId> {
        if self.done {
            None
        } else {
            let last = self.last;
            if self.first == self.last {
                self.done = true;
            }
            self.last = widgets.data[last].prev_sibling_id;

            Some(last)
        }
    }
}

pub struct DFSIterator<'a> {
    tree: &'a WidgetTree,
    walker: DFSWalker,
}

impl<'a> Iterator for DFSIterator<'a> {
    type Item = WidgetId;

    fn next(&mut self) -> Option<Self::Item> {
        self.walker.next(self.tree)
    }
}

pub struct DFSWalker {
    current_id: WidgetId, // Null if done
    root_id: WidgetId,
    skip_children: bool,
}

impl DFSWalker {
    pub fn new(root_id: WidgetId) -> Self {
        Self {
            current_id: root_id,
            root_id,
            skip_children: false,
        }
    }

    pub fn next(&mut self, data_map: &WidgetTree) -> Option<WidgetId> {
        let current_id = self.current_id;
        if current_id.is_null() {
            return None;
        }

        let current = &data_map[current_id];
        let skip_children = std::mem::replace(&mut self.skip_children, false);
        if !skip_children && !current.first_child_id.is_null() {
            self.current_id = current.first_child_id;
            return Some(current_id);
        }

        let mut node_id = current_id;
        self.current_id = loop {
            let node = &data_map[node_id];
            let parent_id = node.parent_id;

            if parent_id.is_null() {
                break WidgetId::null();
            }

            let parent = &data_map[parent_id];
            if node.next_sibling_id != parent.first_child_id {
                break node.next_sibling_id;
            }

            if parent_id == self.root_id {
                break WidgetId::null();
            }

            node_id = parent_id;
        };

        Some(current_id)
    }

    pub fn skip_children(&mut self) {
        self.skip_children = true;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn insert_last_child() {
        let mut tree = WidgetTree::default();
        let root_id = tree.insert_root(WindowId::null());

        let mut children = Vec::new();
        children.push(tree.insert_last_child(root_id));
        children.push(tree.insert_last_child(root_id));
        children.push(tree.insert_last_child(root_id));
        children.push(tree.insert_last_child(root_id));

        let children2: Vec<_> = tree.child_id_iter(root_id).collect();
        assert_eq!(children, children2);
    }

    #[test]
    fn insert_first_child() {
        let mut tree = WidgetTree::default();
        let root_id = tree.insert_root(WindowId::null());

        let mut children = Vec::new();
        children.push(tree.insert_first_child(root_id));
        children.push(tree.insert_first_child(root_id));
        children.push(tree.insert_first_child(root_id));
        children.push(tree.insert_first_child(root_id));

        let children2: Vec<_> = tree.child_id_iter(root_id).rev().collect();
        assert_eq!(children, children2);
    }

    #[test]
    fn reset_node() {
        {
            let mut tree = WidgetTree::default();
            let root_id = tree.insert_root(WindowId::null());
            tree.reset(root_id, |_| {});
            assert_eq!(tree.iter().count(), 1);
        }

        {
            let mut tree = WidgetTree::default();
            let root_id = tree.insert_root(WindowId::null());
            let w1 = tree.insert_last_child(root_id);
            tree.insert_last_child(w1);
            tree.insert_last_child(w1);
            let w2 = tree.insert_last_child(root_id);
            let w21 = tree.insert_last_child(w2);
            let w22 = tree.insert_last_child(w2);

            let children = vec![root_id, w1, w2, w21, w22];
            tree.reset(w1, |_| {});

            let children2: Vec<_> = tree.dfs_iter(root_id).collect();
            assert_eq!(children, children2);
        }
    }

    #[test]
    fn remove_node() {
        {
            let mut tree = WidgetTree::default();
            let root_id = tree.insert_root(WindowId::null());
            tree.remove(root_id, |_| {});
            assert_eq!(tree.iter().count(), 0);
        }

        {
            let mut tree = WidgetTree::default();
            let root_id = tree.insert_root(WindowId::null());
            let w1 = tree.insert_last_child(root_id);
            tree.insert_last_child(w1);
            tree.insert_last_child(w1);
            let w2 = tree.insert_last_child(root_id);
            let w21 = tree.insert_last_child(w2);
            let w22 = tree.insert_last_child(w2);

            let children = vec![root_id, w2, w21, w22];
            tree.remove(w1, |_| {});

            let children2: Vec<_> = tree.dfs_iter(root_id).collect();
            assert_eq!(children, children2);
        }
    }
}
