//! A 2D polygon which has cavities.
#![allow(missing_docs)]

use crate::{
    bounded::{Bounded, Bounds},
    crop::PointLocation,
    obj2::ObjType2d,
    shapes::{point::Point, polygon::Polygon},
    Object,
};
use anyhow::{anyhow, Result};
use std::ops::*;

#[derive(Debug, Clone)]
pub struct PolygonWithCavities {
    pub outer: Polygon,
    pub inner: Vec<Polygon>,
}

#[allow(non_snake_case)]
pub fn PolygonWithCavities(
    a: impl Into<Polygon>,
    b: impl IntoIterator<Item = impl Into<Polygon>>,
) -> Result<PolygonWithCavities> {
    let inner: Vec<Polygon> = b.into_iter().map(|x| x.into()).collect();
    let outer: Polygon = a.into();
    for inner_pg in &inner {
        for pt in &inner_pg.pts {
            if outer.contains_pt(pt)? == PointLocation::Outside {
                return Err(anyhow!("pt in inner is outside of outer"));
            }
        }
    }
    Ok(PolygonWithCavities { outer, inner })
}

impl PartialEq for PolygonWithCavities {
    fn eq(&self, _: &Self) -> bool {
        unimplemented!("TODO(jbuckland): implement partialeq. we should compare each polygon flexibly _and_ w/o respect for inner ordering");
    }
}

impl Bounded for PolygonWithCavities {
    fn bounds(&self) -> Result<Bounds> {
        self.outer.bounds()
    }
}

crate::ops_defaults_t!(PolygonWithCavities, Point);

impl Object for PolygonWithCavities {
    fn objtype(&self) -> ObjType2d {
        ObjType2d::PolygonWithCavities2d
    }

    fn iter(&self) -> Box<dyn Iterator<Item = &Point> + '_> {
        Box::new(
            self.outer
                .iter()
                .chain(self.inner.iter().flat_map(|i| i.iter())),
        )
    }

    fn iter_mut(&mut self) -> Box<dyn Iterator<Item = &mut Point> + '_> {
        Box::new(
            self.outer
                .iter_mut()
                .chain(self.inner.iter_mut().flat_map(|i| i.iter_mut())),
        )
    }
}
