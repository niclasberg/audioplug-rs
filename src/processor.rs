use crate::param::Params;

pub trait Processor: Send + 'static {
    type Parameters: Params;

    fn process();
}
