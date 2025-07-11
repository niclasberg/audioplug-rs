use bitflags::bitflags;
use slotmap::{new_key_type, Key, KeyData};

use crate::{
    core::{Point, Rectangle, RoundedRectangle, Size},
    style::Style,
};

use super::{Shape, WindowId};

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

        const FOCUSABLE = 1 << 4;
        const OVERLAY = 1 << 5;

        const UNDER_MOUSE_CURSOR = 1 << 8;
    }
);

pub struct WidgetData {
    pub(super) id: WidgetId,
    pub(super) window_id: WindowId,
    pub(super) parent_id: WidgetId,
    /// Children, excluding overlays
    pub(super) children: Vec<WidgetId>,
    pub(super) overlays: Vec<WidgetId>,
    pub(super) style: Style,
    pub(super) cache: taffy::Cache,
    pub(super) layout: taffy::Layout,
    pub(super) flags: WidgetFlags,
    pub(super) origin: Point,
}

impl WidgetData {
    pub fn new(window_id: WindowId, id: WidgetId) -> Self {
        Self {
            id,
            window_id,
            parent_id: WidgetId::null(),
            children: Vec::new(),
            overlays: Vec::new(),
            style: Default::default(),
            cache: Default::default(),
            layout: Default::default(),
            flags: WidgetFlags::EMPTY,
            origin: Point::ZERO,
        }
    }

    pub fn with_parent(mut self, parent_id: WidgetId) -> Self {
        self.parent_id = parent_id;
        self
    }

    pub fn id(&self) -> WidgetId {
        self.id
    }

    /// Local bounds of the widget, relative to its parent
    pub fn local_bounds(&self) -> Rectangle {
        Rectangle::from_origin(self.offset(), self.size())
    }

    /// Bounds of the widget, in global coords, including borders and padding
    pub fn global_bounds(&self) -> Rectangle {
        Rectangle::from_origin(self.origin(), self.size())
    }

    fn subtract_padding_and_border(&self, rect: Rectangle) -> Rectangle {
        Rectangle::from_ltrb(
            rect.left() + (self.layout.border.left + self.layout.padding.left) as f64,
            rect.top() + (self.layout.border.top + self.layout.padding.top) as f64,
            rect.right() - (self.layout.border.right + self.layout.padding.right) as f64,
            rect.bottom() - (self.layout.border.bottom + self.layout.padding.bottom) as f64,
        )
    }

    /// Bounds of the widget, in global coords, excluding borders and padding
    pub fn content_bounds(&self) -> Rectangle {
        self.subtract_padding_and_border(self.global_bounds())
    }

    pub fn border(&self) -> Rectangle {
        Rectangle::from_ltrb(
            self.layout.border.left as f64,
            self.layout.border.top as f64,
            self.layout.border.right as f64,
            self.layout.border.bottom as f64,
        )
    }

    pub fn padding(&self) -> Rectangle {
        Rectangle::from_ltrb(
            self.layout.padding.left as f64,
            self.layout.padding.top as f64,
            self.layout.padding.right as f64,
            self.layout.padding.bottom as f64,
        )
    }

    pub fn set_or_clear_flag(&mut self, flag: WidgetFlags, set: bool) {
        if set {
            self.flags |= flag;
        } else {
            self.flags &= !flag;
        }
    }

    pub fn set_flag(&mut self, flag: WidgetFlags) {
        self.flags |= flag;
    }

    pub fn clear_flag(&mut self, flag: WidgetFlags) {
        self.flags &= !flag;
    }

    #[inline(always)]
    pub fn flag_is_set(&self, flag: WidgetFlags) -> bool {
        self.flags.contains(flag)
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
            RoundedRectangle::new(self.global_bounds(), self.style.corner_radius).into()
        }
    }
}
