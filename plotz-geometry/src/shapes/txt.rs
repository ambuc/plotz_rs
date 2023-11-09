//! A character at a point.
#![allow(missing_docs)]

use crate::{
    bounded::{Bounded, Bounds},
    obj::ObjType2d,
    shapes::pt::Pt,
    *,
};
use anyhow::Result;
use std::ops::*;

#[derive(Debug, PartialEq, Clone)]
/// A character laid out at a point.
pub struct Txt {
    /// the point.
    pub pt: Pt,
    /// the text.
    pub inner: String,
    /// The font size.
    pub font_size: f64,
}

impl Bounded for Txt {
    fn bounds(&self) -> Result<Bounds> {
        self.pt.bounds()
    }
}

crate::ops_defaults_t!(Txt, Pt);

impl Object for Txt {
    fn objtype(&self) -> ObjType2d {
        ObjType2d::Point2d
    }

    fn iter(&self) -> Box<dyn Iterator<Item = &Pt> + '_> {
        Box::new(std::iter::once(&self.pt))
    }

    fn iter_mut(&mut self) -> Box<dyn Iterator<Item = &mut Pt> + '_> {
        Box::new(std::iter::once(&mut self.pt))
    }
}
