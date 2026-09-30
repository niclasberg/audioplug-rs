use std::ops::DerefMut;

use crate::{event::MouseClickEvent, ui::EventContext};

pub struct EventHandler<E> {
    f: Box<dyn Fn(&mut EventContext, E)>,
    next: Option<Box<Self>>,
}

impl<E> EventHandler<E> {
    pub fn add(&mut self, next: Self) {
        let mut cur = self;
        while let Some(next) = cur.next.as_mut() {
            cur = next.deref_mut();
        }
        
    }
}

pub struct EventHandlers {
    on_click: Option<EventHandler<MouseClickEvent>>,
}
