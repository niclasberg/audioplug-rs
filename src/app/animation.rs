use std::{
    any::Any,
    marker::PhantomData,
    time::{Duration, Instant},
};

use crate::{core::Interpolate, AnimationFrame};

use super::{
    accessor::SourceId, layout::request_layout, render::invalidate_widget, AppState, CreateContext,
    NodeId, NodeType, ReactiveContext, ReadContext, Readable, ViewContext, WidgetId, WindowId,
};

/// Should be called when the animation timer for a window ticks.
/// Steps all animations that have been enqueued for window.
pub(super) fn drive_animations(
    app_state: &mut AppState,
    window_id: WindowId,
    animation_frame: AnimationFrame,
) {
    let widget_ids = std::mem::take(&mut app_state.window_mut(window_id).pending_widget_animations);
    for widget_id in widget_ids {
        let mut ctx = AnimationContext {
            id: widget_id,
            app_state,
        };
        ctx.run_animation(animation_frame)
    }

    let node_ids = std::mem::take(&mut app_state.window_mut(window_id).pending_node_animations);
    let now = Instant::now();
    let mut should_run_effects = false;
    for node_id in node_ids {
        let did_change = match &mut app_state.runtime_mut().get_node_mut(node_id).node_type {
            NodeType::Animation(animation) => animation.inner.drive(now),
            _ => unreachable!(),
        };

        if did_change {
            should_run_effects = true;
            app_state.runtime_mut().notify(node_id);
            app_state
                .window_mut(window_id)
                .pending_node_animations
                .insert(node_id);
        }
    }

    if should_run_effects {
        app_state.run_effects();
    }
}

pub fn request_animation_frame(app_state: &mut AppState, widget_id: WidgetId) {
    let window_id = app_state.get_window_id_for_widget(widget_id);
    app_state
        .window_mut(window_id)
        .pending_widget_animations
        .insert(widget_id);
}

pub(super) fn request_node_animation(
    app_state: &mut AppState,
    window_id: WindowId,
    node_id: NodeId,
) {
    let was_found = app_state
        .runtime_mut()
        .try_with_node(node_id, |cx, node| match node {
            NodeType::Animation(animation) => animation.inner.reset(cx),
            _ => unreachable!(),
        })
        .is_some();

    if was_found {
        app_state
            .window_mut(window_id)
            .pending_node_animations
            .insert(node_id);
    }
}

pub struct AnimationContext<'a> {
    id: WidgetId,
    app_state: &'a mut AppState,
}

impl AnimationContext<'_> {
    fn run_animation(&mut self, animation_frame: AnimationFrame) {
        if let Some(mut widget) = self.app_state.widgets.remove(self.id) {
            widget.animation_frame(animation_frame, self);
            self.app_state.widgets.insert(self.id, widget);
        }
    }

    pub fn has_focus(&self) -> bool {
        self.app_state.widget_has_focus(self.id)
    }

    pub fn request_render(&mut self) {
        invalidate_widget(self.app_state, self.id);
    }

    pub fn request_animation(&mut self) {
        request_animation_frame(self.app_state, self.id)
    }

    pub fn request_layout(&mut self) {
        request_layout(self.app_state, self.id);
    }
}

pub trait AnyAnimation {
    fn value(&self) -> &dyn Any;
    /// Drives the animation
    /// Should return false when the animation is finished, and true otherwise
    fn drive(&mut self, time: Instant) -> bool;
    fn reset(&mut self, cx: &mut dyn ReadContext);
}

type EasingFn = fn(f64) -> f64;

#[derive(Debug, Default, PartialEq, Eq)]
pub enum Easing {
    #[default]
    Linear,
    SineIn,
    SineOut,
    SineInOut,
    QuadIn,
    QuadOut,
    QuadInOut,
    CubicIn,
    CubicOut,
    CubicInOut,
    Custom(EasingFn),
}

impl Easing {
    fn into_fn(self) -> EasingFn {
        use std::f64::consts::PI;
        match self {
            Self::Linear => |x| x,
            Self::SineIn => |x| 1.0 - ((x * PI) / 2.0).cos(),
            Self::SineOut => |x| ((x * PI) / 2.0).sin(),
            Self::SineInOut => |x| -((PI * x).cos() - 1.0) / 2.0,
            Self::QuadIn => |x| x.powi(2),
            Self::QuadOut => |x| 1.0 - (1.0 - x).powi(2),
            Self::QuadInOut => |x| {
                if x < 0.5 {
                    2.0 * x * x
                } else {
                    1.0 - (-2.0 * x + 2.0).powi(2) / 2.0
                }
            },
            Self::CubicIn => |x| x.powi(3),
            Self::CubicOut => |x| 1.0 - (1.0 - x).powi(3),
            Self::CubicInOut => |x| {
                if x < 0.5 {
                    4.0 * x.powi(3)
                } else {
                    1.0 - (-2.0 * x + 2.0).powi(3) / 2.0
                }
            },
            Self::Custom(f) => f,
        }
    }
}

