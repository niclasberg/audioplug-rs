use crate::view::IdPath;

pub struct ViewMessage<T> {
    pub view_id: IdPath,
    pub message: T,
}