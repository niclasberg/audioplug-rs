use std::{any::Any, collections::HashSet, marker::PhantomData, rc::{Rc, Weak}};
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

enum NodeState {
    /// Reactive value is valid, no need to recompute
    Clean,
    /// Reactive value might be stale, check parent nodes to decide whether to recompute
    Check,
    /// Reactive value is invalid, parents have changed, value needs to be recomputed
    Dirty
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
	f: Rc<Box<dyn Fn(&mut AppContext)>>
}

impl EffectState {
    fn new(f: impl Fn(&mut AppContext) + 'static) -> Self {
        Self {
            f: Rc::new(Box::new(f))
        }
    }
}

enum Task {
    RunEffect {
        id: NodeId,
        f: Weak<Box<dyn Fn(&mut AppContext)>>
    }
}

impl Task {
    fn run(&self, cx: &mut AppContext) {
        match self {
            Task::RunEffect { id, f } => {
                cx.with_scope(Scope::Effect(*id), |cx| {
                    if let Some(f) = f.upgrade() {
                        f(cx)
                    }
                })
            },
        }
    }
}

pub trait SignalGet {
    type Value;

    /// Map the current value using `f` and subscribe to changes
    fn with_ref<R>(&self, cx: &mut AppContext, f: impl Fn(&Self::Value) -> R) -> R;

    /// Get the current value and subscribe to changes
    fn get(&self, cx: &mut AppContext) -> Self::Value 
        where Self::Value: Clone 
    {
        self.with_ref(cx, Self::Value::clone)
    }

    fn with_ref_untracked<R>(&self, cx: &AppContext, f: impl Fn(&Self::Value) -> R) -> R;

    fn get_untracked(&self, cx: &AppContext) -> Self::Value 
        where Self::Value: Clone 
    {
        self.with_ref_untracked(cx, Self::Value::clone)
    }

    fn map<F, R>(self, f: F) -> Map<Self, F>
    where 
        Self: Sized,
        F: Fn(&Self::Value) -> R
    {
        Map {
            source: self,
            f
        }
    }
}

#[derive(Clone, Copy)]
pub struct Map<S, F> {
    source: S,
    f: F
}

impl<B, S: SignalGet, F> SignalGet for Map<S, F> 
where
    F: Fn(&S::Value) -> B
{
    type Value = B;

    fn with_ref<R>(&self, cx: &mut AppContext, f: impl Fn(&Self::Value) -> R) -> R {
        f(&self.source.with_ref(cx, |x| (self.f)(x)))
    }

    fn with_ref_untracked<R>(&self, cx: &AppContext, f: impl Fn(&Self::Value) -> R) -> R {
        f(&self.source.with_ref_untracked(cx, |x| (self.f)(x)))
    }
}

pub trait SignalSet {
    type Value;

    /// Set the current value, notifies subscribers
    fn set(&self, cx: &mut AppContext, value: Self::Value) {
        self.set_with(cx, move || value)
    }

    /// Set the current value, notifies subscribers
    fn set_with(&self, cx: &mut AppContext, f: impl FnOnce() -> Self::Value);
}

pub trait SignalUpdate {
    type Value;

    /// Set the current value, notifies subscribers
    fn update(&self, cx: &mut AppContext, f: impl FnOnce(&mut Self::Value));
}

impl<T: AsRef<T>> SignalGet for T {
    type Value = T;

    fn with_ref<R>(&self, _cx: &mut AppContext, f: impl FnOnce(&Self::Value) -> R) -> R {
        f(&self)
    }

    fn with_ref_untracked<R>(&self, _cx: &AppContext, f: impl FnOnce(&Self::Value) -> R) -> R {
        f(&self)
    }
}

#[derive(Clone, Copy)]
pub struct Signal<T> {
    id: NodeId,
    _marker: PhantomData<T>
}

impl<T: Any> Signal<T> {
    pub fn update(&self, cx: &mut AppContext, f: impl Fn(&T) -> T) {
        let new_value = self.with_ref_untracked(cx, f);
        self.set(cx, new_value);
    }
}

impl<T: Any> SignalSet for Signal<T> {
    type Value = T;

    fn set_with(&self, cx: &mut AppContext, f: impl FnOnce() -> Self::Value) {
        cx.set_signal_value(self, f())
    }
}

impl<T: Any> SignalUpdate for Signal<T> {
    type Value = T;

    fn update(&self, cx: &mut AppContext, f: impl FnOnce(&mut Self::Value)) {
        cx.update_signal_value(self, f)
    }
}

impl<T: 'static> SignalGet for Signal<T> {
    type Value = T;

    fn with_ref<R>(&self, cx: &mut AppContext, f: impl FnOnce(&T) -> R) -> R {
        f(cx.get_signal_value_ref(self))
    }

    fn with_ref_untracked<R>(&self, cx: &AppContext, f: impl FnOnce(&Self::Value) -> R) -> R {
        f(cx.get_signal_value_ref_untracked(self))
    }
}

pub struct Memo<T> {
    id: NodeId,
    _marker: PhantomData<T>
}

impl<T: 'static> SignalGet for Memo<T> {
    type Value = T;

    fn with_ref<R>(&self, ctx: &mut AppContext, f: impl Fn(&Self::Value) -> R) -> R {
        f(ctx.get_memo_value_ref(self))
    }

    fn with_ref_untracked<R>(&self, ctx: &AppContext, f: impl Fn(&Self::Value) -> R) -> R {
        f(ctx.get_memo_value_ref_untracked(self))
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
    pending_tasks: Vec<Task>,
    nodes: SlotMap<NodeId, Node>,
    subscriptions: SecondaryMap<NodeId, HashSet<NodeId>>,
    dependencies: SecondaryMap<NodeId, HashSet<NodeId>>,
}

