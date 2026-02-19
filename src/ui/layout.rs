use std::sync::Arc;

use super::style::{AvailableSpace, LayoutMode, ResolveInto, Style, UiRect};
use super::{OverlayAnchor, OverlayOptions};
use crate::core::{HAlign, Point, Rect, Size, VAlign, Vec2, Zero};
use crate::ui::Widgets;
use crate::ui::widgets::WidgetIdIter;
use taffy::{
    CacheTree, LayoutBlockContainer, LayoutFlexboxContainer, LayoutPartialTree, PrintTree,
    TraversePartialTree, TraverseTree,
};

use super::{AppState, WidgetFlags, WidgetId, WindowId};

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum RecomputeLayout {
    IfNeeded,
    Force,
}

pub fn layout_window(app_state: &mut AppState, window_id: WindowId, mode: RecomputeLayout) {
    app_state.with_id_buffer_mut(move |app_state, overlay_ids| {
        let widgets = &mut app_state.widgets;
        let window = widgets.window(window_id);
        let window_size = window.handle.global_bounds().size();
        let window_rect = Rect::from_origin(Point::ZERO, window_size);
        let root_id = window.root_widget;
        overlay_ids.extend(window.overlays.iter());

        let available_space = taffy::Size {
            width: taffy::AvailableSpace::Definite(window_size.width as f32),
            height: taffy::AvailableSpace::Definite(window_size.height as f32),
        };

        widgets.rebuild_children();
        // Need to layout root first, the overlay positions can depend on their parent positions
        if mode == RecomputeLayout::Force || widgets.data[root_id].needs_layout() {
            println!("Layout main ui");
            let mut cx = LayoutContext {
                widgets,
                window_size,
                region_to_invalidate: None,
            };
            taffy::compute_root_layout(&mut cx, root_id.into(), available_space);
            let region_to_invalidate = cx.region_to_invalidate;

            update_node_origins(widgets, root_id, Point::ZERO);
            if let Some(region_to_invalidate) = region_to_invalidate {
                widgets
                    .window(window_id)
                    .handle
                    .invalidate(region_to_invalidate);
            }
        }

        // This is currently in z-index order, we should probably iterate in insertion order
        // If an overlay's position depends on the position of a previously created overlay,
        // this will be wrong.
        for (i, overlay_id) in overlay_ids.iter().enumerate() {
            if mode == RecomputeLayout::Force || widgets.data[*overlay_id].needs_layout() {
                let mut cx = LayoutContext {
                    widgets,
                    window_size,
                    region_to_invalidate: None,
                };
                taffy::compute_root_layout(&mut cx, (*overlay_id).into(), available_space);
                let region_to_invalidate = cx.region_to_invalidate;
                let options = widgets
                    .window(window_id)
                    .overlays
                    .get_overlay_options(i)
                    .unwrap();

                let offset = compute_overlay_offset(widgets, window_rect, *overlay_id, options);

                update_node_origins(widgets, *overlay_id, offset.into_point());
                if let Some(region_to_invalidate) = region_to_invalidate {
                    widgets
                        .window(window_id)
                        .handle
                        .invalidate(region_to_invalidate.offset(offset));
                }
            }
        }

        //let region_to_invalidate = ctx.region_to_invalidate;

        /*if let Some(region_to_invalidate) = region_to_invalidate {
            app_state
                .window(window_id)
                .handle
                .invalidate(region_to_invalidate);
        }*/
        //taffy::print_tree(&mut ctx, widget_id.into());
    });

    //invalidate_window(app_state, window_id);
}

fn compute_overlay_offset(
    widgets: &Widgets,
    window_rect: Rect,
    overlay_id: WidgetId,
    options: OverlayOptions,
) -> Vec2 {
    let current_bounds = widgets.data[overlay_id].local_bounds();
    let parent_id = widgets.data[overlay_id].parent_id;
    let parent_bounds = widgets.data[parent_id].global_bounds();
    let alignment_offset = match options.anchor {
        OverlayAnchor::Fixed => options.align.compute_offset(current_bounds, window_rect),
        OverlayAnchor::InsideParent => options.align.compute_offset(current_bounds, parent_bounds),
        OverlayAnchor::OutsideParent => {
            let mut result = options.align.compute_offset(current_bounds, parent_bounds);
            match options.align.get_h_align() {
                HAlign::Left => result.x -= current_bounds.width(),
                HAlign::Right => result.x += current_bounds.width(),
                _ => {}
            };
            match options.align.get_v_align() {
                VAlign::Top => result.y -= current_bounds.height(),
                VAlign::Bottom => result.y += current_bounds.height(),
                _ => {}
            }
            result
        }
    };

    alignment_offset + options.offset
}

