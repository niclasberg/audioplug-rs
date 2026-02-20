use std::cell::Cell;

use bitflags::bitflags;
use slotmap::{Key, KeyData, new_key_type};

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
    pub(super) first_overlay_id: WidgetId,
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
            first_overlay_id: WidgetId::null(),
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
