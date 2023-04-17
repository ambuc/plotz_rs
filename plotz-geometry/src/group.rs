//! A group of objects.

use crate::bounded::BoundsCollector;

use {
    crate::{
        bounded::Bounded,
        crop::{CropToPolygonError, Croppable},
        object2d::Object2d,
        point::Pt,
        polygon::Polygon,
        traits::*,
    },
    std::ops::*,
};

#[derive(Debug, PartialEq, Clone)]
/// A group of objects.
pub struct Group(Vec<Object2d>);

impl Group {
    /// Creates a new group.
    pub fn new(objs: impl IntoIterator<Item = Object2d>) -> Group {
        Group(objs.into_iter().collect::<Vec<_>>())
    }

    /// Returns a boxed iterator of immutable Object2dInners, the members of this
    /// group.
    pub fn iter_objects(&self) -> Box<dyn Iterator<Item = &Object2d> + '_> {
        Box::new(self.0.iter())
    }

    /// Mutates each point in each object in the group. See |Mutable|.
    pub fn mutate(&mut self, f: impl Fn(&mut Pt)) {
        for obj in &mut self.0 {
            obj.mutate(&f);
        }
    }
}

impl YieldPoints for Group {
    fn yield_pts(&self) -> Option<Box<dyn Iterator<Item = &Pt> + '_>> {
        Some(Box::new(
            self.0
                .iter()
                .flat_map(|obj| obj.inner_impl_yield_points().yield_pts())
                .flatten(),
        ))
    }
}
impl YieldPointsMut for Group {
    fn yield_pts_mut(&mut self) -> Option<Box<dyn Iterator<Item = &mut Pt> + '_>> {
        Some(Box::new(
            self.0
                .iter_mut()
                .flat_map(|obj| obj.inner_impl_yield_points_mut().yield_pts_mut())
                .flatten(),
        ))
    }
}

impl Bounded for Group {
    fn bounds(&self) -> crate::bounded::Bounds {
        let mut bc = BoundsCollector::default();
        if let Some(iter) = self.yield_pts() {
            for pt in iter {
                bc.incorporate(pt);
            }
        }
        bc.bounds()
    }
}

impl AddAssign<Pt> for Group {
    fn add_assign(&mut self, rhs: Pt) {
        self.0.iter_mut().for_each(|o| {
            *o += rhs;
        });
    }
}

impl SubAssign<Pt> for Group {
    fn sub_assign(&mut self, rhs: Pt) {
        self.0.iter_mut().for_each(|o| {
            *o -= rhs;
        });
    }
}

impl Add<Pt> for Group {
    type Output = Self;
    fn add(self, rhs: Pt) -> Self::Output {
        Self::new(self.0.into_iter().map(|o| o + rhs))
    }
}
impl Sub<Pt> for Group {
    type Output = Self;
    fn sub(self, rhs: Pt) -> Self::Output {
        Self::new(self.0.into_iter().map(|o| o - rhs))
    }
}

impl Mul<f64> for Group {
    type Output = Self;
    fn mul(self, rhs: f64) -> Self::Output {
        Self::new(self.0.into_iter().map(|o| o * rhs))
    }
}

impl MulAssign<f64> for Group {
    fn mul_assign(&mut self, rhs: f64) {
        self.0.iter_mut().for_each(|o| {
            *o *= rhs;
        })
    }
}

impl Div<f64> for Group {
    type Output = Self;
    fn div(self, rhs: f64) -> Self::Output {
        Self::new(self.0.into_iter().map(|o| o / rhs))
    }
}

impl DivAssign<f64> for Group {
    fn div_assign(&mut self, rhs: f64) {
        self.0.iter_mut().for_each(|o| {
            *o /= rhs;
        })
    }
}

impl RemAssign<Pt> for Group {
    fn rem_assign(&mut self, rhs: Pt) {
        self.0.iter_mut().for_each(|o| *o %= rhs);
    }
}

impl Translatable for Group {}
impl Scalable<f64> for Group {}

impl Croppable for Group {
    type Output = Group;
    fn crop_to(&self, frame: &Polygon) -> Result<Vec<Self::Output>, CropToPolygonError> {
        Ok(vec![Group::new(
            self.0
                .iter()
                .flat_map(|d_o| d_o.crop_to(frame))
                .flatten()
                .collect::<Vec<_>>(),
        )])
    }
    fn crop_excluding(&self, _other: &Polygon) -> Result<Vec<Self::Output>, CropToPolygonError>
    where
        Self: Sized,
    {
        unimplemented!("TODO");
    }
}

impl Nullable for Group {
    fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}
