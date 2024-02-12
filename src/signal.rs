use std::{marker::PhantomData, any::Any, collections::HashSet};
use slotmap::{new_key_type, SlotMap, SecondaryMap};

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
            NodeType::Signal(signal) => signal.value.downcast_ref(),
            NodeType::Memo(memo) => memo.value.as_ref().and_then(|value| value.downcast_ref()),
            NodeType::Effect(_) => None,
        }
    }
}

enum NodeType {
    Signal(SignalState),
    Memo(MemoState),
    Effect(EffectState)
}

struct SignalState  {
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
}

struct EffectState {
	f: Box<dyn Fn(&mut AppContext)>,
}

enum NodeState {
    /// Reactive value is valid, no need to recompute
    Clean,
    /// Reactive value might be stale, check parent nodes to decide whether to recompute
    Check,
    /// Reactive value is invalid, parents have changed, value needs to be recomputed
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
        let state = SignalState::new(value);
        let id = self.create_node(NodeType::Signal(state), NodeState::Clean);
        Signal { id, _marker: PhantomData }
    }

    pub fn create_memo<T: PartialEq + 'static>(&mut self, f: impl Fn(&mut Self) -> T + 'static) -> Memo<T> {
        let state = MemoState { f: Box::new(move |cx| Box::new(f(cx))), value: None };
        let id = self.create_node(NodeType::Memo(state), NodeState::Check);
        Memo { id, _marker: PhantomData }
    }

    pub fn create_effect(&mut self, f: impl Fn(&mut Self) + 'static) {
		let state = EffectState { f: Box::new(f) };
        let id = self.create_node(NodeType::Effect(state), NodeState::Clean);
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
            NodeType::Signal(signal) => {
				signal.value = Box::new(new_value);
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

	fn add_subscription(&mut self, src_id: NodeId, dst_id: NodeId) {

	}

    fn notify(&mut self, node_id: &NodeId) {
        let node = self.nodes.get_mut(*node_id).expect("Node has been removed");
        match &node.node_type {
            NodeType::Signal(signal) => todo!(),
            NodeType::Memo(memo) => todo!(),
            NodeType::Effect(effect) => todo!(),
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
            NodeType::Signal(signal) => signal.value.downcast_ref().expect("Node had wrong value type"),
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