//! Crop logic for polygons.

use {
    crate::{
        isxn::{Intersection, Which},
        point::Pt,
    },
    std::fmt::Debug,
};

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct AnnotatedIsxn {
    pub a_idx: usize,
    pub b_idx: usize,
    pub intersection: Intersection,
}

impl AnnotatedIsxn {
    pub fn idx(&self, which: Which) -> usize {
        match which {
            Which::A => self.a_idx,
            Which::B => self.b_idx,
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub struct OnPolygon {
    pub on_polygon: Which,
    pub at_point_index: usize,
}

#[derive(Copy, Clone)]
pub enum Position {
    OnPolygon(OnPolygon),
    OnIsxn(AnnotatedIsxn),
}

impl Debug for Position {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Position::OnPolygon(OnPolygon {
                on_polygon,
                at_point_index,
            }) => {
                write!(
                    f,
                    "on polygon {:?} at point {:?}",
                    on_polygon, at_point_index
                )
            }
            Position::OnIsxn(AnnotatedIsxn {
                a_idx,
                b_idx,
                intersection: Intersection { a_pct, b_pct, .. },
            }) => {
                write!(
                    f,
                    "on isxn {:.2?}% along segment {:?} of A, and {:.2?}% along segment {:?} of B",
                    a_pct.to_f64(),
                    a_idx,
                    b_pct.to_f64(),
                    b_idx
                )
            }
        }
    }
}

pub struct Cursor<'a> {
    // current position
    pub position: Position,
    pub facing_along: Which,
    pub facing_along_segment_idx: usize, // segment index
    // context
    pub a_pts: &'a Vec<Pt>,
    pub b_pts: &'a Vec<Pt>,
}

impl<'a> Debug for Cursor<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "at {:?}, facing {:?} along segment {:?}",
            self.position, self.facing_along, self.facing_along_segment_idx
        )
    }
}

impl<'a> Cursor<'a> {
    fn pts(&self, which: Which) -> &'a Vec<Pt> {
        match which {
            Which::A => self.a_pts,
            Which::B => self.b_pts,
        }
    }

    pub fn pt(&self) -> Pt {
        match &self.position {
            Position::OnPolygon(OnPolygon {
                on_polygon,
                at_point_index,
            }) => self.pts(*on_polygon)[*at_point_index],
            Position::OnIsxn(AnnotatedIsxn { intersection, .. }) => intersection.pt(),
        }
    }

    pub fn march_to_next_point(&mut self) {
        match self.position {
            Position::OnPolygon(OnPolygon { at_point_index, .. }) => {
                let v = (at_point_index + 1) % self.pts(self.facing_along).len();
                self.position = Position::OnPolygon(OnPolygon {
                    on_polygon: self.facing_along,
                    at_point_index: v,
                });
                self.facing_along_segment_idx = v;
            }
            Position::OnIsxn(isxn) => {
                let v = (isxn.idx(self.facing_along) + 1) % self.pts(self.facing_along).len();
                self.position = Position::OnPolygon(OnPolygon {
                    on_polygon: self.facing_along,
                    at_point_index: v,
                });
                self.facing_along_segment_idx = v;
            }
        }
    }

    pub fn march_to_isxn(&mut self, next_isxn: AnnotatedIsxn, should_flip: bool) {
        let new_facing_along = if should_flip {
            self.facing_along.flip()
        } else {
            self.facing_along
        };

        self.position = Position::OnIsxn(next_isxn);
        self.facing_along = new_facing_along;
        self.facing_along_segment_idx = next_isxn.idx(new_facing_along);
    }
}
