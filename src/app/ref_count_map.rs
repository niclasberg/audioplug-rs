use std::{cell::RefCell, rc::Weak};

use super::NodeId;
use slotmap::SecondaryMap;

pub struct RefCountMap {
    refs: SecondaryMap<NodeId, usize>,
    nodes_to_remove: Vec<NodeId>,
}

pub type WeakRefCountMap = Weak<RefCell<RefCountMap>>;

impl RefCountMap {
    pub(super) fn new() -> Self {
        Self {
            refs: SecondaryMap::new(),
            nodes_to_remove: Vec::new(),
        }
    }

    pub(super) fn take_nodes_to_remove(&mut self) -> Vec<NodeId> {
        std::mem::take(&mut self.nodes_to_remove)
    }

    pub(super) fn insert(this: &WeakRefCountMap, key: NodeId) {
        this.upgrade().unwrap().borrow_mut().refs.insert(key, 1);
    }

    pub(super) fn remove(this: &WeakRefCountMap, key: NodeId) {
        if let Some(this) = this.upgrade() {
            let mut this = this.borrow_mut();
            this.nodes_to_remove.push(key);
            this.refs.remove(key);
        }
    }

    pub(super) fn increment_ref_count(this: &WeakRefCountMap, key: NodeId) {
        if let Some(this) = this.upgrade() {
            let mut this = this.borrow_mut();
            let ref_count = this
                .refs
                .get_mut(key)
                .expect("Could not increment ref count, node is deleted");
            *ref_count += 1;
        }
    }

    pub(super) fn decrement_ref_count(this: &WeakRefCountMap, key: NodeId) {
        if let Some(this) = this.upgrade() {
            let mut this = this.borrow_mut();
            let ref_count = this
                .refs
                .get_mut(key)
                .expect("Could not decrement ref count, node is deleted");
            *ref_count -= 1;
            if *ref_count == 0 {
                this.nodes_to_remove.push(key);
            }
        }
    }
}
