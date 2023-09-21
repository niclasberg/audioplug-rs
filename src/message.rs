use crate::view::IdPath;

pub enum Message<T> {
    Widget(T)
}

pub struct ViewMessage<T> {
    pub view_id: IdPath,
    pub body: Message<T>,
}