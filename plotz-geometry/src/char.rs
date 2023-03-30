//！ A character at a point.
use crate::{
    bounded::Bounded,
    point::Pt,
    traits::{Mutable, YieldPoints, YieldPointsMut},
};

#[derive(Debug, PartialEq, Clone)]
/// A character laid out at a point.
pub struct Char {
    /// the point.
    pub pt: Pt,
    /// the character.
    pub chr: char,
}

impl Bounded for Char {
    fn right_bound(&self) -> f64 {
        self.pt.right_bound()
    }
    fn left_bound(&self) -> f64 {
        self.pt.left_bound()
    }
    fn bottom_bound(&self) -> f64 {
        self.pt.bottom_bound()
    }
    fn top_bound(&self) -> f64 {
        self.pt.top_bound()
    }
}

impl YieldPoints for Char {
    fn yield_pts(&self) -> Option<Box<dyn Iterator<Item = &Pt> + '_>> {
        Some(Box::new(std::iter::once(&self.pt)))
    }
}

impl YieldPointsMut for Char {
    fn yield_pts_mut(&mut self) -> Option<Box<dyn Iterator<Item = &mut Pt> + '_>> {
        Some(Box::new(std::iter::once(&mut self.pt)))
    }
}

impl Mutable for Char {}
