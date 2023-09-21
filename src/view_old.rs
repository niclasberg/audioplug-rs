use std::any::Any;

use bitflags::bitflags;

use crate::{widget::{Widget, AnyWidget}, Message, Id, IdPath};

pub struct BuildContext {
    id_path: Vec<Id>
}

impl BuildContext {
    pub fn new() -> Self {
        Self { id_path: Vec::new() }
    }

    pub fn with_id<T>(&mut self, id: Id, _f: impl Fn(&mut Self) -> T) -> (IdPath, T) {
        self.id_path.push(id);
        //let result = f(self);
        self.id_path.pop();
        todo!()
    }
}

bitflags!(
    pub struct ChangeFlags : u32 {
        const LAYOUT = 0x01;
        const TREE_STRUCTURE = 0x02;
    }
);

pub trait View {
    type Element: Widget;
    type State;

    fn build(&self, id_path: &IdPath) -> (Self::State, Self::Element);
    fn rebuild(&self, id_path: &IdPath, prev: &Self, state: &mut Self::State, widget: &mut Self::Element) -> ChangeFlags;
    fn message(&mut self, _msg: &Message<<Self::Element as Widget>::Message>) {}
    //fn visit(&mut self, f: &dyn FnMut(&mut dyn AnyView));
}

pub trait AnyView {
    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;
    fn dyn_build(&self, id_path: &IdPath) -> (Box<dyn Any>, Box<dyn AnyWidget>);
    fn dyn_rebuild(&self, id_path: &IdPath, prev: &Box<dyn AnyView>, state: &mut Box<dyn Any>, widget: &mut Box<dyn AnyWidget>) -> ChangeFlags;
    //fn dyn_visit(&mut self, f: &dyn FnMut(&mut dyn AnyView));
}

impl<V: View + 'static> AnyView for V 
where 
    V::State: 'static,
    V::Element: 'static
{
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn dyn_build(&self, id_path: &IdPath) -> (Box<dyn Any>, Box<dyn AnyWidget>) {
        let (state, widget) = self.build(id_path);
        (Box::new(state), Box::new(widget))
    }

    fn dyn_rebuild(&self, id_path: &IdPath, prev: &Box<dyn AnyView>, state: &mut Box<dyn Any>, widget: &mut Box<dyn AnyWidget>) -> ChangeFlags {
        if let Some(prev) = prev.as_any().downcast_ref() {
            let state = state.downcast_mut().expect("State has wrong type");
            let widget = widget.as_any_mut().downcast_mut().expect("Widget has wrong type");
            self.rebuild(id_path, prev, state, widget)
        } else {
            let (new_state, new_widget) = self.build(id_path);
            *state = Box::new(new_state);
            *widget = Box::new(new_widget);
            ChangeFlags::TREE_STRUCTURE
        }
    }

    /*fn dyn_visit(&mut self, f: &mut dyn FnMut(&mut dyn AnyView)) {
        f(self)
    }*/
}

impl View for Box<dyn AnyView> {
    type Element = Box<dyn AnyWidget>;
    type State = Box<dyn Any>;

    fn build(&self, id_path: &IdPath) -> (Self::State, Self::Element) {
        self.dyn_build(id_path)
    }

    fn rebuild(&self, id_path: &IdPath, prev: &Self, state: &mut Self::State, widget: &mut Self::Element) -> ChangeFlags {
        self.dyn_rebuild(id_path, prev, state, widget)
    }

    /*fn visit(&mut self, f: &dyn FnMut(&mut dyn AnyView)) {
        self.dyn_visit(f)
    }*/
}
