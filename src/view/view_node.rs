use crate::core::{Size, Point};
use bitflags::bitflags;

bitflags!(
    #[derive(Debug, Clone, Copy)]
    pub struct ViewFlags : u32 {
        const EMPTY = 0;
        const NEEDS_LAYOUT = 1 << 1;
        const NEEDS_RENDER = 1 << 2;
        const NEEDS_REBUILD = 1 << 3;
    }
);

#[derive(Debug, Clone)]
pub struct ViewNode {
    pub(crate) size: Size,
    pub(crate) origin: Point,
    pub(crate) flags: ViewFlags,
    pub(crate) children: Vec<ViewNode>
}

impl ViewNode {
    pub fn new() -> Self {
        Self {
            size: Size::ZERO,
            origin: Point::ZERO,
            flags: ViewFlags::EMPTY,
            children: Vec::new(),
        }
    }

    pub fn set_flag(&mut self, flag: ViewFlags) {
        self.flags |= flag;
    }

    pub fn clear_flag(&mut self, flag: ViewFlags) {
        self.flags &= !flag;
    }

    pub fn flag_is_set(&mut self, flag: ViewFlags) -> bool{
        self.flags.contains(flag)
    }

    pub fn combine_child_flags(&mut self, index: usize) {
        self.flags |= self.children[index].flags;
    }

    pub fn set_size(&mut self, new_size: Size) {
        self.size = new_size;
    }

    pub fn set_origin(&mut self, new_origin: Point) {
        self.origin = new_origin;
    }
}

/*impl<W:View> ViewNode<W> {
    pub fn new(widget: W, view_id: IdPath) -> Self {
        let state = ViewNode::new(view_id);
        let s = widget.build(&view_id);
        Self { widget, state, s }
    }

    pub fn event(&mut self, event: crate::event::Event, ctx: &mut EventContext<W::Message>) { 
        //self.widget.event(event, ctx);
    }

    pub fn layout(&mut self, constraint: Constraint) -> Size {
        self.widget.layout(constraint)
    }

    pub fn render(&self, bounds: Rectangle, ctx: &mut Renderer) {
        ctx.use_transform(Transform::translate(self.state.origin.into()), |ctx| {
            self.widget.render(bounds, ctx)
        });
    }
}*/

//pub type AnyWidgetNode = ViewNode<Box<dyn AnyView>>;