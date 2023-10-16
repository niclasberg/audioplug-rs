use std::marker::PhantomData;
use crate::{ViewMessage, Id, ViewFlags, core::{Rectangle, Point, Color, Vector}, Event, text::TextLayout, Shape, MouseEvent};
use super::{IdPath, ViewNode};
use crate::platform;

pub struct ContextIter<'a> {
    parent_id_path: IdPath,
    current_id: Id,
    nodes_start: *mut ViewNode,
    nodes_end: *mut ViewNode,
    _phantom: PhantomData<&'a mut ViewNode>
}

impl<'a> ContextIter<'a> {
    fn new(parent_id: &IdPath, nodes: &'a mut Vec<ViewNode>) -> Self {
        let range = nodes.as_mut_ptr_range();
        ContextIter { 
            current_id: Id(0),
            nodes_start: range.start,
            nodes_end: range.end,
            parent_id_path: parent_id.clone(),
            _phantom: PhantomData
        }
    }
}

impl<'a> Iterator for ContextIter<'a> {
    type Item = (IdPath, &'a mut ViewNode);

    fn next(&mut self) -> Option<Self::Item> {
        if self.nodes_start == self.nodes_end {
            None
        } else {
            let result = (
                self.parent_id_path.child_id(self.current_id),
                unsafe { self.nodes_start.as_mut().unwrap() }
            );

            self.nodes_start = unsafe { self.nodes_start.offset(1) };
            self.current_id = self.current_id.next();
            Some(result)
        }
    }
}

impl<'a> DoubleEndedIterator for ContextIter<'a> {
    fn next_back(&mut self) -> Option<Self::Item> {
        if self.nodes_start == self.nodes_end {
            None
        } else {
            self.nodes_end = unsafe { self.nodes_end.offset(-1) };
            let offset_from_start = unsafe { self.nodes_end.offset_from(self.nodes_start) };
            let id = Id(self.current_id.0 + offset_from_start as usize);
            Some((
                self.parent_id_path.child_id(id),
                unsafe { self.nodes_end.as_mut().unwrap() }
            ))
        }
    }
}

pub struct BuildContext<'a> {
    id_path: IdPath,
    node: &'a mut ViewNode,
}

pub struct BuildContextIter<'a> {
    node_iter: ContextIter<'a>,
}

impl<'a> Iterator for BuildContextIter<'a> {
    type Item = BuildContext<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        self.node_iter.next().map(|(id_path, node)| {
            BuildContext { id_path, node }
        })
    }
}

impl<'a> BuildContext<'a> {
    pub fn root(node: &'a mut ViewNode) -> Self {
        Self {
            id_path: IdPath::root(),
            node,
        }
    }

    pub fn set_number_of_children(&mut self, count: usize) {
        self.node.children = Vec::from_iter(std::iter::repeat(ViewNode::new()).take(count));
    }

    pub fn id_path(&self) -> &IdPath {
        &self.id_path
    }

    pub fn child_iter(&mut self) -> BuildContextIter<'_> {
        BuildContextIter { node_iter: ContextIter::new(&self.id_path, &mut self.node.children) }
    }

    pub fn get_child<'s>(&'s mut self, id: Id) -> Option<BuildContext<'s>> {
        self.node.children.get_mut(id.0).map(|node| {
            BuildContext { 
                id_path: self.id_path.child_id(id), 
                node
            }
        })
    }

    pub fn with_child<'s, T>(&'s mut self, id: Id, f: impl FnOnce(&mut BuildContext<'s>) -> T) -> T {
        let mut child_ctx = BuildContext { 
            id_path: self.id_path.child_id(id), 
            node: self.node.children.get_mut(id.0).unwrap()
        };
        f(&mut child_ctx)
    }
}

pub struct LayoutContext<'a> {
    id_path: IdPath,
    pub node: &'a mut ViewNode,
}

pub struct LayoutContextIter<'a> {
    node_iter: ContextIter<'a>
}

impl<'a> Iterator for LayoutContextIter<'a> {
    type Item = LayoutContext<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        self.node_iter.next().map(|(id_path, node)| {
            LayoutContext { id_path, node }
        })
    }
}

impl<'a> LayoutContext<'a> {
    pub fn new(node: &'a mut ViewNode) -> Self {
        Self { node, id_path: IdPath::root() }
    }

