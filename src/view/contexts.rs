use std::any::Any;

use crate::{app::{Accessor, AppState, Signal, SignalContext, SignalGet}, core::{Color, Point, Rectangle, Shape, Transform}, text::TextLayout, window::WindowState, Cursor, Id};
use super::{IdPath, View, ViewFlags, Widget, WidgetData, WidgetNode};
use crate::platform;

pub struct BuildContext<'a, 'b> {
    id_path: IdPath,
    widget_data: &'a mut WidgetData,
    app_state: &'b mut AppState
}

impl<'a, 'b> BuildContext<'a, 'b> {
    pub fn root(widget_data: &'a mut WidgetData, app_state: &'b mut AppState) -> Self {
        Self {
            id_path: IdPath::root(),
            widget_data,
            app_state
        }
    }

    pub fn get_and_track<W: Widget, T: Clone + 'static>(&mut self, accessor: Accessor<T>, f: impl Fn(&T, &mut WidgetContext, &mut W) + 'static) -> T {
        let value = accessor.get_untracked(self.app_state);
        let binding = self.app_state.create_binding(accessor, self.id_path.clone(), move |value, widget_node| {
            let mut ctx = WidgetContext {
                widget_data: &mut widget_node.data,
            };
            // SAFETY: The window class guarantees that the message is dispatched to *this* widget
            let widget = unsafe { &mut *(widget_node.widget.as_mut() as *mut dyn Widget as *mut W) };
            f(value, &mut ctx, widget)
        });
        if let Some(binding) = binding {
            self.widget_data.bindings.push(binding);
        }
        value
    }

    pub fn id_path(&self) -> &IdPath {
        &self.id_path
    }

    pub fn set_focusable(&mut self, focusable: bool) {
        if focusable {
            self.widget_data.set_flag(ViewFlags::FOCUSABLE)
        } else {
            self.widget_data.clear_flag(ViewFlags::FOCUSABLE)
        }
    }

    pub fn build_child<'s, V: View>(&'s mut self, id: Id, view: V) -> WidgetNode {
		let mut id_path = self.id_path.clone();
        id_path.push_child(id);
        let mut data = WidgetData::new(id_path.clone());
        let widget = Box::new(view.build(&mut BuildContext {
            id_path,
            widget_data: &mut data,
            app_state: &mut self.app_state
        }));
        data.style = widget.style();
        WidgetNode {
            widget,
            data
        }
    }
}

pub struct WidgetContext<'a> {
    widget_data: &'a mut WidgetData
}

pub struct LayoutContext<'a, 'b, 'c> {
    widget_data: &'a mut WidgetData,
	handle: &'b platform::HandleRef<'c>
}

impl<'a, 'b, 'c> LayoutContext<'a, 'b, 'c> {
    pub fn new(widget_data: &'a mut WidgetData, handle: &'b platform::HandleRef<'c>) -> Self {
        Self { widget_data, handle }
    }

    pub fn compute_block_layout(&mut self, widget: &mut dyn Widget, inputs: taffy::LayoutInput) -> taffy::LayoutOutput {
        let mut tree = LayoutNodeRef { widget, data: &mut self.widget_data, handle: self.handle };
        taffy::compute_block_layout(&mut tree, LayoutNodeRef::SELF_NODE_ID, inputs)
    }

    pub fn compute_flexbox_layout(&mut self, widget: &mut dyn Widget, inputs: taffy::LayoutInput) -> taffy::LayoutOutput {
        let mut tree = LayoutNodeRef { widget, data: &mut self.widget_data, handle: self.handle };
        taffy::compute_flexbox_layout(&mut tree, LayoutNodeRef::SELF_NODE_ID, inputs)
    }

    pub fn compute_leaf_layout<MeasureFunction>(&mut self, inputs: taffy::LayoutInput, measure_function: MeasureFunction) -> taffy::LayoutOutput 
    where 
        MeasureFunction: FnOnce(taffy::Size<Option<f32>>, taffy::Size<taffy::AvailableSpace>) -> taffy::Size<f32>
    {
        taffy::compute_leaf_layout(inputs, &self.widget_data.style, measure_function)
    }

    pub fn invalidate(&mut self) {
        self.handle.invalidate(self.widget_data.global_bounds())
    }
}

