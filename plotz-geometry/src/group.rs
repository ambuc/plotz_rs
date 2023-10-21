//! A group of objects.

use crate::{
    bounded::{Bounded, Bounds, BoundsCollector},
    crop::{CropType, Croppable},
    obj::Obj,
    shapes::{pg::Pg, pt::Pt},
    style::Style,
    *,
};
use anyhow::Result;
use std::ops::*;

#[derive(Debug, PartialEq, Clone)]
/// A group of objects.
pub struct Group<T>(Vec<(Obj, T)>);

impl<T> Group<T> {
    /// Creates a new group.
    pub fn new(objs: impl IntoIterator<Item = (Obj, T)>) -> Group<T> {
        Group(objs.into_iter().collect::<Vec<_>>())
    }

    /// Returns a boxed iterator of immutable Object2dInners, the members of this
    /// group.
    pub fn iter_objects(&self) -> Box<dyn Iterator<Item = &(Obj, T)> + '_> {
        Box::new(self.0.iter())
    }

    /// Iterator.
    pub fn iter(&self) -> impl Iterator<Item = &Pt> {
        self.0.iter().flat_map(|(x, _)| x.iter())
    }

    /// Mutable iterator.
    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut Pt> {
        self.0.iter_mut().flat_map(|(x, _)| x.iter_mut())
    }
}

impl<T> Bounded for Group<T> {
    fn bounds(&self) -> Bounds {
        let mut bc = BoundsCollector::default();
        for pt in self.iter() {
            bc.incorporate(pt);
        }
        bc.bounds()
    }
}

impl<T> AddAssign<Pt> for Group<T> {
    fn add_assign(&mut self, rhs: Pt) {
        self.0.iter_mut().for_each(|(o, _)| {
            *o += rhs;
        });
    }
}

impl<T> SubAssign<Pt> for Group<T> {
    fn sub_assign(&mut self, rhs: Pt) {
        self.0.iter_mut().for_each(|(o, _)| {
            *o -= rhs;
        });
    }
}

impl<T> Add<Pt> for Group<T> {
    type Output = Self;
    fn add(self, rhs: Pt) -> Self::Output {
        Self::new(self.0.into_iter().map(|(o, s)| (o + rhs, s)))
    }
}
impl<T> Sub<Pt> for Group<T> {
    type Output = Self;
    fn sub(self, rhs: Pt) -> Self::Output {
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

impl<T> RemAssign<Pt> for Group<T> {
    fn rem_assign(&mut self, rhs: Pt) {
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
    fn crop(&self, frame: &Pg, crop_type: CropType) -> Result<Vec<Self::Output>> {
        Ok(vec![Group::new(
            self.0
                .iter()
                .flat_map(|(obj, s)| {
                    obj.crop(frame, crop_type)
                        .expect("todo")
                        .into_iter()
                        .map(|o| (o, s.clone()))
                })
                .collect::<Vec<(Obj, T)>>(),
        )])
    }
    fn crop_excluding(&self, _other: &Pg) -> Result<Vec<Self::Output>>
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
    fn annotate(&self, settings: &AnnotationSettings) -> Vec<(Obj, Style)> {
        self.0
            .iter()
            .flat_map(|(o, _)| o.annotate(settings))
            .collect()
    }
}
