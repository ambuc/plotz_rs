//! A 2D polygon which has cavities.
#![allow(missing_docs)]

use crate::{
    bounded::{Bounded, Bounds},
    crop::PointLoc,
    obj::ObjType2d,
    shapes::{point::Point, polygon::Pg},
    Object,
};
use anyhow::{anyhow, Result};
use std::ops::*;

#[derive(Debug, Clone)]
pub struct Pgc {
    pub outer: Pg,
    pub inner: Vec<Pg>,
}

#[allow(non_snake_case)]
pub fn Pgc(a: impl Into<Pg>, b: impl IntoIterator<Item = impl Into<Pg>>) -> Result<Pgc> {
    let inner: Vec<Pg> = b.into_iter().map(|x| x.into()).collect();
    let outer: Pg = a.into();
    for inner_pg in &inner {
        for pt in &inner_pg.pts {
            if outer.contains_pt(pt)? == PointLoc::Outside {
                return Err(anyhow!("pt in inner is outside of outer"));
            }
        }
    }
    Ok(Pgc { outer, inner })
}

impl PartialEq for Pgc {
    fn eq(&self, _: &Self) -> bool {
        unimplemented!("TODO(jbuckland): implement partialeq. we should compare each polygon flexibly _and_ w/o respect for inner ordering");
    }
}

impl Bounded for Pgc {
    fn bounds(&self) -> Result<Bounds> {
        self.outer.bounds()
    }
}

crate::ops_defaults_t!(Pgc, Point);

impl Object for Pgc {
    fn objtype(&self) -> ObjType2d {
        ObjType2d::Polygon2dWithCavities
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
