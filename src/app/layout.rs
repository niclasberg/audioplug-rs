use taffy::{LayoutPartialTree, TraversePartialTree, TraverseTree};

use super::{widget_node::WidgetFlags, AppState, WidgetId, WindowId};

pub fn layout_window(app_state: &mut AppState, window_id: WindowId) {
    let (bounds, widget_id) = {
        let window = app_state.window(window_id);
        (window.handle.global_bounds().size(), window.root_widget)
    };

    {
        let available_space = taffy::Size {
            width: taffy::AvailableSpace::Definite(bounds.width as f32),
            height: taffy::AvailableSpace::Definite(bounds.height as f32),
        };
        let mut ctx = LayoutContext { app_state };
        taffy::compute_root_layout(&mut ctx, widget_id.into(), available_space);;
    }
    
    {
        // Update origin for all nodes
    }
}

pub struct LayoutChildIter<'a> {
    inner: std::slice::Iter<'a, WidgetId>
}

impl<'a> Iterator for LayoutChildIter<'a> {
    type Item = taffy::NodeId;

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next().map(|x| (*x).into())
    }
}

pub struct LayoutContext<'a> {
	app_state: &'a mut AppState
}

impl<'a> taffy::TraversePartialTree for LayoutContext<'a> {
    type ChildIter<'b> = LayoutChildIter<'b> where Self: 'b;

    fn child_ids(&self, parent_node_id: taffy::NodeId) -> Self::ChildIter<'_> {
        let inner = self.app_state.widget_data[parent_node_id.into()].children.iter();
        LayoutChildIter { inner }
    }

    fn child_count(&self, parent_node_id: taffy::NodeId) -> usize {
        self.app_state.widget_data[parent_node_id.into()].children.len()
    }

    fn get_child_id(&self, parent_node_id: taffy::NodeId, child_index: usize) -> taffy::NodeId {
        self.app_state.widget_data[parent_node_id.into()].children[child_index].into()
    }
}

impl<'a> TraverseTree for LayoutContext<'a> {}

impl<'a> LayoutPartialTree for LayoutContext<'a> {
    fn get_style(&self, node_id: taffy::NodeId) -> &taffy::Style {
        &self.app_state.widget_data[node_id.into()].style
    }

    fn set_unrounded_layout(&mut self, node_id: taffy::NodeId, layout: &taffy::Layout) {
        self.app_state.widget_data[node_id.into()].layout = *layout;
    }

    fn get_cache_mut(&mut self, node_id: taffy::NodeId) -> &mut taffy::Cache {
        &mut self.app_state.widget_data[node_id.into()].cache
    }

    fn compute_child_layout(&mut self, node_id: taffy::NodeId, inputs: taffy::LayoutInput) -> taffy::LayoutOutput {
        // If RunMode is PerformHiddenLayout then this indicates that an ancestor node is `Display::None`
        // and thus that we should lay out this node using hidden layout regardless of it's own display style.
        if inputs.run_mode == taffy::RunMode::PerformHiddenLayout {
            return taffy::compute_hidden_layout(self, node_id);
        }

        {
            let widget_data = &self.app_state.widget_data[node_id.into()];
            if widget_data.flag_is_set(WidgetFlags::NEEDS_LAYOUT) {
                widget_data.cache.clear();
                widget_data.clear_flag(WidgetFlags::NEEDS_LAYOUT);
            }
        }
        
        taffy::compute_cached_layout(self, node_id, inputs, |tree, node, inputs| {
            let display_mode = tree.get_style(node).display;
            let has_children = tree.child_count(node) > 0;

            // Dispatch to a layout algorithm based on the node's display style and whether the node has children or not.
            match (display_mode, has_children) {
                (taffy::Display::None, _) => taffy::compute_hidden_layout(tree, node),
                (taffy::Display::Block, true) => taffy::compute_block_layout(tree, node, inputs),
                (taffy::Display::Flex, true) => taffy::compute_flexbox_layout(tree, node, inputs),
                (taffy::Display::Grid, true) => taffy::compute_grid_layout(tree, node, inputs),
                (_, false) => {
                    let widget_id = node.into();
                    let style = &tree.app_state.widget_data[widget_id].style;
                    let widget = &tree.app_state.widgets[widget_id];
                    let measure_function = |known_dimensions, available_space| {
                        widget.measure(style, known_dimensions, available_space)
                    };
                    taffy::compute_leaf_layout(inputs, style, measure_function)
                }
            }
        })
    }
}

