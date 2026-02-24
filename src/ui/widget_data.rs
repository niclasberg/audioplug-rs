use std::{
    cell::Cell,
    ops::{Index, IndexMut},
};

use bitflags::bitflags;
use slotmap::{Key, KeyData, SlotMap, new_key_type};

use crate::{
    core::{Point, Rect, RoundedRect, Shape, Size, Zero},
    ui::Scene,
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

pub struct WidgetData {
    pub(super) id: WidgetId,
    pub(super) window_id: WindowId,
    pub(super) parent_id: WidgetId,
    pub(super) first_child_id: WidgetId,
    pub(super) next_sibling_id: WidgetId,
    pub(super) prev_sibling_id: WidgetId,
    pub(super) style: Style,
    pub(super) cache: taffy::Cache,
    pub(super) layout: taffy::Layout,
    flags: Cell<WidgetFlags>,
    pub(super) origin: Point,
    pub(super) scene: Scene,
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
            style: Default::default(),
            cache: Default::default(),
            layout: Default::default(),
            flags: Cell::new(WidgetFlags::EMPTY),
            origin: Point::ZERO,
            scene: Scene::default(),
        }
    }

    pub fn reset(&mut self) {
        self.flags.set(WidgetFlags::EMPTY);
        self.cache.clear();
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
pub struct WidgetDataMap {
    data: SlotMap<WidgetId, WidgetData>,
}

impl WidgetDataMap {
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

    /// Iterator over the ids of all siblings of a node
    pub fn sibling_id_iter(&self, widget_id: WidgetId) -> ChildIdIter<'_> {
        ChildIdIter {
            inner: SiblingWalker::all_siblings(&self, widget_id),
            widgets: &self,
        }
    }

    /// Iterator over the ids of all children of a node
    pub fn child_id_iter(&self, widget_id: WidgetId) -> ChildIdIter<'_> {
        ChildIdIter {
            inner: SiblingWalker::all_children(&self, widget_id),
            widgets: &self,
        }
    }

    /// Removes a widget without updating any sibling/parent links. Only used as an optimization when removing
    /// a
    pub(super) fn unchecked_remove(&mut self, id: WidgetId) -> Option<WidgetData> {
        self.data.remove(id)
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
        // Maybe also assert that the sibling is not an overlay?
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

impl Index<WidgetId> for WidgetDataMap {
    type Output = WidgetData;

    #[inline(always)]
    fn index(&self, index: WidgetId) -> &Self::Output {
        &self.data[index]
    }
}

impl IndexMut<WidgetId> for WidgetDataMap {
    fn index_mut(&mut self, index: WidgetId) -> &mut Self::Output {
        &mut self.data[index]
    }
}

pub struct ChildIdIter<'a> {
    inner: SiblingWalker,
    widgets: &'a WidgetDataMap,
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
pub(super) struct SiblingWalker {
    first: WidgetId,
    last: WidgetId,
    done: bool,
}

impl SiblingWalker {
    pub(super) fn all_siblings(widgets: &WidgetDataMap, first: WidgetId) -> Self {
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

    pub(super) fn all_children(widgets: &WidgetDataMap, parent_id: WidgetId) -> Self {
        let first = widgets.data[parent_id].first_child_id;
        Self::all_siblings(widgets, first)
    }

    pub(super) fn next_id(&mut self, widgets: &WidgetDataMap) -> Option<WidgetId> {
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

    pub(super) fn next_back_id(&mut self, widgets: &WidgetDataMap) -> Option<WidgetId> {
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

pub struct DepthFirstWalker {
    current: WidgetId, // Null if done
    root: WidgetId,
}

impl DepthFirstWalker {
    pub fn new(root: WidgetId) -> Self {
        Self {
            current: root,
            root,
        }
    }

    fn next(&mut self, data_map: &WidgetDataMap) -> Option<WidgetId> {
        if self.current.is_null() {
            return None;
        }

        let current_id = self.current;
        let current = &data_map[current_id];
        self.current = if !current.first_child_id.is_null() {
            current.first_child_id
        } else if current_id != data_map[current.parent_id].first_child_id {
            current.next_sibling_id
        } else {
            // Climb back until first parent with sibling, that is the new current
            todo!()
        };

        Some(current_id)
    }
}
