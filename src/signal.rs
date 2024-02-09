use std::{marker::PhantomData, rc::{Weak, Rc}, cell::RefCell, sync::atomic::AtomicUsize, any::Any, collections::HashSet};
use slotmap::{new_key_type, SlotMap, SecondaryMap, Key};

pub trait SignalGet {
    type Value;

    /// Map the current value using `f` and subscribe to changes
    fn map_ref<R>(&self, ctx: &mut AppContext, f: impl Fn(&Self::Value) -> R) -> R;

    /// Get the current value and subscribe to changes
    fn get(&self, ctx: &mut AppContext) -> Self::Value 
        where Self::Value: Clone 
    {
        self.map_ref(ctx, Self::Value::clone)
    }

    fn map_ref_untracked<R>(&self, ctx: &mut AppContext, f: impl Fn(&Self::Value) -> R) -> R;

    fn get_untracked(&self, ctx: &mut AppContext) -> Self::Value 
        where Self::Value: Clone 
    {
        self.map_ref_untracked(ctx, Self::Value::clone)
    }
}

pub trait SignalSet {
    type Value;

    /// Sets the current value without notifying subscribers
    fn set_untracked(&self, ctx: &mut AppContext, value: Self::Value) {
        self.set_with_untracked(ctx, move || value)
    }

    /// Sets the current value without notifying subscribers
    fn set_with_untracked(&self, ctx: &mut AppContext, f: impl FnOnce() -> Self::Value);

    /// Set the current value, notifies subscribers
    fn set(&self, ctx: &mut AppContext, value: Self::Value) {
        self.set_with(ctx, move || value)
    }

    /// Set the current value, notifies subscribers
    fn set_with(&self, ctx: &mut AppContext, f: impl FnOnce() -> Self::Value);
}

new_key_type! { 
    pub struct SignalId; 
    pub struct MemoId;
    pub struct EffectId;
}

pub struct Signal<T> {
    id: SignalId,
    ref_counts: Weak<RefCell<RefCounts<SignalId>>>,
    _marker: PhantomData<T>
}

impl<T: Any> SignalSet for Signal<T> {
    type Value = T;

    fn set_with(&self, ctx: &mut AppContext, f: impl FnOnce() -> Self::Value) {
        ctx.set_signal_value(self, f())
    }

    fn set_with_untracked(&self, ctx: &mut AppContext, f: impl FnOnce() -> Self::Value) {
        ctx.set_signal_value_untracked(self, f())
    }
}

impl<T: 'static> SignalGet for Signal<T> {
    type Value = T;

    fn map_ref<R>(&self, ctx: &mut AppContext, f: impl Fn(&T) -> R) -> R {
        f(ctx.get_signal_value_ref(self))
    }

    fn map_ref_untracked<R>(&self, ctx: &mut AppContext, f: impl Fn(&Self::Value) -> R) -> R {
        todo!()
    }
}

impl<T> Clone for Signal<T> {
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
}

pub struct Memo<T> {
    id: MemoId,
    ref_counts: Weak<RefCell<RefCounts<MemoId>>>,
    _marker: PhantomData<T>
}

impl<T> SignalGet for Memo<T> {
    type Value = T;

    fn map_ref<R>(&self, ctx: &mut AppContext, f: impl Fn(&Self::Value) -> R) -> R {
        todo!()
    }

    fn map_ref_untracked<R>(&self, ctx: &mut AppContext, f: impl Fn(&Self::Value) -> R) -> R {
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
    f: Box<dyn Fn(&mut AppContext)>,
    dependencies: HashSet<SourceId>
}

impl EffectState {
    fn new(f: impl Fn(&mut AppContext) + 'static) -> Self {
        Self {
            f: Box::new(f),
            dependencies: Default::default()
        }
    }

