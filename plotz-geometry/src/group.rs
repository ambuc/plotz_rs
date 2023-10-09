//! A group of objects.

use crate::{obj2::Obj2, style::Style};

use {
    crate::{
        bounded::{Bounded, Bounds, BoundsCollector},
        crop::{CropType, Croppable},
        shapes::{pg2::Pg2, pt2::Pt2},
        traits::*,
    },
    std::ops::*,
};

#[derive(Debug, PartialEq, Clone)]
/// A group of objects.
pub struct Group<T>(Vec<(Obj2, T)>);

impl<T> Group<T> {
    /// Creates a new group.
    pub fn new(objs: impl IntoIterator<Item = (Obj2, T)>) -> Group<T> {
        Group(objs.into_iter().collect::<Vec<_>>())
    }

    /// Returns a boxed iterator of immutable Object2dInners, the members of this
    /// group.
    pub fn iter_objects(&self) -> Box<dyn Iterator<Item = &(Obj2, T)> + '_> {
        Box::new(self.0.iter())
    }

    /// Mutates each point in each object in the group. See |Mutable|.
    pub fn mutate(&mut self, f: impl Fn(&mut Pt2)) {
        for (obj, _) in &mut self.0 {
            obj.mutate(&f);
        }
    }
}

impl<T> YieldPoints for Group<T> {
    fn yield_pts(&self) -> Box<dyn Iterator<Item = &Pt2> + '_> {
        Box::new(self.0.iter().flat_map(|(obj, _)| obj.yield_pts()))
    }
}
impl<T> YieldPointsMut for Group<T> {
    fn yield_pts_mut(&mut self) -> Box<dyn Iterator<Item = &mut Pt2> + '_> {
        Box::new(self.0.iter_mut().flat_map(|(obj, _)| obj.yield_pts_mut()))
    }
}

impl<T> Bounded for Group<T> {
    fn bounds(&self) -> Bounds {
        let mut bc = BoundsCollector::default();
        for pt in self.yield_pts() {
            bc.incorporate(pt);
        }
        bc.bounds()
    }
}

impl<T> AddAssign<Pt2> for Group<T> {
    fn add_assign(&mut self, rhs: Pt2) {
        self.0.iter_mut().for_each(|(o, _)| {
            *o += rhs;
        });
    }
}

impl<T> SubAssign<Pt2> for Group<T> {
    fn sub_assign(&mut self, rhs: Pt2) {
        self.0.iter_mut().for_each(|(o, _)| {
            *o -= rhs;
        });
    }
}

impl<T> Add<Pt2> for Group<T> {
    type Output = Self;
    fn add(self, rhs: Pt2) -> Self::Output {
        Self::new(self.0.into_iter().map(|(o, s)| (o + rhs, s)))
    }
}
impl<T> Sub<Pt2> for Group<T> {
    type Output = Self;
    fn sub(self, rhs: Pt2) -> Self::Output {
        Self::new(self.0.into_iter().map(|(o, s)| (o - rhs, s)))
    }
}

impl<T> Mul<f64> for Group<T> {
    type Output = Self;
    fn mul(self, rhs: f64) -> Self::Output {
        Self::new(self.0.into_iter().map(|(o, s)| (o * rhs, s)))
    }
}

impl<T> MulAssign<f64> for Group<T> {
    fn mul_assign(&mut self, rhs: f64) {
        self.0.iter_mut().for_each(|(o, _)| {
            *o *= rhs;
        })
    }
}

impl<T> Div<f64> for Group<T> {
    type Output = Self;
    fn div(self, rhs: f64) -> Self::Output {
        Self::new(self.0.into_iter().map(|(o, s)| (o / rhs, s)))
    }
}

impl<T> DivAssign<f64> for Group<T> {
    fn div_assign(&mut self, rhs: f64) {
        self.0.iter_mut().for_each(|(o, _)| {
            *o /= rhs;
        })
    }
}

impl<T> RemAssign<Pt2> for Group<T> {
    fn rem_assign(&mut self, rhs: Pt2) {
        self.0.iter_mut().for_each(|(o, _)| *o %= rhs);
    }
}

impl<T> Translatable for Group<T> {}

impl<T> Scalable<f64> for Group<T> {}

impl<T> Croppable for Group<T>
where
    T: Clone,
{
    type Output = Group<T>;
    fn crop(&self, frame: &Pg2, crop_type: CropType) -> Vec<Self::Output> {
        vec![Group::new(
            self.0
                .iter()
                .flat_map(|(obj, s)| {
                    obj.crop(frame, crop_type)
                        .into_iter()
                        .map(|o| (o, s.clone()))
                })
                .collect::<Vec<(Obj2, T)>>(),
        )]
    }
    fn crop_excluding(&self, _other: &Pg2) -> Vec<Self::Output>
    where
        Self: Sized,
    {
        unimplemented!("TODO");
    }
}

impl<T> Nullable for Group<T> {
    fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

impl<T> Annotatable for Group<T> {
    fn annotate(&self, settings: &AnnotationSettings) -> Vec<(Obj2, Style)> {
        self.0
            .iter()
            .flat_map(|(o, _)| o.annotate(settings))
            .collect()
    }
}