pub(crate) struct LayoutNodeRef<'a, 'b, 'c, 'd> {
    pub(crate) widget: &'a mut dyn Widget,
    pub(crate) data: &'b mut WidgetData,
    pub(crate) handle: &'c platform::HandleRef<'d>
}

impl<'a, 'b, 'c, 'd> LayoutNodeRef<'a, 'b, 'c, 'd> {
    pub const SELF_NODE_ID: taffy::NodeId = taffy::NodeId::new(u64::MAX);
}

pub struct ChildIter(std::ops::Range<usize>);
impl Iterator for ChildIter {
    type Item = taffy::NodeId;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next().map(|idx| taffy::NodeId::from(idx))
    }
}

impl<'a, 'b, 'c, 'd> taffy::TraversePartialTree for LayoutNodeRef<'a, 'b, 'c, 'd> {
    type ChildIter<'e> = ChildIter where Self: 'e;

    fn child_ids(&self, _parent_node_id: taffy::NodeId) -> Self::ChildIter<'_> {
        ChildIter(0..self.widget.child_count())
    }

    fn child_count(&self, _parent_node_id: taffy::NodeId) -> usize {
        self.widget.child_count()
    }

    fn get_child_id(&self, _parent_node_id: taffy::NodeId, child_index: usize) -> taffy::NodeId {
        taffy::NodeId::from(child_index)
    }
}

impl<'a, 'b, 'c, 'd> taffy::LayoutPartialTree for LayoutNodeRef<'a, 'b, 'c, 'd> {
    fn get_style(&self, node_id: taffy::prelude::NodeId) -> &taffy::prelude::Style {
        if node_id == Self::SELF_NODE_ID {
            &self.data.style
        } else {
            &self.widget.get_child(node_id.into()).data.style
        }
    }

    fn set_unrounded_layout(&mut self, node_id: taffy::prelude::NodeId, layout: &taffy::prelude::Layout) {
        if node_id == Self::SELF_NODE_ID {
            self.data.layout = *layout;
        } else {
            self.widget.get_child_mut(node_id.into()).data.layout = *layout
        }
    }

    fn get_cache_mut(&mut self, node_id: taffy::prelude::NodeId) -> &mut taffy::Cache {
        if node_id == Self::SELF_NODE_ID {
            &mut self.data.cache
        } else {
            &mut self.widget.get_child_mut(node_id.into()).data.cache
        }
    }

    fn compute_child_layout(&mut self, node_id: taffy::prelude::NodeId, inputs: taffy::LayoutInput) -> taffy::LayoutOutput {
        taffy::compute_cached_layout(self, node_id, inputs, |parent, node_id, inputs| {
            if node_id == Self::SELF_NODE_ID {
                let mut layout_context = LayoutContext { widget_data: &mut parent.data, handle: parent.handle };
                parent.widget.layout(inputs, &mut layout_context)
            } else {
                let child = parent.widget.get_child_mut(node_id.into());
                let mut layout_context = LayoutContext { widget_data: &mut child.data, handle: parent.handle };
                child.widget.layout(inputs, &mut layout_context)
            }
        })
    }
}

pub struct RenderContext<'a, 'b, 'c, 'd> {
    widget_data: &'a mut WidgetData,
    renderer: &'b mut platform::RendererRef<'c>,
    window_state: &'d mut WindowState
}

impl<'a, 'b, 'c, 'd> RenderContext<'a, 'b, 'c, 'd> {
    pub(crate) fn new(widget_data: &'a mut WidgetData, renderer: &'b mut platform::RendererRef<'c>, window_state: &'d mut WindowState) -> Self {
        Self { widget_data, renderer, window_state }
    }

    pub fn local_bounds(&self) -> Rectangle {
        self.widget_data.local_bounds()
    }

    pub fn global_bounds(&self) -> Rectangle {
        self.widget_data.global_bounds()
    }

    pub fn has_focus(&self) -> bool {
        self.window_state.focus_view.as_ref()
            .is_some_and(|id_path| *id_path == *self.widget_data.id_path())
    }