    pub fn request_render(&mut self) {
        self.node.set_flag_recursive(ViewFlags::NEEDS_RENDER);
    }

    pub fn id_path(&self) -> &IdPath {
        &self.id_path
    }

    pub fn child_iter(&mut self) -> LayoutContextIter<'_> {
        LayoutContextIter { node_iter: ContextIter::new(&self.id_path, &mut self.node.children) }
    }

    pub fn with_child<T>(&mut self, id: Id, f: impl FnOnce(&mut LayoutContext<'_>) -> T) -> T {
        let result = {
            let child = self.node.children.get_mut(id.0).unwrap();
            let mut child_ctx = LayoutContext { 
                id_path: self.id_path.child_id(id), 
                node: child
            };
            f(&mut child_ctx)
        };

        self.node.combine_child_flags(id.0);

        result
    }
}

pub struct RenderContext<'a> {
    id_path: IdPath,
    node: &'a mut ViewNode,
    renderer: &'a mut platform::Renderer,
}

impl<'a> RenderContext<'a> {
    pub(crate) fn new(node: &'a mut ViewNode, renderer: &'a mut platform::Renderer) -> Self {
        Self { node, id_path: IdPath::root(), renderer}
    }

    pub fn local_bounds(&self) -> Rectangle {
        Rectangle::new(Point::ZERO, self.node.size())
    }

    pub fn fill(&mut self, shape: &Shape, origin: Point, color: Color) {
        match shape {
            Shape::Rect { size } => 
                self.renderer.fill_rectangle(Rectangle::new(
                    origin - Vector::new(size.width / 2.0, size.height / 2.0), *size), 
                    color),
            Shape::RoundedRect { size, corner_radius } => 
                self.renderer.fill_rounded_rectangle(Rectangle::new(
                    origin- Vector::new(size.width / 2.0, size.height / 2.0) , *size), 
                    *corner_radius, 
                    color),
            Shape::Ellipse { radii } => 
                self.renderer.fill_ellipse(origin, *radii, color),
        }
    }

    pub fn draw_rectangle(&mut self, rect: Rectangle, color: Color, line_width: f32) {
        self.renderer.draw_rectangle(rect, color, line_width)
    }

    pub fn draw_text(&mut self, text_layout: &TextLayout, position: Point, color: Color) {
        self.renderer.draw_text(&text_layout.0, position, color)
    }

    pub fn with_child<T>(&mut self, id: Id, f: impl FnOnce(&mut RenderContext<'_>) -> T) -> T {
        let child = self.node.children.get_mut(id.0).unwrap();
        self.renderer.set_offset(child.origin().into());
        let mut child_ctx = RenderContext { 
            id_path: self.id_path.child_id(id), 
            node: child, 
            renderer: self.renderer
        };
        f(&mut child_ctx)
    }
}

pub struct EventContext<'a, Msg: 'static> {
    id_path: IdPath,
    node: &'a mut ViewNode,
    pub(crate) messages: &'a mut Vec<ViewMessage<Msg>>,
    is_handled: &'a mut bool
}

/*pub struct EventContextIter<'a, Msg: 'static> {
    node_iter: ContextIter<'a>,
    messages: *mut Vec<ViewMessage<Msg>>,
    is_handled: &'a mut bool,
}

impl<'a, Msg: 'static> Iterator for EventContextIter<'a, Msg> {
    type Item = EventContext<'a, Msg>;

    fn next(&mut self) -> Option<Self::Item> {
        self.node_iter.next().map(|(id_path, node)| {
            EventContext { 
                id_path, 
                node, 
                messages: unsafe {&mut *self.messages },
                is_handled: self.is_handled
            }
        })
    }
}

impl<'a, Msg: 'static> DoubleEndedIterator for EventContextIter<'a, Msg> {
    fn next_back(&mut self) -> Option<Self::Item> {
        self.node_iter.next_back().map(|(id_path, node)| {
            EventContext { 
                id_path, 
                node, 
                messages: unsafe {&mut *self.messages },
                is_handled: self.is_handled 
            }
        })
    }
}*/

impl<'a, Msg> EventContext<'a, Msg> {
    pub fn new(node: &'a mut ViewNode, messages: &'a mut Vec<ViewMessage<Msg>>, is_handled: &'a mut bool) -> Self{
        Self { id_path: IdPath::root(), node, messages, is_handled }
    }

