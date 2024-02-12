use std::{marker::PhantomData, rc::{Weak, Rc}, cell::RefCell, sync::atomic::AtomicUsize, any::Any, collections::HashSet};
use slotmap::{new_key_type, SlotMap, SecondaryMap, Key};

new_key_type! { 
    pub struct NodeId;
} 

struct Node {
    node_type: NodeType,
    state: NodeState
}

impl Node {
    fn get_value_as<T: Any>(&self) -> Option<&T> {
        match &self.node_type {
            NodeType::Signal { value } => value.downcast_ref(),
            NodeType::Memo { value, .. } => value.as_ref().and_then(|value| value.downcast_ref()),
            NodeType::Effect { .. } => None,
        }
    }
}

enum NodeType {
    Signal {
        value: Box<dyn Any>
    },
    Memo {
        f: Box<dyn Fn(&mut AppContext) -> Box<dyn Any>>,
        value: Option<Box<dyn Any>>,
    },
    Effect {
        f: Box<dyn Fn(&mut AppContext)>,
    }
}

enum NodeState {
    /// Reactive value is valid, no need to recompute
    Clean,
    /// Reactive value might be stale, check parent nodes to decide whether to recompute
    Check,
    /// Reactive value is invalid, parents have changed, valueneeds to be recomputed
    Dirty
}

pub trait SignalGet {
    type Value;

    /// Map the current value using `f` and subscribe to changes
    fn map_ref<R>(&self, cx: &mut AppContext, f: impl Fn(&Self::Value) -> R) -> R;

    /// Get the current value and subscribe to changes
    fn get(&self, cx: &mut AppContext) -> Self::Value 
        where Self::Value: Clone 
    {
        self.map_ref(cx, Self::Value::clone)
    }

    fn map_ref_untracked<R>(&self, cx: &AppContext, f: impl Fn(&Self::Value) -> R) -> R;

    fn get_untracked(&self, cx: &AppContext) -> Self::Value 
        where Self::Value: Clone 
    {
        self.map_ref_untracked(cx, Self::Value::clone)
    }
}

pub trait SignalSet {
    type Value;

    /// Sets the current value without notifying subscribers
    fn set_untracked(&self, cx: &mut AppContext, value: Self::Value) {
        self.set_with_untracked(cx, move || value)
    }

    /// Sets the current value without notifying subscribers
    fn set_with_untracked(&self, cx: &mut AppContext, f: impl FnOnce() -> Self::Value);

    /// Set the current value, notifies subscribers
    fn set(&self, cx: &mut AppContext, value: Self::Value) {
        self.set_with(cx, move || value)
    }

    /// Set the current value, notifies subscribers
    fn set_with(&self, cx: &mut AppContext, f: impl FnOnce() -> Self::Value);
}

new_key_type! { 
    pub struct SignalId; 
    pub struct MemoId;
    pub struct EffectId;
}

#[derive(Clone, Copy)]
pub struct Signal<T> {
    id: NodeId,
    _marker: PhantomData<T>
}

impl<T: Any> Signal<T> {
    pub fn update(&self, cx: &mut AppContext, f: impl Fn(&T) -> T) {
        let new_value = self.map_ref_untracked(cx, f);
        self.set(cx, new_value);
    }
}

impl<T: Any> SignalSet for Signal<T> {
    type Value = T;

    fn set_with(&self, cx: &mut AppContext, f: impl FnOnce() -> Self::Value) {
        cx.set_signal_value(self, f())
    }

    fn set_with_untracked(&self, cx: &mut AppContext, f: impl FnOnce() -> Self::Value) {
        cx.set_signal_value_untracked(self, f())
    }
}

impl<T: 'static> SignalGet for Signal<T> {
    type Value = T;

    fn map_ref<R>(&self, cx: &mut AppContext, f: impl Fn(&T) -> R) -> R {
        f(cx.get_signal_value_ref(self))
    }

    fn map_ref_untracked<R>(&self, cx: &AppContext, f: impl Fn(&Self::Value) -> R) -> R {
        f(cx.get_signal_value_ref_untracked(self))
    }
}

/*impl<T> Clone for Signal<T> {
    fn clone(&self) -> Self {
        if let Some(ref_counts) = self.ref_counts.upgrade() {
            RefCounts::retain(&mut ref_counts.borrow_mut(), self.id);
        }

        Self { 
            id: self.id.clone(), 
            ref_counts: self.ref_counts.clone(), 
            _marker: self._marker.clone() 
        }
    }
}

impl<T> Drop for Signal<T> {
    fn drop(&mut self) {
        if let Some(ref_counts) = self.ref_counts.upgrade() {
            RefCounts::release(&ref_counts, self.id)
        }
    }
}*/

