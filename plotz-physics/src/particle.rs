use plotz_geometry::shapes::point::Pt;
use typed_builder::TypedBuilder;

pub enum Mobility {
    Fixed,
    Mobile,
}

pub enum Visibility {
    Visible,
    Invisible,
}

#[derive(TypedBuilder)]
pub struct Particle<T> {
    #[builder(setter(into))]
    pub position: Pt,

    pub mobility: Mobility,

    #[builder(default=Visibility::Visible)]
    pub visibility: Visibility,

    #[builder(default = None, setter(strip_option))]
    pub charge: Option<f64>,

    #[builder(default=None, setter(strip_option))]
    pub metadata: Option<T>,

    #[builder(default=vec![])]
    pub history: Vec<Pt>,
}

impl<T> Particle<T> {
    pub fn is_fixed(&self) -> bool {
        matches!(self.mobility, Mobility::Fixed)
    }

    pub fn is_visible(&self) -> bool {
        matches!(self.visibility, Visibility::Visible)
    }
}
