use std::{marker::PhantomData, ops::{Deref, DerefMut}};

use bitflags::bitflags;
use slotmap::{new_key_type, Key, KeyData};

use crate::{core::{Point, Rectangle, Size}, view::Widget};

use super::{contexts::BuildContext, AppState, RenderContext, WindowId};

new_key_type! {
    pub struct WidgetId;
}

impl From<taffy::NodeId> for WidgetId {
    fn from(value: taffy::NodeId) -> Self {
        Self::from(KeyData::from_ffi(value.into()))
    }
}

impl Into<taffy::NodeId> for WidgetId {
    fn into(self) -> taffy::NodeId {
        self.0.as_ffi().into()
    }
}

bitflags!(
    #[derive(Debug, Clone, Copy)]
    pub struct WidgetFlags : u32 {
        const EMPTY = 0;
        const NEEDS_LAYOUT = 1 << 1;
        const NEEDS_RENDER = 1 << 2;
        const NEEDS_REBUILD = 1 << 3;
        const FOCUSABLE = 1 << 4;

        const UNDER_MOUSE_CURSOR = 1 << 8;
    }
);

pub struct WidgetData {
    pub(super) id: WidgetId,
    pub(super) window_id: WindowId,
    pub(super) parent_id: WidgetId,
    pub(super) children: Vec<WidgetId>,
    pub(super) style: taffy::Style,
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

    pub fn local_bounds(&self) -> Rectangle {
        Rectangle::new(self.offset(), self.size())
    }

    pub fn global_bounds(&self) -> Rectangle {
        Rectangle::new(self.origin(), self.size())
    }

    pub fn set_flag(&mut self, flag: WidgetFlags) {
        self.flags |= flag;
    }

    pub fn clear_flag(&mut self, flag: WidgetFlags) {
        self.flags &= !flag;
    }

    pub fn flag_is_set(& self, flag: WidgetFlags) -> bool{
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

    pub fn with_style(mut self, f: impl Fn(&mut taffy::Style)) -> Self {
        f(&mut self.style);
        self
    }
}

pub struct WidgetNode {
    pub(crate) widget: Box<dyn Widget>,
    pub(crate) data: WidgetData,
}

impl WidgetNode {
    pub fn data(&self) -> &WidgetData {
        &self.data
    }

    pub fn layout_requested(&self) -> bool {
        self.data.flag_is_set(WidgetFlags::NEEDS_LAYOUT)
    }
}

pub struct WidgetRef<'a, W: 'a + Widget + ?Sized> {
    pub(super) id: WidgetId,
    pub(super) app_state: &'a AppState,
    _phantom: PhantomData<&'a W>
}

impl<'a, W: 'a + Widget + ?Sized> WidgetRef<'a, W> {
    pub(super) fn new(id: WidgetId, app_state: &'a AppState) -> Self {
        Self {
            id,
            app_state,
            _phantom: PhantomData
        }
    }

    pub fn data(&self) -> &WidgetData {
        self.app_state.widget_data.get(self.id).unwrap()
    }

    pub fn local_bounds(&self) -> Rectangle {
        self.data().local_bounds()
    }

    pub fn global_bounds(&self) -> Rectangle {
        self.data().global_bounds()
    }

    pub fn layout_requested(&self) -> bool {
        self.data().flag_is_set(WidgetFlags::NEEDS_LAYOUT)
    }
}

pub struct WidgetMut<'a, W: 'a + Widget + ?Sized> {
    pub(super) id: WidgetId,
    pub(super) app_state: &'a AppState,
    _phantom: PhantomData<&'a mut W>
}

impl<'a, W: 'a + Widget + ?Sized> WidgetMut<'a, W> {
    pub(super) fn new(id: WidgetId, app_state: &'a AppState) -> Self {
        Self {
            id,
            app_state,
            _phantom: PhantomData
        }
    }

    pub fn data(&self) -> &WidgetData {
        self.app_state.widget_data.get(self.id).unwrap()
    }

    pub fn data_mut(&self) -> &mut WidgetData {
        self.app_state.widget_data.get_mut(self.id).unwrap()
    }

    pub fn local_bounds(&self) -> Rectangle {
        self.data().local_bounds()
    }

    pub fn global_bounds(&self) -> Rectangle {
        self.data().global_bounds()
    }

    pub fn request_layout(&mut self) {
        self.data().set_flag(WidgetFlags::NEEDS_LAYOUT);
    }

    pub fn layout_requested(&self) -> bool {
        self.data().flag_is_set(WidgetFlags::NEEDS_LAYOUT)
    }

    pub fn add_child<'s, W2: Widget + Sized>(&'s mut self, f: impl FnOnce(&mut BuildContext) -> W2) -> WidgetMut<'s, W2> {
        let id = self.app_state.add_widget(self.id, f);
        WidgetMut {
            id,
            app_state: self.app_state,
            _phantom: PhantomData
        }
    }
}

impl<'a, W: 'a + Widget> Deref for WidgetMut<'a, W> {
    type Target = W;

    fn deref(&self) -> &Self::Target {
        self.app_state.widgets.get(self.id).and_then(|x| x.as_any().downcast_ref()).unwrap()
    }
}

impl<'a, W: 'a + Widget> DerefMut for WidgetMut<'a, W> {    
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.app_state.widgets.get_mut(self.id).and_then(|x| x.as_any().downcast_mut()).unwrap()
    }
}