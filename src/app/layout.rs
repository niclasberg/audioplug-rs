use taffy::{CacheTree, LayoutBlockContainer, LayoutFlexboxContainer, LayoutPartialTree, PrintTree, TraversePartialTree, TraverseTree};
use crate::{core::{Point, Size}, style::{AvailableSpace, DisplayStyle, LayoutStyle}};

use super::{invalidate_window, WidgetFlags, AppState, WidgetId, WindowId};

pub fn layout_window(app_state: &mut AppState, window_id: WindowId) {
    let (window_size, widget_id) = {
        let window = app_state.window(window_id);
        (window.handle.global_bounds().size(), window.root_widget)
    };

    {
        let available_space = taffy::Size {
            width: taffy::AvailableSpace::Definite(window_size.width as f32),
            height: taffy::AvailableSpace::Definite(window_size.height as f32),
        };
        let mut ctx = LayoutContext { app_state, window_size };
        taffy::compute_root_layout(&mut ctx, widget_id.into(), available_space);
        //taffy::print_tree(&mut ctx, widget_id.into());
    }

    update_node_origins(app_state, widget_id);
    invalidate_window(app_state, window_id);
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

pub struct LayoutContext<'a> {
	app_state: &'a mut AppState,
    window_size: Size
}

impl<'a> LayoutContext<'a> {
	fn get_layout_style<'b>(&'b self, node_id: taffy::NodeId) -> LayoutStyle<'b> {
		LayoutStyle { 
			style: &self.app_state.widget_data[node_id.into()].style, 
			display_style: self.app_state.widgets[node_id.into()].display_style(),
            window_size: self.window_size
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

impl<'a> CacheTree for LayoutContext<'a> {
    fn cache_get(
        &self,
        node_id: taffy::NodeId,
        known_dimensions: taffy::Size<Option<f32>>,
        available_space: taffy::Size<taffy::AvailableSpace>,
        run_mode: taffy::RunMode,
    ) -> Option<taffy::LayoutOutput> {
        self.app_state.widget_data[node_id.into()].cache.get(known_dimensions, available_space, run_mode)
    }

    fn cache_store(
        &mut self,
        node_id: taffy::NodeId,
        known_dimensions: taffy::Size<Option<f32>>,
        available_space: taffy::Size<taffy::AvailableSpace>,
        run_mode: taffy::RunMode,
        layout_output: taffy::LayoutOutput,
    ) {
        self.app_state.widget_data[node_id.into()].cache.store(known_dimensions, available_space, run_mode, layout_output)
    }

    fn cache_clear(&mut self, node_id: taffy::NodeId) {
        self.app_state.widget_data[node_id.into()].cache.clear()
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

impl<'a> LayoutPartialTree for LayoutContext<'a> {
    type CoreContainerStyle<'b> = LayoutStyle<'b> where Self : 'b;
    
    fn get_core_container_style(&self, node_id: taffy::NodeId) -> Self::CoreContainerStyle<'_> {
		self.get_layout_style(node_id)
    }

    fn set_unrounded_layout(&mut self, node_id: taffy::NodeId, layout: &taffy::Layout) {
        self.app_state.widget_data[node_id.into()].layout = *layout;
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
            if tree.app_state.widget_data[node_id.into()].is_hidden() {
                taffy::compute_hidden_layout(tree, node)
            } else {
                let has_children = tree.child_count(node) > 0;
                let display_style = tree.app_state.widgets[node.into()].display_style();
                match (display_style, has_children) {
                    (DisplayStyle::Block, true) => taffy::compute_block_layout(tree, node, inputs),
                    (DisplayStyle::Flex(_), true) => taffy::compute_flexbox_layout(tree, node, inputs),
					(DisplayStyle::Grid(_), _) => unreachable!(),
                    (DisplayStyle::Leaf(measure), _) => {
                        let style = &tree.app_state.widget_data[node.into()].style;    
                        let measure_function = |known_dimensions: taffy::Size<Option<f32>>, available_space: taffy::Size<taffy::AvailableSpace>| {
                            let available_size = known_dimensions.zip_map(available_space, |known, available| {
                                known.map(|x| AvailableSpace::Exact(x as f64)).unwrap_or_else(|| match available {
                                    taffy::AvailableSpace::Definite(x) => AvailableSpace::Exact(x as f64),
                                    taffy::AvailableSpace::MinContent => AvailableSpace::MinContent,
                                    taffy::AvailableSpace::MaxContent => AvailableSpace::MaxContent,
                                })
                            });

                            let size = measure.measure(&style, available_size.width, available_size.height);
                            taffy::Size {
                                width: size.width as _,
                                height: size.height as _,
                            }
                        };
                        taffy::compute_leaf_layout(inputs, &tree.get_layout_style(node_id), measure_function)
                    }, 
                    (_, false) => {
                        let measure_function = |_, _| { taffy::Size::ZERO };
                        taffy::compute_leaf_layout(inputs, &tree.get_layout_style(node_id), measure_function)
                    } 
                } 
            }
        })
    }
}