pub struct Memo<T> {
    id: NodeId,
    _marker: PhantomData<T>
}

impl<T> SignalGet for Memo<T> {
    type Value = T;

    fn map_ref<R>(&self, ctx: &mut AppContext, f: impl Fn(&Self::Value) -> R) -> R {
        todo!()
    }

    fn map_ref_untracked<R>(&self, ctx: &AppContext, f: impl Fn(&Self::Value) -> R) -> R {
        todo!()
    }
}

struct RefCounts<K: Key> {
    counts: SlotMap<K, AtomicUsize>,
    dropped_ids: Vec<K>
}

impl<K: Key> RefCounts<K> {
    fn alloc_id(this: &RefCell<Self>) -> K {
        let mut this = this.borrow_mut();
        this.counts.insert(1.into())
    }

    fn clear_dropped(&mut self) -> Vec<K> {
        let dropped_ids = std::mem::take(&mut self.dropped_ids);
        for id in dropped_ids.iter() {
            self.counts.remove(*id);
        }
        dropped_ids
    }

    /// Increment the reference count for a value
    /// 
    /// # Panics
    /// 
    /// Panics if no value exists for the given id.
    fn retain(&mut self, id: K) -> usize {
        let count = self.counts.get(id)
            .expect("Tried to retain a dropped Signal");
        count.fetch_add(1, std::sync::atomic::Ordering::SeqCst)
    }

    fn release(this: &RefCell<Self>, id: K) {
        let last_count = {
            let signal_map = this.borrow();
            let count = signal_map.counts.get(id)
                .expect("Signal should not be dropped");
            count.fetch_sub(1, std::sync::atomic::Ordering::SeqCst)
        };
        
        if last_count == 1 {
            let mut signal_map = this.borrow_mut();
            signal_map.dropped_ids.push(id);
        }
    }
}

impl<K: Key> Default for RefCounts<K> {
    fn default() -> Self {
        Self { 
            counts: Default::default(), 
            dropped_ids: Default::default() 
        }
    }
}

struct SignalState {
    value: Box<dyn Any>
}

impl SignalState {
    fn new<T: Any>(value: T) -> Self {
        Self {
            value: Box::new(value)
        }
    }
}

struct MemoState {
    f: Box<dyn Fn(&mut AppContext) -> Box<dyn Any>>,
    value: Option<Box<dyn Any>>,
    subscribers: HashSet<SubscriberId>,
    dependencies: HashSet<SourceId>
}

impl MemoState {
    fn new() -> Self {
        todo!()
    }
}

struct EffectState {
    id: NodeId,
    f: Box<dyn Fn(&mut AppContext)>,
    dependencies: HashSet<SourceId>
}

impl EffectState {
    fn new(id: NodeId, f: impl Fn(&mut AppContext) + 'static) -> Self {
        Self {
            id,
            f: Box::new(f),
            dependencies: Default::default()
        }
    }

