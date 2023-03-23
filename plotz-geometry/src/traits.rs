//! Traits
#![allow(dead_code)]
#![allow(unused)]
#![allow(missing_docs)]

use crate::point::Pt;

pub trait YieldPoints {
    fn yield_pts(&self) -> Box<dyn Iterator<Item = &Pt> + '_>;
}

pub trait YieldPointsMut {
    fn yield_pts_mut(&mut self) -> Box<dyn Iterator<Item = &mut Pt> + '_>;
}