fn update_node_origins(widgets: &mut Widgets, root_widget: WidgetId, position: Point) {
    let mut stack = vec![];
    widgets.data[root_widget].origin = position;
    for child in widgets.child_id_iter(root_widget) {
        stack.push((child, position));
    }

    while let Some((widget_id, parent_origin)) = stack.pop() {
        let origin = widgets.data[widget_id].offset() + parent_origin.into_vec2();
        for child in widgets.child_id_iter(widget_id) {
            stack.push((child, origin))
        }
        widgets.data[widget_id].origin = origin;
    }
}

pub struct LayoutChildIter<'a> {
    inner: std::slice::Iter<'a, WidgetId>,
}

impl Iterator for LayoutChildIter<'_> {
    type Item = taffy::NodeId;

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next().map(|x| (*x).into())
    }
}

pub struct LayoutContext<'a> {
    widgets: &'a mut Widgets,
    window_size: Size,
    region_to_invalidate: Option<Rect>,
}

impl LayoutContext<'_> {
    fn get_layout_style(&self, node_id: taffy::NodeId) -> LayoutStyle<'_> {
        let node_id = node_id.into();
        LayoutStyle {
            style: &self.widgets.data[node_id].style,
            display_style: self.widgets.widgets[node_id].layout_mode(),
            window_size: self.window_size,
        }
    }
}

impl taffy::TraversePartialTree for LayoutContext<'_> {
    type ChildIter<'b>
        = LayoutChildIter<'b>
    where
        Self: 'b;

    fn child_ids(&self, parent_node_id: taffy::NodeId) -> Self::ChildIter<'_> {
        let inner: &Vec<WidgetId> = self
            .widgets
            .cached_child_ids(parent_node_id.into())
            .as_ref();
        LayoutChildIter {
            inner: inner.iter(),
        }
    }

    fn child_count(&self, parent_node_id: taffy::NodeId) -> usize {
        self.widgets.cached_child_ids(parent_node_id.into()).len()
    }

    fn get_child_id(&self, parent_node_id: taffy::NodeId, child_index: usize) -> taffy::NodeId {
        self.widgets.cached_child_ids(parent_node_id.into())[child_index].into()
    }
}

impl TraverseTree for LayoutContext<'_> {}

impl PrintTree for LayoutContext<'_> {
    fn get_debug_label(&self, node_id: taffy::NodeId) -> &'static str {
        self.widgets.widgets[node_id.into()].debug_label()
    }

    fn get_final_layout(&self, node_id: taffy::NodeId) -> taffy::Layout {
        self.widgets.data[node_id.into()].layout
    }
}

impl LayoutBlockContainer for LayoutContext<'_> {
    type BlockContainerStyle<'b>
        = LayoutStyle<'b>
    where
        Self: 'b;
    type BlockItemStyle<'b>
        = LayoutStyle<'b>
    where
        Self: 'b;

    fn get_block_container_style(&self, node_id: taffy::NodeId) -> Self::BlockContainerStyle<'_> {
        self.get_layout_style(node_id)
    }

    fn get_block_child_style(&self, child_node_id: taffy::NodeId) -> Self::BlockItemStyle<'_> {
        self.get_layout_style(child_node_id)
    }
}

impl LayoutFlexboxContainer for LayoutContext<'_> {
    type FlexboxContainerStyle<'b>
        = LayoutStyle<'b>
    where
        Self: 'b;
    type FlexboxItemStyle<'b>
        = LayoutStyle<'b>
    where
        Self: 'b;

    fn get_flexbox_container_style(
        &self,
        node_id: taffy::NodeId,
    ) -> Self::FlexboxContainerStyle<'_> {
        self.get_layout_style(node_id)
    }

    fn get_flexbox_child_style(&self, child_node_id: taffy::NodeId) -> Self::FlexboxItemStyle<'_> {
        self.get_layout_style(child_node_id)
    }
}

