//! A 2D polygon which has cavities.
#![allow(missing_docs)]

use crate::{
    bounded::{Bounded, Bounds},
    crop::PointLoc,
    obj::{Obj, ObjType},
    shapes::{pg::Pg, pt::Pt},
    style::Style,
    Annotatable, AnnotationSettings, Nullable,
};
use anyhow::{anyhow, Result};
use std::ops::*;

use super::txt::Txt;

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

impl Pgc {
    /// Iterator.
    pub fn iter(&self) -> impl Iterator<Item = &Pt> {
        self.outer
            .iter()
            .chain(self.inner.iter().flat_map(|i| i.iter()))
    }

    /// Mutable iterator.
    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut Pt> {
        self.outer
            .iter_mut()
            .chain(self.inner.iter_mut().flat_map(|i| i.iter_mut()))
    }

    pub fn objtype(&self) -> ObjType {
        ObjType::PolygonWithCavities
    }
}

impl PartialEq for Pgc {
    fn eq(&self, _: &Self) -> bool {
        unimplemented!("TODO(jbuckland): implement partialeq. we should compare each polygon flexibly _and_ w/o respect for inner ordering");
    }
}

impl Nullable for Pgc {
    fn is_empty(&self) -> bool {
        self.outer.is_empty() || self.inner.is_empty()
    }
}

impl Bounded for Pgc {
    fn bounds(&self) -> Result<Bounds> {
        self.outer.bounds()
    }
}

crate::ops_defaults_t!(Pgc, Pt);

impl Annotatable for Pgc {
    fn annotate(&self, settings: &AnnotationSettings) -> Vec<(Obj, Style)> {
        let mut a = vec![];

        let AnnotationSettings {
            font_size,
            precision,
        } = settings;
        for (_idx, pt) in self.iter().enumerate() {
            let x = format!("{:.1$}", pt.x, precision);
            let y = format!("{:.1$}", pt.y, precision);
            a.push((
                Txt {
                    pt: *pt,
                    inner: format!("({}, {})", x, y),
                    font_size: *font_size,
                }
                .into(),
                Style::default(),
            ));
        }

        a
    }
}
