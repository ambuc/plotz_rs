//! A group of objects.
//!

use anyhow::Result;

use crate::{
    bounded3::{Bounded3, Bounds3, Bounds3Collector},
    obj3::Obj3,
    shapes::{pt3::Pt3, ry3::Ry3},
    Rotatable, RotatableBounds,
};
use std::ops::*;

#[derive(Debug, Clone)]
pub struct Group3<T>(Vec<(Obj3, T)>);

impl<T: 'static> Group3<T> {
    /// Creates a new Group3.
    pub fn new(objs: impl IntoIterator<Item = (Obj3, T)>) -> Group3<T> {
        Group3(objs.into_iter().collect::<Vec<_>>())
    }
    /// Returns a boxed iterator of immutable (Obj3, T), the members of this
    /// group.
    pub fn iter_objects(&self) -> Box<dyn Iterator<Item = &(Obj3, T)> + '_> {
        Box::new(self.0.iter())
    }
    pub fn into_iter_objects(self) -> Box<dyn Iterator<Item = (Obj3, T)>> {
        Box::new(self.0.into_iter())
    }
}

impl<T: 'static> Add<Pt3> for Group3<T> {
    type Output = Self;
    fn add(self, rhs: Pt3) -> Self::Output {
        Self::new(self.0.into_iter().map(|(o, s)| (o + rhs, s)))
    }
}
impl<T> AddAssign<Pt3> for Group3<T> {
    fn add_assign(&mut self, rhs: Pt3) {
        self.0.iter_mut().for_each(|(o, _)| {
            *o += rhs;
        });
    }
}

impl<T: 'static> Rotatable for Group3<T>
where
    T: Clone,
{
    fn rotate(&self, by: f64, about: Ry3) -> Result<Self> {
        let mut v: Vec<(Obj3, T)> = vec![];
        for (obj3, style) in self.iter_objects() {
            //
            v.push((obj3.rotate(by, about)?, (*style).clone()));
        }
        Ok(Group3::<T>(v))
    }
}

impl<T> RotatableBounds for Group3<T>
where
    T: Clone + 'static,
{
    //
}

impl<T: 'static> Bounded3 for Group3<T> {
    fn bounds3(&self) -> Result<Bounds3> {
        let mut bc = Bounds3Collector::default();
        for (i, _) in self.0.iter() {
            bc.incorporate(&i.bounds3()?)?;
        }
        bc.bounds3()
    }
}