impl AppContext {
    pub fn new() -> Self {
        Self {
            scope: Scope::Root,
            pending_tasks: Default::default(),
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
		let state = EffectState::new(f);
        let id = self.create_node(NodeType::Effect(state), NodeState::Clean);
        self.notify(&id);
    }

    fn run_effect(&mut self, node_id: &NodeId, effect: &EffectState) {
        
    }

    fn with_scope<R>(&mut self, scope: Scope, f: impl FnOnce(&mut Self) -> R) -> R {
        let old_scope = self.scope;
        self.scope = scope;
        let value = f(self);
        self.scope = old_scope;
        value
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

    fn set_signal_value<T: Any>(&mut self, signal: &Signal<T>, value: T) {
        self.update_signal_value(signal, move |x| { *x = value });
    }

    fn update_signal_value<T: Any>(&mut self, signal: &Signal<T>, f: impl FnOnce(&mut T)) {
        {
            let signal = self.nodes.get_mut(signal.id).expect("No signal found");
            match &mut signal.node_type {
                NodeType::Signal(signal) => {
                    let mut value = signal.value.downcast_mut().expect("Invalid signal value type");
                    f(&mut value);
                },
                _ => unreachable!()
            }
        }

        // Take all the current subscribers, the effects will resubscribe when evaluated
        let subscribers = std::mem::take(&mut self.subscriptions[signal.id]);
        subscribers.iter().for_each(|subscriber| {
            self.notify(subscriber);
        });
        //std::mem::swap(&mut subscribers, &mut self.signal_subscriptions[signal.id]);
    }

    fn track(&mut self, source_id: NodeId) {
        match self.scope {
            Scope::Root => { 
                // Not in an effect/memo, nothing to track
            },
            Scope::Memo(node_id) | Scope::Effect(node_id) => {
                self.add_subscription(source_id, node_id);
            },
        }
    }

	fn add_subscription(&mut self, source_id: NodeId, observer_id: NodeId) {
        self.subscriptions[source_id].insert(observer_id);
        self.dependencies[observer_id].insert(source_id);
	}

    fn remove_subscription(&mut self, source_id: NodeId, observer_id: NodeId) {
        self.subscriptions[source_id].remove(&observer_id);
        self.dependencies[observer_id].remove(&source_id);
    }

    fn notify(&mut self, node_id: &NodeId) {
        let node = self.nodes.get_mut(*node_id).expect("Node has been removed");
        match &mut node.node_type {
            NodeType::Effect(effect) => {
                let task = Task::RunEffect { 
                    id: *node_id, 
                    f: Rc::downgrade(&effect.f)
                };
        
                if self.pending_tasks.is_empty() {
                    task.run(self);
                } else {
                    self.pending_tasks.push(task);
                    self.flush_effects();
                }
            },
            NodeType::Memo(memo) => todo!(),
            NodeType::Signal(_) => unreachable!(),
        }
    }

    fn get_signal_value_ref_untracked<'a, T: Any>(&'a self, signal: &Signal<T>) -> &'a T {
        let node = self.nodes.get(signal.id).expect("No Signal found");
        match &node.node_type {
            NodeType::Signal(signal) => signal.value.downcast_ref().expect("Node had wrong value type"),
            _ => unreachable!()
        }
    }

    fn get_signal_value_ref<'a, T: Any>(&'a mut self, signal: &Signal<T>) -> &'a T {
        self.track(signal.id);
        self.get_signal_value_ref_untracked(signal)
    }

    fn get_memo_value_ref_untracked<'a, T: Any>(&'a self, memo: &Memo<T>) -> &'a T {
        todo!()
    }

    fn get_memo_value_ref<'a, T: Any>(&'a mut self, memo: &Memo<T>) -> &'a T {
        self.track(memo.id);
        self.get_memo_value_ref_untracked(memo)
    }

    fn flush_effects(&mut self) {
        while let Some(task) = self.pending_tasks.pop() {
            task.run(self);
        }
    }

    /*pub fn create_stateful_effect<S>(&mut self, f_init: impl FnOnce() -> S, f: impl Fn(S) -> S) {

    }*/
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn map_signal() {
        let mut cx = AppContext::new();
        let signal = cx.create_signal(0);
        let mapped_signal = signal.clone().map(|x| x * 2);    
        signal.set(&mut cx, 2);
        cx.create_effect(move |cx| {
            assert_eq!(mapped_signal.get(cx), 4);
        });
        assert_eq!(mapped_signal.get_untracked(&cx), 4);
    }

    #[test]
    fn effects_execute_upon_creation() {
        let mut cx = AppContext::new();
        let signal = cx.create_signal(0);
        cx.create_effect(move |cx| {
            signal.set(cx, 1);
        });
        
        assert_eq!(signal.get_untracked(&cx), 1);
    }

    #[test]
    fn effects_execute_when_signal_changes() {
        let mut cx = AppContext::new();
        let source_signal = cx.create_signal(0);
        let dest_signal = cx.create_signal(0);
        cx.create_effect(move |cx| {
            let new_value = source_signal.get(cx);
            dest_signal.set(cx, new_value);
        });

        source_signal.set(&mut cx, 1);
        assert_eq!(dest_signal.get_untracked(&cx), 1);

        source_signal.set(&mut cx, 2);
        assert_eq!(dest_signal.get_untracked(&cx), 2);
    }
}