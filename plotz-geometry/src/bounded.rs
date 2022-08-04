use crate::{
    point::Pt,
    polygon::{Polygon, PolygonConstructorError},
};

#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum BoundingBoxError {
    #[error("could not construct bounding box")]
    PolygonConstructorError(#[from] PolygonConstructorError),
}

pub trait Bounded {
    fn right_bound(&self) -> f64;
    fn left_bound(&self) -> f64;
    fn top_bound(&self) -> f64;
    fn bottom_bound(&self) -> f64;
    fn width(&self) -> f64 {
        self.right_bound() - self.left_bound()
    }
    fn height(&self) -> f64 {
        self.bottom_bound() - self.top_bound()
    }
    fn tl_bound(&self) -> Pt {
        Pt(self.left_bound(), self.top_bound())
    }
    fn tr_bound(&self) -> Pt {
        Pt(self.right_bound(), self.top_bound())
    }
    fn bl_bound(&self) -> Pt {
        Pt(self.left_bound(), self.bottom_bound())
    }
    fn br_bound(&self) -> Pt {
        Pt(self.right_bound(), self.bottom_bound())
    }
    fn bbox(&self) -> Result<Polygon, BoundingBoxError> {
        Ok(Polygon([
            self.tl_bound(),
            self.tr_bound(),
            self.br_bound(),
            self.bl_bound(),
        ])?)
    }
    fn center(&self) -> Pt {
        Pt(
            self.left_bound() + (self.width() / 2.0),
            self.top_bound() + (self.height() / 2.0),
        )
    }
}
