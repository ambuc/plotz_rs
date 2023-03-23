use crate::draw_obj::DrawObjInner;
use float_ord::FloatOrd;
use plotz_geometry::bounded::Bounded;
use plotz_geometry::point::Pt;
use plotz_geometry::traits::{YieldPoints, YieldPointsMut};

#[derive(Debug, PartialEq, Clone)]
pub struct Group(Vec<DrawObjInner>);

impl Group {
    pub fn new(dois: impl IntoIterator<Item = DrawObjInner>) -> Group {
        Group(dois.into_iter().collect::<Vec<_>>())
    }

    pub fn iter_dois(&self) -> Box<dyn Iterator<Item = &DrawObjInner> + '_> {
        Box::new(self.0.iter())
    }

    pub fn mutate(&mut self, f: impl Fn(&mut Pt)) {
        for doi in &mut self.0 {
            doi.mutate(&f);
        }
    }
}

impl YieldPoints for Group {
    fn yield_pts(&self) -> Box<dyn Iterator<Item = &Pt> + '_> {
        Box::new(
            self.0
                .iter()
                .flat_map(|doi| doi.inner_impl_yield_points().yield_pts()),
        )
    }
}
impl YieldPointsMut for Group {
    fn yield_pts_mut(&mut self) -> Box<dyn Iterator<Item = &mut Pt> + '_> {
        Box::new(
            self.0
                .iter_mut()
                .flat_map(|doi| doi.inner_impl_yield_points_mut().yield_pts_mut()),
        )
    }
}

impl Bounded for Group {
    fn right_bound(&self) -> f64 {
        self.iter_dois()
            .map(|doi| FloatOrd(doi.right_bound()))
            .max()
            .unwrap()
            .0
    }
    fn left_bound(&self) -> f64 {
        self.iter_dois()
            .map(|doi| FloatOrd(doi.left_bound()))
            .min()
            .unwrap()
            .0
    }
    fn bottom_bound(&self) -> f64 {
        self.iter_dois()
            .map(|doi| FloatOrd(doi.bottom_bound()))
            .max()
            .unwrap()
            .0
    }
    fn top_bound(&self) -> f64 {
        self.iter_dois()
            .map(|doi| FloatOrd(doi.top_bound()))
            .min()
            .unwrap()
            .0
    }
}
