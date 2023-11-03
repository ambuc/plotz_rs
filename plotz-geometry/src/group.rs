//! A group of objects.
#![allow(missing_docs)]

use crate::{
    bounded::{Bounded, Bounds, BoundsCollector},
    crop::{CropType, Croppable},
    obj::{Obj, ObjType},
    shapes::{pg::Pg, pt::Pt},
    style::Style,
    *,
};
use anyhow::Result;
use itertools::Itertools;
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
}

impl<T> Bounded for Group<T> {
    fn bounds(&self) -> Result<Bounds> {
        let mut bc = BoundsCollector::default();
        for pt in self.iter() {
            bc.incorporate(pt)?;
        }
        bc.bounds()
    }
}

crate::ops_generic_defaults_t!(Group, Pt);

impl<T> Croppable for Group<T>
where
    T: Clone,
{
    type Output = Group<T>;
    fn crop(&self, frame: &Pg, crop_type: CropType) -> Result<Vec<Self::Output>> {
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
                .collect::<Result<Vec<(Obj, T)>>>()?,
        )])
    }
    fn crop_excluding(&self, _other: &Pg) -> Result<Vec<Self::Output>>
    where
        Self: Sized,
    {
        unimplemented!("TODO");
    }
}

impl<T> Object for Group<T> {
    fn annotate(&self, settings: &AnnotationSettings) -> Vec<(Obj, Style)> {
        self.0
            .iter()
            .flat_map(|(o, _)| o.annotate(settings))
            .collect()
    }

    fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    fn objtype(&self) -> ObjType {
        ObjType::Group
    }

    fn iter(&self) -> Box<dyn Iterator<Item = &Pt> + '_> {
        Box::new(self.0.iter().flat_map(|(x, _)| x.iter()))
    }

    fn iter_mut(&mut self) -> Box<dyn Iterator<Item = &mut Pt> + '_> {
        Box::new(self.0.iter_mut().flat_map(|(x, _)| x.iter_mut()))
    }
}