    pub fn id_path(&self) -> &IdPath {
        &self.id_path
    }

    pub fn local_bounds(&self) -> Rectangle {
        Rectangle::new(Point::ZERO, self.node.size())
    }

    pub fn set_handled(&mut self) {
        *self.is_handled = true;
    }

    pub fn is_handled(&self) -> bool {
        *self.is_handled
    }

    /*pub fn child_iter(&mut self) -> EventContextIter<'_, Msg> {
        EventContextIter { 
            node_iter: ContextIter::new(&self.id_path, &mut self.node.children),
            messages: self.messages,
            is_handled: self.is_handled
        }
    }*/

    pub fn forward_to_child(&mut self, id: Id, event: Event, mut f: impl FnMut(&mut EventContext<'_, Msg>, Event)) {
        if *self.is_handled {
            return;
        }

        let child = self.node.children.get_mut(id.0).unwrap();
        let offset = child.offset();
        let bounds = child.local_bounds();

        match event {
            Event::Mouse(mouse_event) => {
                let mouse_event = mouse_event.with_offset(offset);
                match mouse_event {
                    MouseEvent::Moved { position } if bounds.contains(position) => {
                        if !child.flag_is_set(ViewFlags::UNDER_MOUSE_CURSOR) {
                            child.set_flag(ViewFlags::UNDER_MOUSE_CURSOR);
                            f(&mut EventContext { 
                                id_path: self.id_path.child_id(id), 
                                node: child, 
                                messages: &mut self.messages, 
                                is_handled: self.is_handled
                            }, Event::Mouse(MouseEvent::Enter));
                        }

                        f(&mut EventContext { 
                            id_path: self.id_path.child_id(id), 
                            node: child, 
                            messages: &mut self.messages, 
                            is_handled: self.is_handled
                        }, Event::Mouse(mouse_event));
                    },
                    // Mouse exited the child's parent, or the mouse moved
                    // and the mouse is not over the child
                    MouseEvent::Moved { .. } | MouseEvent::Exit => {
                        if child.flag_is_set(ViewFlags::UNDER_MOUSE_CURSOR) {
                            child.clear_flag(ViewFlags::UNDER_MOUSE_CURSOR);
                            f(&mut EventContext { 
                                id_path: self.id_path.child_id(id), 
                                node: child, 
                                messages: &mut self.messages, 
                                is_handled: self.is_handled
                            }, Event::Mouse(MouseEvent::Exit));
                        }
                        f(&mut EventContext { 
                            id_path: self.id_path.child_id(id), 
                            node: child, 
                            messages: &mut self.messages, 
                            is_handled: self.is_handled
                        }, Event::Mouse(mouse_event));
                    },
                    // Filter these out
                    MouseEvent::Enter => {},
                    _ => f(&mut EventContext { 
                        id_path: self.id_path.child_id(id), 
                        node: child, 
                        messages: &mut self.messages, 
                        is_handled: self.is_handled
                    }, Event::Mouse(mouse_event))
                };
            },
            other => f(&mut EventContext { 
                id_path: self.id_path.child_id(id), 
                node: child, 
                messages: &mut self.messages, 
                is_handled: self.is_handled
            }, other),
        };
    }

    pub fn with_message_container<'s, Msg2: 'static>(&'s mut self, messages: &'s mut Vec<ViewMessage<Msg2>>, f: impl FnOnce(&mut EventContext<'s, Msg2>)) {
        let mut ctx = EventContext {
            id_path: self.id_path.clone(),
            node: self.node,
            messages,
            is_handled: self.is_handled
        };
        f(&mut ctx)
    }

    pub fn request_layout(&mut self) {
        self.node.set_flag(ViewFlags::NEEDS_LAYOUT);
    }

    pub fn request_render(&mut self) {
        self.node.set_flag(ViewFlags::NEEDS_RENDER);
    }

    pub fn request_rebuild(&mut self) {
        self.node.set_flag(ViewFlags::NEEDS_REBUILD);
    }

    pub fn publish_message(&mut self, message: Msg) {
        self.messages.push(ViewMessage { 
            view_id: self.id_path().clone(),
            message
        });
    }
}
