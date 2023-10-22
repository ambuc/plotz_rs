//! A group of objects.
//!

use crate::{obj3::Obj3, shapes::pt3::Pt3};
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