pub struct TweenOptions {
    pub duration: Duration,
    pub easing: Easing,
}

impl TweenOptions {
    pub const fn new() -> Self {
        Self {
            duration: Duration::from_secs(1),
            easing: Easing::Linear,
        }
    }
}

impl Default for TweenOptions {
    fn default() -> Self {
        Self::new()
    }
}

enum TweenState<T> {
    Reset { target: T },
    Running { start_time: Instant, from: T, to: T },
    Done,
}

pub struct TweenAnimation<T: Readable> {
    source: T,
    state: TweenState<T::Value>,
    value: T::Value,
    duration: f64,
    tween_fn: fn(progress: f64) -> f64,
}

impl<T> AnyAnimation for TweenAnimation<T>
where
    T: Readable,
    T::Value: Any + Clone + Interpolate + std::fmt::Debug,
{
    fn value(&self) -> &dyn Any {
        &self.value
    }

    fn drive(&mut self, time: Instant) -> bool {
        match &mut self.state {
            TweenState::Reset { target } => {
                self.state = TweenState::Running {
                    start_time: time,
                    from: self.value.clone(),
                    to: target.clone(),
                };
                println!("Reset: {:?}", self.value);
                true
            }
            TweenState::Running {
                start_time,
                from,
                to,
            } => {
                let seconds_passed = time.duration_since(*start_time).as_secs_f64();
                if seconds_passed >= self.duration {
                    self.value = to.clone();
                    self.state = TweenState::Done;
                } else {
                    self.value = from.lerp(&to, (self.tween_fn)(seconds_passed / self.duration));
                }
                println!("Running: {:?}", self.value);
                true
            }
            TweenState::Done => false,
        }
    }

    fn reset(&mut self, cx: &mut dyn ReadContext) {
        self.state = TweenState::Reset {
            target: self.source.get_untracked(cx),
        };
    }
}

pub struct SpringOptions {
    mass: f64,
    stiffness: f64,
    damping: f64,
}

pub struct SpringState<T: Readable> {
    source: T,
    current: T::Value,
    target: T::Value,
}

pub struct AnimationState {
    inner: Box<dyn AnyAnimation>,
    pub(super) window_id: WindowId,
}

pub struct Animated<T> {
    pub(crate) id: NodeId,
    _phantom: PhantomData<*const T>,
}

impl<T> Clone for Animated<T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T> Copy for Animated<T> {}

impl<T> Animated<T> {
    pub fn tween<S, Ctx>(cx: &mut Ctx, signal: S, options: TweenOptions) -> Self
    where
        Ctx: CreateContext + ViewContext,
        S: Readable<Value = T> + 'static,
        T: Clone + Interpolate + 'static + std::fmt::Debug,
    {
        let window_id = cx.window_id();
        let value = signal.get_untracked(cx);
        let source_id = signal.get_source_id();
        let owner = cx.owner();
        let inner = Box::new(TweenAnimation {
            source: signal,
            duration: options.duration.as_secs_f64(),
            tween_fn: options.easing.into_fn(),
            state: TweenState::Done,
            value,
        });
        let state = AnimationState { inner, window_id };
        Self {
            id: cx
                .runtime_mut()
                .create_animation_node(source_id, state, owner),
            _phantom: PhantomData,
        }
    }
}

impl<T: 'static> Readable for Animated<T> {
    type Value = T;

    fn get_source_id(&self) -> SourceId {
        SourceId::Node(self.id)
    }

    fn with_ref<R>(&self, cx: &mut dyn ReadContext, f: impl FnOnce(&T) -> R) -> R {
        let scope = cx.scope();
        cx.runtime_mut().track(self.id, scope);
        f(get_animation_value_ref(cx, self.id)
            .downcast_ref()
            .expect("Animation value had wrong type"))
    }

    fn with_ref_untracked<R>(
        &self,
        cx: &mut dyn ReactiveContext,
        f: impl FnOnce(&Self::Value) -> R,
    ) -> R {
        f(get_animation_value_ref(cx, self.id)
            .downcast_ref()
            .expect("Animation value had wrong type"))
    }
}

fn get_animation_value_ref(cx: &mut dyn ReactiveContext, node_id: NodeId) -> &dyn Any {
    match &cx.runtime().get_node(node_id).node_type {
        NodeType::Animation(animation) => animation.inner.value(),
        _ => unreachable!(),
    }
}
