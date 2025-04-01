use std::marker::PhantomData;

use crate::{
    app::{AnyView, Signal, View},
    core::Size,
    param::{AnyParameter, AnyParameterGroup, ParamVisitor, ParameterTraversal, Params},
    style::{Length, UiRect},
    views::{Column, Container, Label, ParameterSlider, Row, Scoped, ViewExt},
};

pub trait Editor: 'static {
    type Parameters: Params;

    fn new() -> Self;
    fn view(&self, parameters: &Self::Parameters) -> impl View;
    fn min_size(&self) -> Option<Size> {
        None
    }
    fn max_size(&self) -> Option<Size> {
        None
    }
    fn prefered_size(&self) -> Option<Size> {
        None
    }
}

struct CreateParameterViewsVisitor {
    views: Vec<AnyView>,
}

impl CreateParameterViewsVisitor {
    pub fn new() -> Self {
        Self { views: Vec::new() }
    }
}

impl ParamVisitor for CreateParameterViewsVisitor {
    fn bool_parameter(&mut self, _p: &crate::param::BoolParameter) {}

    fn bypass_parameter(&mut self, _p: &crate::param::ByPassParameter) {}

    fn float_parameter(&mut self, p: &crate::param::FloatParameter) {
        let view = Row::new((Label::new(p.info().name()), ParameterSlider::new(p)));
        self.views.push(view.into_any_view());
    }

    fn int_parameter(&mut self, p: &crate::param::IntParameter) {
        let view = Row::new((
            Label::new(p.info().name()),
            ParameterSlider::new(p).into_any_view(),
        ));
        self.views.push(view.into_any_view());
    }

    fn string_list_parameter(&mut self, _p: &crate::param::StringListParameter) {}

    fn group<P: ParameterTraversal>(&mut self, group: &crate::param::ParameterGroup<P>) {
        let mut child_visitor = Self::new();
        group.children().visit(&mut child_visitor);
        let name = group.name().to_string();

        let view = Scoped::new(move |cx| {
            let hide_children = Signal::new(cx, false);
            Column::new((
                Label::new(name),
                Column::new(child_visitor.views)
                    .style(|style| style.padding(UiRect::left_px(20.0)).hidden(hide_children)),
            ))
            .style(|style| style.padding(UiRect::top_px(10.0)))
        });
        self.views.push(view.into_any_view());
    }
}

pub struct GenericEditor<P> {
    _phantom: PhantomData<P>,
}

impl<P: Params> Editor for GenericEditor<P> {
    type Parameters = P;

    fn new() -> Self {
        Self {
            _phantom: PhantomData,
        }
    }

    fn view(&self, parameters: &P) -> impl View {
        let mut visitor = CreateParameterViewsVisitor::new();
        parameters.visit(&mut visitor);
        Container::new(Column::new(visitor.views))
            .style(|s| s.width(Length::Vw(100.0)).height(Length::Vh(100.0)))
    }
}
