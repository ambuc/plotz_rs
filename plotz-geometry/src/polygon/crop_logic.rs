//! Crop logic for polygons.

use derivative::Derivative;

use crate::{isxn::Intersection, point::Pt, segment::Segment};

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct AnnotatedIsxn {
    pub a_idx: usize,
    pub b_idx: usize,
    pub intersection: Intersection,
}

#[derive(Debug, Copy, Clone)]
pub enum WhichPolygon {
    A,
    B,
}
impl WhichPolygon {
    pub fn flip(&self) -> WhichPolygon {
        match self {
            WhichPolygon::A => WhichPolygon::B,
            WhichPolygon::B => WhichPolygon::A,
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub struct OnPolygon {
    pub on_polygon: WhichPolygon,
    pub at_point_index: usize,
}

#[derive(Debug, Copy, Clone)]
pub enum Position {
    OnPolygon(OnPolygon),
    OnIsxn(AnnotatedIsxn),
}

#[derive(Derivative)]
#[derivative(Debug)]
pub struct Cursor<'a> {
    // current position
    pub position: Position,
    pub facing_along: WhichPolygon,
    pub facing_along_segment_idx: usize, // segment index
    // context
    #[derivative(Debug = "ignore")]
    pub a_pts: &'a Vec<(usize, &'a Pt)>,
    #[derivative(Debug = "ignore")]
    pub a_pts_len: &'a usize,
    #[derivative(Debug = "ignore")]
    pub b_pts: &'a Vec<(usize, &'a Pt)>,
    #[derivative(Debug = "ignore")]
    pub b_pts_len: &'a usize,
    #[derivative(Debug = "ignore")]
    pub a_segments: &'a Vec<(usize, Segment)>,
}
impl<'a> Cursor<'a> {
    pub fn pt(&self) -> Pt {
        match &self.position {
            Position::OnPolygon(one_polygon) => match one_polygon.on_polygon {
                WhichPolygon::A => *self.a_pts[one_polygon.at_point_index].1,
                WhichPolygon::B => *self.b_pts[one_polygon.at_point_index].1,
            },
            Position::OnIsxn(AnnotatedIsxn { intersection, .. }) => intersection.pt(),
        }
    }
    pub fn pts_len(&self, on: WhichPolygon) -> usize {
        match on {
            WhichPolygon::A => *self.a_pts_len,
            WhichPolygon::B => *self.b_pts_len,
        }
    }
    pub fn march_to_next_point(&mut self) {
        let v = (match self.position {
            Position::OnPolygon(one_polygon) => one_polygon.at_point_index,
            Position::OnIsxn(AnnotatedIsxn { a_idx, b_idx, .. }) => match self.facing_along {
                WhichPolygon::A => a_idx,
                WhichPolygon::B => b_idx,
            },
        } + 1)
            % self.pts_len(self.facing_along);
        self.position = Position::OnPolygon(OnPolygon {
            on_polygon: self.facing_along,
            at_point_index: v,
        });
        self.facing_along_segment_idx = v;
    }

    pub fn march_to_isxn(&mut self, next_isxn: AnnotatedIsxn, should_flip: bool) {
        let new_position = Position::OnIsxn(next_isxn);
        let new_facing_along = if should_flip {
            self.facing_along.flip()
        } else {
            self.facing_along
        };
        let new_facing_along_segment_idx = match new_facing_along {
            WhichPolygon::B => next_isxn.b_idx,
            WhichPolygon::A => next_isxn.a_idx,
        };
        self.position = new_position;
        self.facing_along = new_facing_along;
        self.facing_along_segment_idx = new_facing_along_segment_idx;
    }
}