    fn run(&self, cx: &mut AppContext) {
        let old_scope = cx.scope;
        cx.scope = Scope::Effect(self.id);
        (self.f)(cx);
        cx.scope = old_scope;
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
enum SubscriberId {
    Memo(MemoId),
    Effect(EffectId)
}

enum SourceId {
    Memo(MemoId),
    Effect(EffectId)
}

#[derive(Debug, Clone, Copy)]
enum Scope {
    Root,
    Memo(NodeId),
    Effect(NodeId)
}

pub struct AppContext {
    scope: Scope,
    pending_effects: Vec<NodeId>,
    nodes: SlotMap<NodeId, Node>,
    subscriptions: SecondaryMap<NodeId, HashSet<NodeId>>,
    dependencies: SecondaryMap<NodeId, HashSet<NodeId>>,
}

impl AppContext {
    pub fn new() -> Self {
        Self {
            scope: Scope::Root,
            pending_effects: Default::default(),
            nodes: Default::default(),
            subscriptions: Default::default(),
            dependencies: Default::default()
        }
    }

    pub fn create_signal<T: Any>(&mut self, value: T) -> Signal<T> {
        let node_type = NodeType::Signal { value: Box::new(value) };
        let id = self.create_node(node_type, NodeState::Clean);
        Signal { id, _marker: PhantomData }
    }

    pub fn create_memo<T: PartialEq + 'static>(&mut self, f: impl Fn(&mut Self) -> T + 'static) -> Memo<T> {
        let node_type = NodeType::Memo { f: Box::new(move |cx| Box::new(f(cx))), value: None };
        let id = self.create_node(node_type, NodeState::Check);
        Memo { id, _marker: PhantomData }
    }

    pub fn create_effect(&mut self, f: impl Fn(&mut Self) + 'static) {
        let id = self.create_node(NodeType::Effect { f: Box::new(f) }, NodeState::Clean);
        //let effect = EffectState::new(id, f);
        //self.pending_effects.push(effect);
        self.flush_effects();
    }

    fn create_node(&mut self, node_type: NodeType, state: NodeState) -> NodeId {
        let node = Node { node_type, state };
        let id = self.nodes.insert(node);
        self.subscriptions.insert(id, HashSet::new());
        self.dependencies.insert(id, HashSet::new());
        id
    }

    fn remove_node(&mut self, id: NodeId) -> Node{
        // Remove the node's subscriptions to other nodes
        let node_dependencies = self.dependencies.remove(id).expect("Missing dependencies for node");
        for node_id in node_dependencies {
            self.subscriptions[node_id].remove(&id);
        }

        // Remove other nodes' subscriptions to this node
        let node_subscriptions = self.subscriptions.remove(id).expect("Missing subscriptions for node");
        for node_id in node_subscriptions {
            self.dependencies[node_id].remove(&id);
        }

        self.nodes.remove(id).expect("Missing node")
    }

    fn set_signal_value_untracked<T: Any>(&mut self, signal: &Signal<T>, new_value: T) {
        let signal = self.nodes.get_mut(signal.id).expect("No signal found");
        match &mut signal.node_type {
            NodeType::Signal { ref value } => {
                //value = Box::new(new_value);
            },
            _ => unreachable!()
        }
    }

    fn set_signal_value<T: Any>(&mut self, signal: &Signal<T>, value: T) {
        self.set_signal_value_untracked(signal, value);
        
        // Take all the current subscribers, the effects will resubscribe when evaluated
        let subscribers = std::mem::take(&mut self.subscriptions[signal.id]);
        subscribers.iter().for_each(|subscriber| {
            self.notify(subscriber);
        });
        //std::mem::swap(&mut subscribers, &mut self.signal_subscriptions[signal.id]);
    }

    fn notify(&mut self, node_id: &NodeId) {
        let node = self.nodes.get_mut(*node_id).expect("Node has been removed");
        match &node.node_type {
            NodeType::Signal { value } => todo!(),
            NodeType::Memo { f, value } => todo!(),
            NodeType::Effect { f } => todo!(),
        }
        /*{
            if let Some(effect_state) = std::mem::take(&mut self.effects[*id]) {
                self.pending_effects.push(effect_state);
                self.flush_effects();
            }
        }*/
    }

    fn get_signal_value_ref_untracked<T: Any>(&self, signal: &Signal<T>) -> &T {
        let node = self.nodes.get(signal.id).expect("No Signal found");
        match &node.node_type {
            NodeType::Signal { value } => value.downcast_ref().expect("Node had wrong value type"),
            _ => unreachable!()
        }
    }

    fn get_signal_value_ref<T: Any>(&mut self, signal: &Signal<T>) -> &T {
        match self.scope {
            Scope::Root => { 
                todo!()
            },
            Scope::Memo(_) => todo!(),
            Scope::Effect(node_id) => {
                self.subscriptions[signal.id].insert(node_id);
            },
        }
        
        self.get_signal_value_ref_untracked(signal)
    }

    fn flush_effects(&mut self) {
        while let Some(effect_state) = self.pending_effects.pop() {
            /*let id = effect_state.id;
            effect_state.run(self);
            // Put it back in the effects map so it can be executed again
            self.effects[id] = Some(effect_state);*/
        }
    }

    /*pub fn create_stateful_effect<S>(&mut self, f_init: impl FnOnce() -> S, f: impl Fn(S) -> S) {

    }*/
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn effects_execute_upon_creation() {
        let mut cx = AppContext::new();
        let signal = cx.create_signal(0);
        {
            let signal = signal.clone();
            cx.create_effect(move |cx| {
                signal.set_untracked(cx, 1);
            });
        }
        
        assert_eq!(signal.get_untracked(&cx), 1);
    }

    #[test]
    fn effects_execute_when_signal_changes() {
        let mut cx = AppContext::new();
        let source_signal = cx.create_signal(0);
        let dest_signal = cx.create_signal(0);
        cx.create_effect(move |cx| {
            let new_value = source_signal.get(cx);
            dest_signal.set_untracked(cx, new_value);
        });

        source_signal.set(&mut cx, 1);
        assert_eq!(dest_signal.get_untracked(&cx), 1);

        source_signal.set(&mut cx, 2);
        assert_eq!(dest_signal.get_untracked(&cx), 2);
    }
}