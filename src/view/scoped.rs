use crate::app::{CreateContext, View};

pub struct Scoped<F> {
    f: F
}

impl<V, F> Scoped<F> 
where 
    V: View,
    F: FnOnce(&mut dyn CreateContext) -> V
{
    pub fn new(f: F) -> Self {
        Self { f }
    }
}

impl<V, F> View for Scoped<F> 
where 
    V: View,
    F: FnOnce(&mut dyn CreateContext) -> V
{
    type Element = V::Element;

    fn build(self, ctx: &mut crate::app::BuildContext<Self::Element>) -> Self::Element {
        let inner_view = (self.f)(ctx);
        inner_view.build(ctx)
    }
}