impl CacheTree for LayoutContext<'_> {
    fn cache_get(
        &self,
        node_id: taffy::NodeId,
        known_dimensions: taffy::Size<Option<f32>>,
        available_space: taffy::Size<taffy::AvailableSpace>,
        run_mode: taffy::RunMode,
    ) -> Option<taffy::LayoutOutput> {
        self.widgets.data[node_id.into()]
            .cache
            .get(known_dimensions, available_space, run_mode)
    }

    fn cache_store(
        &mut self,
        node_id: taffy::NodeId,
        known_dimensions: taffy::Size<Option<f32>>,
        available_space: taffy::Size<taffy::AvailableSpace>,
        run_mode: taffy::RunMode,
        layout_output: taffy::LayoutOutput,
    ) {
        self.widgets.data[node_id.into()].cache.store(
            known_dimensions,
            available_space,
            run_mode,
            layout_output,
        )
    }

    fn cache_clear(&mut self, node_id: taffy::NodeId) {
        self.widgets.data[node_id.into()].cache.clear();
    }
}

/*impl<'a> LayoutGridContainer for LayoutContext<'a> {
    type GridContainerStyle<'b> = LayoutStyle<'b> where Self: 'b;
    type GridItemStyle<'b> = LayoutStyle<'b> where Self: 'b;

    fn get_grid_container_style(&self, node_id: taffy::NodeId) -> Self::GridContainerStyle<'_> {
        self.get_layout_style(node_id)
    }

    fn get_grid_child_style(&self, child_node_id: taffy::NodeId) -> Self::GridItemStyle<'_> {
        self.get_layout_style(child_node_id)
    }
}*/

impl LayoutPartialTree for LayoutContext<'_> {
    type CoreContainerStyle<'b>
        = LayoutStyle<'b>
    where
        Self: 'b;
    type CustomIdent = Arc<str>;

    fn get_core_container_style(&self, node_id: taffy::NodeId) -> Self::CoreContainerStyle<'_> {
        self.get_layout_style(node_id)
    }

    fn set_unrounded_layout(&mut self, node_id: taffy::NodeId, layout: &taffy::Layout) {
        let old_bounds = self.widgets.data[node_id.into()].global_bounds();
        let widget_data = &mut self.widgets.data[node_id.into()];
        widget_data.layout = *layout;
        let new_bounds = widget_data.global_bounds();
        if new_bounds != old_bounds {
            let rect = new_bounds.union(&old_bounds);
            self.region_to_invalidate = self
                .region_to_invalidate
                .map(|old| old.union(&rect))
                .or(Some(rect));

            // Need to request layout for all overlays, their position
            // might depend on the bounding box of its parent
            let mut id_iter = WidgetIdIter::all_overlays(self.widgets, node_id.into());
            while let Some(overlay_id) = id_iter.next_id(self.widgets) {
                self.widgets.request_layout(overlay_id);
            }
        }
    }

    fn compute_child_layout(
        &mut self,
        node_id: taffy::NodeId,
        inputs: taffy::LayoutInput,
    ) -> taffy::LayoutOutput {
        // If RunMode is PerformHiddenLayout then this indicates that an ancestor node is `Display::None`
        // and thus that we should lay out this node using hidden layout regardless of it's own display style.
        if inputs.run_mode == taffy::RunMode::PerformHiddenLayout {
            return taffy::compute_hidden_layout(self, node_id);
        }

        {
            let widget_data = &mut self.widgets.data[node_id.into()];
            if widget_data.flag_is_set(WidgetFlags::NEEDS_LAYOUT) {
                widget_data.cache.clear();
                widget_data.clear_flag(WidgetFlags::NEEDS_LAYOUT);
            }
        }

        taffy::compute_cached_layout(self, node_id, inputs, |tree, node, inputs| {
            if tree.widgets.data[node_id.into()].is_hidden() {
                taffy::compute_hidden_layout(tree, node)
            } else {
                let has_children = tree.child_count(node) > 0;
                let display_style = tree.widgets.widgets[node.into()].layout_mode();
                match (display_style, has_children) {
                    (LayoutMode::Block, true) => taffy::compute_block_layout(tree, node, inputs),
                    (LayoutMode::Flex(_), true) => {
                        taffy::compute_flexbox_layout(tree, node, inputs)
                    }
                    (LayoutMode::Grid(_), _) => unreachable!(),
                    (LayoutMode::Stack, _) => compute_stack_layout(tree, node_id, inputs),
                    (LayoutMode::Leaf(measure), _) => {
                        let style = &tree.widgets.data[node.into()].style;
                        let measure_function =
                            |known_dimensions: taffy::Size<Option<f32>>, available_space| {
                                let available_size = known_dimensions.zip_map(
                                    available_space,
                                    |known, available| {
                                        known
                                            .map(|x| AvailableSpace::Exact(x as f64))
                                            .unwrap_or_else(|| match available {
                                                taffy::AvailableSpace::Definite(x) => {
                                                    AvailableSpace::Exact(x as f64)
                                                }
                                                taffy::AvailableSpace::MinContent => {
                                                    AvailableSpace::MinContent
                                                }
                                                taffy::AvailableSpace::MaxContent => {
                                                    AvailableSpace::MaxContent
                                                }
                                            })
                                    },
                                );

                                let size = measure.measure(
                                    style,
                                    available_size.width,
                                    available_size.height,
                                );
                                taffy::Size {
                                    width: size.width as _,
                                    height: size.height as _,
                                }
                            };

                        let run_mode = inputs.run_mode;
                        let output = taffy::compute_leaf_layout(
                            inputs,
                            &tree.get_layout_style(node_id),
                            |_val, _basis| 0.0,
                            measure_function,
                        );

                        if run_mode == taffy::RunMode::PerformLayout {}

                        output
                    }
                    (_, false) => {
                        let measure_function = |_, _| taffy::Size::ZERO;
                        taffy::compute_leaf_layout(
                            inputs,
                            &tree.get_layout_style(node_id),
                            |_val, _basis| 0.0,
                            measure_function,
                        )
                    }
                }
            }
        })
    }
}

