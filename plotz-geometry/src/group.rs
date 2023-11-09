//! A group of objects.
#![allow(missing_docs)]

use crate::{
    bounded::{streaming_bbox, Bounded, Bounds},
    crop::{CropType, Croppable},
    obj2::{Obj2, ObjType2d},
    shapes::{point::Point, polygon::Polygon},
    *,
};
use anyhow::Result;
use itertools::Itertools;
use std::ops::*;

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
}

impl<T> Bounded for Group<T> {
    fn bounds(&self) -> Result<Bounds> {
        streaming_bbox(self.iter())
    }
}

crate::ops_generic_defaults_t!(Group, Point);

impl<T> Croppable for Group<T>
where
    T: Clone,
{
    type Output = Group<T>;
    fn crop(&self, frame: &Polygon, crop_type: CropType) -> Result<Vec<Self::Output>> {
        Ok(vec![Group::new(
            self.0
                .iter()
                .map(|(obj, s)| {
                    Ok(obj
                        .crop(frame, crop_type)?
                        .into_iter()
                        .map(|o| (o, s.clone())))
                })
                .flatten_ok()
                .collect::<Result<Vec<(Obj2, T)>>>()?,
        )])
    }
    fn crop_excluding(&self, _other: &Polygon) -> Result<Vec<Self::Output>>
    where
        Self: Sized,
    {
        unimplemented!("TODO");
    }
}

impl<T> Object for Group<T> {
    fn objtype(&self) -> ObjType2d {
        ObjType2d::Group2d
    }

    fn iter(&self) -> Box<dyn Iterator<Item = &Point> + '_> {
        Box::new(self.0.iter().flat_map(|(x, _)| x.iter()))
    }

    fn iter_mut(&mut self) -> Box<dyn Iterator<Item = &mut Point> + '_> {
        Box::new(self.0.iter_mut().flat_map(|(x, _)| x.iter_mut()))
    }
}
