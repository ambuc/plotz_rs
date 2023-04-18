//! Crop logic for polygons.

use crate::{crop::PointLoc, isxn::Pct};

use super::Polygon;

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
                    "on isxn {:.0?}% along segment {:?} of A, and {:.0?}% along segment {:?} of B",
                    a_pct.to_f64() * 100.0,
                    a_idx,
                    b_pct.to_f64() * 100.0,
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
    // context
    pub a_pts: &'a Vec<Pt>,
    pub b_pts: &'a Vec<Pt>,

    pub a: &'a Polygon,
}

impl<'a> Debug for Cursor<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "[{:?}] at {:?}, facing along {:?}",
            self.facing_along, self.position, self.facing_along
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

    pub fn idx(&self) -> usize {
        match &self.position {
            Position::OnPolygon(OnPolygon { at_point_index, .. }) => *at_point_index,
            Position::OnIsxn(AnnotatedIsxn { a_idx, b_idx, .. }) => match self.facing_along {
                Which::A => *a_idx,
                Which::B => *b_idx,
            },
        }
    }

    pub fn march_to_next_point(&mut self) {
        match self.position {
            Position::OnPolygon(OnPolygon { at_point_index, .. }) => {
                let next_isxn_idx = (at_point_index + 1) % self.pts(self.facing_along).len();
                self.position = Position::OnPolygon(OnPolygon {
                    on_polygon: self.facing_along,
                    at_point_index: next_isxn_idx,
                });
            }
            Position::OnIsxn(isxn) => {
                let prev_pt = self.pt();
                let next_isxn_idx =
                    (isxn.idx(self.facing_along) + 1) % self.pts(self.facing_along).len();
                self.position = Position::OnPolygon(OnPolygon {
                    on_polygon: self.facing_along,
                    at_point_index: next_isxn_idx,
                });
                let new_pt = self.pt();
                if prev_pt == new_pt {
                    self.march_to_next_point();
                }
            }
        }
    }

    pub fn march_to_isxn(&mut self, next_isxn: AnnotatedIsxn, should_flip: bool) {
        // if we are given an intersection which is at a point, we should replace it with being at a point instead. (and flip);

        let new_facing_along = if should_flip {
            self.facing_along.flip()
        } else {
            self.facing_along
        };

        self.position = Position::OnIsxn(next_isxn);
        self.facing_along = new_facing_along;
    }
}