fn compute_stack_layout(
    _tree: &mut LayoutContext,
    _node_id: taffy::NodeId,
    _inputs: taffy::LayoutInput,
) -> taffy::LayoutOutput {
    todo!()
}

/// Style used during layout
pub struct LayoutStyle<'a> {
    pub(crate) style: &'a Style,
    pub(crate) display_style: LayoutMode<'a>,
    pub(crate) window_size: Size,
}

const TAFFY_DEFAULT_STYLE: taffy::Style<Arc<str>> = taffy::Style::<Arc<str>>::DEFAULT;

impl taffy::CoreStyle for LayoutStyle<'_> {
    type CustomIdent = Arc<str>;

    fn box_generation_mode(&self) -> taffy::BoxGenerationMode {
        if self.style.hidden {
            taffy::BoxGenerationMode::None
        } else {
            taffy::BoxGenerationMode::Normal
        }
    }

    fn is_block(&self) -> bool {
        matches!(self.display_style, LayoutMode::Block)
    }

    fn box_sizing(&self) -> taffy::BoxSizing {
        taffy::BoxSizing::BorderBox
    }

    fn overflow(&self) -> taffy::Point<taffy::Overflow> {
        taffy::Point {
            x: self.style.overflow_x,
            y: self.style.overflow_y,
        }
    }

    fn scrollbar_width(&self) -> f32 {
        self.style.scrollbar_width as _
    }

    fn position(&self) -> taffy::Position {
        taffy::Position::Relative
    }

    fn inset(&self) -> taffy::Rect<taffy::LengthPercentageAuto> {
        self.style.inset.resolve_into(self.window_size)
    }

    fn size(&self) -> taffy::Size<taffy::Dimension> {
        self.style.size.resolve_into(self.window_size)
    }

    fn min_size(&self) -> taffy::Size<taffy::Dimension> {
        self.style.min_size.resolve_into(self.window_size)
    }

    fn max_size(&self) -> taffy::Size<taffy::Dimension> {
        self.style.max_size.resolve_into(self.window_size)
    }

    fn aspect_ratio(&self) -> Option<f32> {
        self.style.aspect_ratio.map(|x| x as _)
    }

    fn margin(&self) -> taffy::Rect<taffy::LengthPercentageAuto> {
        self.style.margin.resolve_into(self.window_size)
    }

    fn padding(&self) -> taffy::Rect<taffy::LengthPercentage> {
        self.style.padding.resolve_into(self.window_size)
    }

    fn border(&self) -> taffy::Rect<taffy::LengthPercentage> {
        UiRect::all(self.style.border).resolve_into(self.window_size)
    }
}

