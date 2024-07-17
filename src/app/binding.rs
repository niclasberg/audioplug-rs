use std::{any::Any, rc::Rc};

use crate::{view::WidgetNode, IdPath};

use super::{AppState, NodeId, RefCountMap, WeakRefCountMap};

pub struct Binding {
    pub(super) id: NodeId,
    ref_count_map: WeakRefCountMap
}

impl Binding {
    pub(super) fn new(id: NodeId, ref_count_map: WeakRefCountMap) -> Self {
        RefCountMap::insert(&ref_count_map, id);
        Self {
            id,
            ref_count_map
        }
    }
}

impl Drop for Binding {
    fn drop(&mut self) {
        RefCountMap::remove(&self.ref_count_map, self.id);
    }
}

pub(super) struct BindingState {
    pub widget_id: IdPath,
    pub f: Rc<Box<dyn Fn(&mut AppState, &mut WidgetNode)>>,
}

impl BindingState {
    pub(super) fn new(widget_id: IdPath, f: impl Fn(&mut AppState, &mut WidgetNode) + 'static) -> Self {
        Self {
            widget_id,
            f: Rc::new(Box::new(f))
        }
    }
}