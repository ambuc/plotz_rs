use {
    crate::style::Style3d, plotz_geometry::traits::AnnotationSettings, std::fmt::Debug,
    typed_builder::TypedBuilder,
};

#[derive(Debug, Clone, TypedBuilder)]
pub struct SceneDebug {
    #[builder(default, setter(strip_option))]
    pub draw_wireframes: Option<Style3d>,

    #[builder(default, setter(strip_option))]
    pub annotate: Option<AnnotationSettings>,
}
