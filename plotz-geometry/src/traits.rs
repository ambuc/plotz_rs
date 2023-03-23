//! Traits
#![allow(dead_code)]
#![allow(unused)]
#![allow(missing_docs)]

use crate::point::Pt;

pub trait YieldPoints {
    fn yield_pts(&self) -> Option<Box<dyn Iterator<Item = &Pt> + '_>>;
}

pub trait YieldPointsMut {
    fn yield_pts_mut(&mut self) -> Option<Box<dyn Iterator<Item = &mut Pt> + '_>>;
}

pub trait Mutable: YieldPointsMut {
    fn mutate(&mut self, f: impl Fn(&mut Pt)) -> bool {
        if let Some(yp) = self.yield_pts_mut() {
            yp.for_each(|pt| f(pt));
            return true;
        }
        false
    }
}
