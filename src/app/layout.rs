use taffy::{LayoutBlockContainer, LayoutFlexboxContainer, LayoutGridContainer, LayoutPartialTree, PrintTree, TraversePartialTree, TraverseTree};
use crate::{core::Point, style::{DisplayStyle, LayoutStyle}};

use super::{invalidate_window, widget_node::WidgetFlags, AppState, WidgetData, WidgetId, WindowId};

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
        taffy::compute_root_layout(&mut ctx, widget_id.into(), available_space);
    }

    update_node_origins(app_state, widget_id);
    invalidate_window(app_state, window_id);
}

pub fn hide_widget(app_state: &mut AppState, widget_id: WidgetId) {
    
}

pub(super) fn request_layout(app_state: &mut AppState, widget_id: WidgetId) {
	app_state.widget_data_mut(widget_id).set_flag(WidgetFlags::NEEDS_LAYOUT);
	app_state.merge_widget_flags(widget_id);
}

fn update_node_origins(app_state: &mut AppState, root_widget: WidgetId) {
	let mut stack = vec![];
	for child in app_state.widget_data[root_widget].children.iter() {
		stack.push((*child, Point::zero()));
	}

	while let Some((widget_id, parent_origin)) = stack.pop() {
		let data = &mut app_state.widget_data[widget_id];
		let origin = data.offset() + parent_origin;
		for child in data.children.iter() {
			stack.push((*child, origin))
		}
		data.origin = origin;
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

enum LayoutAlgorithm {
	Hidden,
	Block,
	Grid,
	Flex, 
	Leaf
}

pub struct LayoutContext<'a> {
	app_state: &'a mut AppState
}

impl<'a> LayoutContext<'a> {
	fn get_layout_style<'b>(&self, node_id: taffy::NodeId) -> LayoutStyle<'b> {
		LayoutStyle { 
			style: &self.app_state.widget_data[node_id.into()].style, 
			display_style: &self.app_state.widget_data[node_id.into()].display_style,
			scale_factor: todo!(), 
			window_size: todo!() 
		}
	}

	fn get_layout_algorithm(&self, node_id: taffy::NodeId) -> LayoutAlgorithm {
		let data = &self.app_state.widget_data[node_id.into()];
		if data.is_hidden() {
			LayoutAlgorithm::Hidden
		} else if data.children.len() == 0 {
			LayoutAlgorithm::Leaf
		} else {
			match data.display_style {
				DisplayStyle::Block => LayoutAlgorithm::Block,
				DisplayStyle::Flex(_) => LayoutAlgorithm::Flex,
			}
		}
	}
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

impl<'a> PrintTree for LayoutContext<'a> {
	fn get_debug_label(&self, node_id: taffy::NodeId) -> &'static str {
		self.app_state.widgets[node_id.into()].debug_label()
	}

	fn get_final_layout(&self, node_id: taffy::NodeId) -> &taffy::Layout {
		&self.app_state.widget_data[node_id.into()].layout
	}
}

impl<'a> LayoutBlockContainer for LayoutContext<'a> {
    type BlockContainerStyle<'b> = LayoutStyle<'b> where Self: 'b;
    type BlockItemStyle<'b> = LayoutStyle<'b> where Self: 'b;

    fn get_block_container_style(&self, node_id: taffy::NodeId) -> Self::BlockContainerStyle<'_> {
        self.get_layout_style(node_id)
    }

    fn get_block_child_style(&self, child_node_id: taffy::NodeId) -> Self::BlockItemStyle<'_> {
        self.get_layout_style(child_node_id)
    }
}

impl<'a> LayoutFlexboxContainer for LayoutContext<'a> {
    type FlexboxContainerStyle<'b> = LayoutStyle<'b> where Self: 'b;
    type FlexboxItemStyle<'b> = LayoutStyle<'b>  where Self: 'b;

    fn get_flexbox_container_style(&self, node_id: taffy::NodeId) -> Self::FlexboxContainerStyle<'_> {
        self.get_layout_style(node_id)
    }

    fn get_flexbox_child_style(&self, child_node_id: taffy::NodeId) -> Self::FlexboxItemStyle<'_> {
        self.get_layout_style(child_node_id)
    }
}

impl<'a> LayoutGridContainer for LayoutContext<'a> {
    type GridContainerStyle<'b> = LayoutStyle<'b> where Self: 'b;
    type GridItemStyle<'b> = LayoutStyle<'b> where Self: 'b;

    fn get_grid_container_style(&self, node_id: taffy::NodeId) -> Self::GridContainerStyle<'_> {
        self.get_layout_style(node_id)
    }

    fn get_grid_child_style(&self, child_node_id: taffy::NodeId) -> Self::GridItemStyle<'_> {
        self.get_layout_style(child_node_id)
    }
}

impl<'a> LayoutPartialTree for LayoutContext<'a> {
    type CoreContainerStyle<'b> = LayoutStyle<'b> where Self : 'b;
    type CacheMut<'b> = &'b mut taffy::Cache where Self : 'b;
    
    fn get_core_container_style(&self, node_id: taffy::NodeId) -> Self::CoreContainerStyle<'_> {
		self.get_layout_style(node_id)
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
            let widget_data = &mut self.app_state.widget_data[node_id.into()];
            if widget_data.flag_is_set(WidgetFlags::NEEDS_LAYOUT) {
                widget_data.cache.clear();
                widget_data.clear_flag(WidgetFlags::NEEDS_LAYOUT);
            }
        }
        
        taffy::compute_cached_layout(self, node_id, inputs, |tree, node, inputs| {
			match tree.get_layout_algorithm(node) {
				LayoutAlgorithm::Hidden => taffy::compute_hidden_layout(tree, node),
				LayoutAlgorithm::Block => taffy::compute_block_layout(tree, node, inputs),
				LayoutAlgorithm::Grid => taffy::compute_grid_layout(tree, node, inputs),
				LayoutAlgorithm::Flex => taffy::compute_flexbox_layout(tree, node, inputs),
				LayoutAlgorithm::Leaf => {
                    let widget_id = node.into();
                    let style = tree.get_layout_style(node);
                    let widget = &tree.app_state.widgets[widget_id];
                    let measure_function = |known_dimensions, available_space| {
                        widget.measure(known_dimensions, available_space)
                    };
                    taffy::compute_leaf_layout(inputs, &style, measure_function)
                },
			}
        })
    }
}