impl taffy::FlexboxContainerStyle for LayoutStyle<'_> {
    fn flex_direction(&self) -> taffy::FlexDirection {
        match &self.display_style {
            LayoutMode::Flex(flex) => flex.direction,
            _ => TAFFY_DEFAULT_STYLE.flex_direction,
        }
    }

    fn flex_wrap(&self) -> taffy::FlexWrap {
        match &self.display_style {
            LayoutMode::Flex(flex) => flex.wrap,
            _ => TAFFY_DEFAULT_STYLE.flex_wrap,
        }
    }

    fn gap(&self) -> taffy::Size<taffy::LengthPercentage> {
        match &self.display_style {
            LayoutMode::Flex(flex) => taffy::Size {
                width: flex.gap.resolve_into(self.window_size),
                height: flex.gap.resolve_into(self.window_size),
            },
            _ => TAFFY_DEFAULT_STYLE.gap,
        }
    }

    fn align_content(&self) -> Option<taffy::AlignContent> {
        match &self.display_style {
            LayoutMode::Flex(flex) => flex.align_content,
            _ => TAFFY_DEFAULT_STYLE.align_content,
        }
    }

    fn align_items(&self) -> Option<taffy::AlignItems> {
        match &self.display_style {
            LayoutMode::Flex(flex) => flex.align_items,
            _ => TAFFY_DEFAULT_STYLE.align_items,
        }
    }

    fn justify_content(&self) -> Option<taffy::JustifyContent> {
        TAFFY_DEFAULT_STYLE.justify_content
    }
}

impl taffy::FlexboxItemStyle for LayoutStyle<'_> {
    fn flex_basis(&self) -> taffy::Dimension {
        TAFFY_DEFAULT_STYLE.flex_basis
    }

    fn flex_grow(&self) -> f32 {
        self.style.flex_grow
    }

    fn flex_shrink(&self) -> f32 {
        self.style.flex_shrink
    }

    fn align_self(&self) -> Option<taffy::AlignSelf> {
        self.style.align_self
    }
}

impl taffy::BlockContainerStyle for LayoutStyle<'_> {
    fn text_align(&self) -> taffy::TextAlign {
        TAFFY_DEFAULT_STYLE.text_align
    }
}

impl taffy::BlockItemStyle for LayoutStyle<'_> {
    fn is_table(&self) -> bool {
        false
    }
}

/*impl taffy::GridContainerStyle for LayoutStyle<'_> {
    type TemplateTrackList<'b>
        = &'b [taffy::TrackSizingFunction]
    where
        Self: 'b;
    type AutoTrackList<'b>
        = &'b [taffy::NonRepeatedTrackSizingFunction]
    where
        Self: 'b;

    fn grid_template_rows(&self) -> &[taffy::TrackSizingFunction] {
        match self.display_style {
            LayoutMode::Grid(grid_style) => &grid_style.row_templates,
            _ => &[],
        }
    }

    fn grid_template_columns(&self) -> Self::TemplateTrackList<'_> {
        match self.display_style {
            LayoutMode::Grid(grid_style) => &grid_style.column_templates,
            _ => &[],
        }
    }

    fn grid_auto_rows(&self) -> Self::AutoTrackList<'_> {
        todo!()
    }

    fn grid_auto_columns(&self) -> Self::AutoTrackList<'_> {
        todo!()
    }

    fn grid_auto_flow(&self) -> taffy::GridAutoFlow {
        TAFFY_DEFAULT_STYLE.grid_auto_flow
    }

    fn gap(&self) -> taffy::Size<taffy::LengthPercentage> {
        TAFFY_DEFAULT_STYLE.gap
    }

    fn align_content(&self) -> Option<taffy::AlignContent> {
        TAFFY_DEFAULT_STYLE.align_content
    }

    fn justify_content(&self) -> Option<taffy::JustifyContent> {
        TAFFY_DEFAULT_STYLE.justify_content
    }

    fn align_items(&self) -> Option<taffy::AlignItems> {
        TAFFY_DEFAULT_STYLE.align_items
    }

    fn justify_items(&self) -> Option<taffy::AlignItems> {
        TAFFY_DEFAULT_STYLE.justify_items
    }
}
*/