    fn run(&self, cx: &mut AppContext) {
        (self.f)(cx);
    }
}

enum SubscriberId {
    Memo(MemoId),
    Effect(EffectId)
}

enum SourceId {
    Memo(MemoId),
    Effect(EffectId)
}

enum Scope {
    Signal(SignalId),
    Memo(MemoId),
    Effect(EffectId)
}

pub struct AppContext {
    current_scope: Vec<EffectId>,
    signals: SecondaryMap<SignalId, SignalState>,
    signal_ref_counts: Rc<RefCell<RefCounts<SignalId>>>,
    signal_subscriptions: SecondaryMap<SignalId, HashSet<SubscriberId>>,
    memos: SecondaryMap<MemoId, MemoState>,
    memo_ref_counts: Rc<RefCell<RefCounts<MemoId>>>,
    effects: SlotMap<EffectId, Option<EffectState>>
}

impl AppContext {
    pub fn new() -> Self {
        Self {
            current_scope: Default::default(),
            signal_ref_counts: Default::default(),
            signals: Default::default(),
            signal_subscriptions: Default::default(),
            memos: Default::default(),
            memo_ref_counts: Default::default(),
            effects: Default::default()
        }
    }

    pub fn create_signal<T: Any>(&mut self, value: T) -> Signal<T> {
        let id = RefCounts::alloc_id(&self.signal_ref_counts);
        self.signals.insert(id, SignalState::new(value));
        self.signal_subscriptions.insert(id, HashSet::new());
        Signal { 
            ref_counts: Rc::downgrade(&self.signal_ref_counts), 
            id,
            _marker: PhantomData
        }
    }

    fn remove_signal(&mut self, id: SignalId) {
        self.signals.remove(id);
        self.signal_subscriptions.remove(id);
    }

    fn set_signal_value_untracked<T: Any>(&mut self, signal: &Signal<T>, value: T) {
        let signal = self.signals.get_mut(signal.id).expect("No signal found");
        signal.value = Box::new(value);
    }

    fn set_signal_value<T: Any>(&mut self, signal: &Signal<T>, value: T) {
        self.set_signal_value_untracked(signal, value);
        
        // Take all the current subscribers, the effects will resubscribe when evaluated
        let subscribers = std::mem::take(&mut self.signal_subscriptions[signal.id]);
        subscribers.iter().for_each(|subscriber| {
            self.notify(subscriber);
        });
        //std::mem::swap(&mut subscribers, &mut self.signal_subscriptions[signal.id]);
    }

    fn notify(&mut self, subscriber: &SubscriberId) {
        match subscriber {
            SubscriberId::Memo(_) => {
                todo!()
            },
            SubscriberId::Effect(_) => {
                todo!()
            }
        }
    }

    fn get_signal_value_ref_untracked<T: Any>(&self, signal: &Signal<T>) -> &T {
        let signal = self.signals.get(signal.id).expect("No Signal found");
        let signal = signal.value.downcast_ref();
        signal.as_ref().expect("Signal has wrong type")
    }

    fn get_signal_value_ref<T: Any>(&mut self, signal: &Signal<T>) -> &T {
        let value = self.get_signal_value_ref_untracked(signal);

        value
    }

    pub fn create_memo<T: PartialEq>(&mut self, f: impl Fn(&mut Self) -> T) -> Memo<T> {
        let id = RefCounts::alloc_id(&self.memo_ref_counts);
        //self.current_scope.push(id);
        Memo { 
            id, 
            ref_counts: Rc::downgrade(&self.memo_ref_counts), 
            _marker: PhantomData 
        }
    }

    pub fn create_effect(&mut self, f: impl Fn(&mut Self) + 'static) {
        let id = self.effects.insert(None);
        let effect = EffectState::new(f);
        self.current_scope.push(id);
        (effect.f)(self);
        self.current_scope.pop();
    }

    pub fn create_stateful_effect<S>(&mut self, f_init: impl FnOnce() -> S, f: impl Fn(S) -> S) {

    }
}