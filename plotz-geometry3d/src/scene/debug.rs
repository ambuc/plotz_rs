use plotz_geometry::{style::Style, *};
use std::fmt::Debug;
use typed_builder::TypedBuilder;

#[derive(Debug, Clone, TypedBuilder)]
pub struct SceneDebug {
    #[builder(default, setter(strip_option))]
    pub draw_wireframes: Option<Style>,

    #[builder(default, setter(strip_option))]
    pub annotate: Option<AnnotationSettings>,
}