    pub fn fill(&mut self, shape: impl Into<Shape>, color: impl Into<Color>) {
		let color = color.into();
        match shape.into() {
            Shape::Rect(rect) => 
                self.renderer.fill_rectangle(rect, color),
            Shape::RoundedRect { rect, corner_radius } => 
                self.renderer.fill_rounded_rectangle(rect, corner_radius, color),
            Shape::Ellipse { center, radii } => 
                self.renderer.fill_ellipse(center, radii, color),
            Shape::Line { p0, p1 } =>
                self.renderer.draw_line(p0, p1, color, 1.0)
        }
    }

    pub fn stroke(&mut self, shape: impl Into<Shape>, color: impl Into<Color>, line_width: f32) {
        match shape.into() {
            Shape::Rect(rect) => 
                self.renderer.draw_rectangle(rect, color.into(), line_width),
            Shape::RoundedRect { rect, corner_radius } => 
                self.renderer.draw_rounded_rectangle(
                    rect, 
                    corner_radius, 
                    color.into(), 
                    line_width),
            Shape::Ellipse { center, radii } => self.renderer.draw_ellipse(center, radii, color.into(), line_width),
            Shape::Line { p0, p1 } => self.renderer.draw_line(p0, p1, color.into(), line_width)
        }
    }

    pub fn draw_text(&mut self, text_layout: &TextLayout, position: Point) {
        self.renderer.draw_text(&text_layout.0, position)
    }

    pub fn use_clip(&mut self, rect: impl Into<Rectangle>, f: impl FnOnce(&mut RenderContext<'_, '_, 'c, 'd>)) {
        self.renderer.save();
        self.renderer.clip(rect.into());
        f(self);
        self.renderer.restore();
    }

    pub fn transform(&mut self, transform: impl Into<Transform>) {
        self.renderer.transform(transform.into());
    }

    pub(super) fn with_child<'e>(&mut self, widget_data: &'e mut WidgetData, f: impl FnOnce(&mut RenderContext<'e, '_, 'c, '_>)) {
        f(&mut RenderContext { 
            widget_data,
            renderer: self.renderer,
            window_state: self.window_state
        });
    }
}

pub struct EventContext<'a, 'b, 'c> {
    widget_data: &'a mut WidgetData,
    window_state: &'b mut WindowState,
	handle: &'b platform::HandleRef<'c>
}

impl<'a, 'b, 'c> EventContext<'a, 'b, 'c> {
    pub fn new(widget_data: &'a mut WidgetData, window_state: &'b mut WindowState, handle: &'b platform::HandleRef<'c>) -> Self{
        Self { widget_data, window_state, handle }
    }

    pub fn bounds(&self) -> Rectangle {
        self.widget_data.global_bounds()
    }

    pub(crate) fn with_child<'d>(&mut self, widget_data: &'d mut WidgetData, f: impl FnOnce(&mut EventContext<'d, '_, '_>)) {
        let flags = {
            let mut ctx = EventContext { 
                widget_data,
                window_state: self.window_state,
                handle: self.handle
            };
            f(&mut ctx);
            ctx.view_flags()
        };

        self.widget_data.flags |= flags & (ViewFlags::NEEDS_LAYOUT | ViewFlags::NEEDS_RENDER);
    }

    pub fn capture_mouse(&mut self) {
        self.window_state.mouse_capture_view = Some(self.widget_data.id_path().clone())
    }

    pub fn take_focus(&mut self) {
        self.window_state.focus_view = Some(self.widget_data.id_path().clone())
    }

    pub fn request_layout(&mut self) {
        self.widget_data.set_flag(ViewFlags::NEEDS_LAYOUT);
    }

    pub fn request_render(&mut self) {
        self.handle.invalidate(self.widget_data.global_bounds())
    }

    pub fn request_rebuild(&mut self) {
        self.widget_data.set_flag(ViewFlags::NEEDS_REBUILD);
    }

    pub fn get_clipboard(&mut self) -> Option<String> {
        self.handle.get_clipboard().ok().flatten()
    }

    pub fn set_clipboard(&mut self, string: &str) {
        self.handle.set_clipboard(string).unwrap();
    }

    pub fn view_flags(&self) -> ViewFlags {
        self.widget_data.flags
    }

    pub fn set_cursor(&mut self, cursor: Cursor) {
        self.window_state.cursor = cursor;
    }
}
