//！ A character at a point.
use plotz_geometry::point::Pt;
use plotz_geometry::bounded::Bounded;

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

