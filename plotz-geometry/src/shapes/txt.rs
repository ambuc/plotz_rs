//! A character at a point.
#![allow(missing_docs)]

use crate::{
    bounded::{Bounded, Bounds},
    obj::ObjType,
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

impl Txt {
    /// Iterator.
    pub fn iter(&self) -> impl Iterator<Item = &Pt> {
        std::iter::once(&self.pt)
    }

    /// Mutable iterator.
    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut Pt> {
        std::iter::once(&mut self.pt)
    }

    pub fn objtype(&self) -> ObjType {
        ObjType::Point
    }
}

impl Bounded for Txt {
    fn bounds(&self) -> Result<Bounds> {
        self.pt.bounds()
    }
}

crate::ops_defaults_t!(Txt, Pt);

impl Nullable for Txt {
    fn is_empty(&self) -> bool {
        false
    }
}
