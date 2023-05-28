//! A group of objects.

use {
    crate::{
        bounded::{Bounded, BoundsCollector},
        crop::{CropType, Croppable},
        object2d::Object2d,
        shapes::{point::Pt, polygon::Polygon},
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
    fn yield_pts(&self) -> Box<dyn Iterator<Item = &Pt> + '_> {
        Box::new(
            self.0
                .iter()
                .flat_map(|obj| obj.inner_impl_yield_points().yield_pts()),
        )
    }
}
impl YieldPointsMut for Group {
    fn yield_pts_mut(&mut self) -> Box<dyn Iterator<Item = &mut Pt> + '_> {
        Box::new(
            self.0
                .iter_mut()
                .flat_map(|obj| obj.inner_impl_yield_points_mut().yield_pts_mut()),
        )
    }
}

impl Bounded for Group {
    fn bounds(&self) -> crate::bounded::Bounds {
        let mut bc = BoundsCollector::default();
        for pt in self.yield_pts() {
            bc.incorporate(pt);
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
    fn crop(&self, frame: &Polygon, crop_type: CropType) -> Vec<Self::Output> {
        vec![Group::new(
            self.0
                .iter()
                .flat_map(|d_o| d_o.crop(frame, crop_type))
                .collect::<Vec<_>>(),
        )]
    }
    fn crop_excluding(&self, _other: &Polygon) -> Vec<Self::Output>
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

impl Annotatable for Group {
    fn annotate(&self, settings: &AnnotationSettings) -> Vec<Object2d> {
        self.0.iter().flat_map(|o| o.annotate(settings)).collect()
    }
}
