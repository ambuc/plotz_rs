use crate::draw_obj::DrawObjInner;
use plotz_geometry::point::Pt;

#[derive(Debug, PartialEq, Clone)]
pub struct Group(Vec<DrawObjInner>);

impl Group {
    pub fn new(dois: impl IntoIterator<Item = DrawObjInner>) -> Group {
        Group(dois.into_iter().collect::<Vec<_>>())
    }

    /// to iterator
    pub fn iter_pts(&self) -> Box<dyn Iterator<Item = &Pt> + '_> {
        Box::new(self.0.iter().map(|doi| doi.iter_pts()).flatten())
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
