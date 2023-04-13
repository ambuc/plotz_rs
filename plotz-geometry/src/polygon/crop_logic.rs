//! Crop logic for polygons.

use derivative::Derivative;
use either::Either;

use crate::{isxn::Intersection, point::Pt, segment::Segment};

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct AnnotatedIsxn {
    pub a_idx: usize,
    pub b_idx: usize,
    pub intersection: Intersection,
}
impl AnnotatedIsxn {
    pub fn pt(&self) -> Pt {
        self.intersection.pt()
    }
}

#[derive(Debug, Copy, Clone)]
pub enum On {
    OnA,
    OnB,
}
impl On {
    pub fn flip(&self) -> On {
        match self {
            On::OnA => On::OnB,
            On::OnB => On::OnA,
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub struct OnePolygon {
    pub on_polygon: On,
    pub at_point_index: usize,
}

#[derive(Derivative)]
#[derivative(Debug)]
pub struct Cursor<'a> {
    // current position
    pub position: Either<OnePolygon, AnnotatedIsxn>,
    pub facing_along: On,
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
            Either::Left(one_polygon) => match one_polygon.on_polygon {
                On::OnA => *self.a_pts[one_polygon.at_point_index].1,
                On::OnB => *self.b_pts[one_polygon.at_point_index].1,
            },
            Either::Right(isxn) => isxn.pt(),
        }
    }
    pub fn pts_len(&self, on: On) -> usize {
        match on {
            On::OnA => *self.a_pts_len,
            On::OnB => *self.b_pts_len,
        }
    }
    pub fn march_to_next_point(&mut self) {
        let v = (match self.position {
            Either::Left(one_polygon) => one_polygon.at_point_index,
            Either::Right(isxn) => match self.facing_along {
                On::OnA => isxn.a_idx,
                On::OnB => isxn.b_idx,
            },
        } + 1)
            % self.pts_len(self.facing_along);
        self.position = Either::Left(OnePolygon {
            on_polygon: self.facing_along,
            at_point_index: v,
        });
        self.facing_along_segment_idx = v;
    }

    pub fn march_to_isxn(&mut self, next_isxn: AnnotatedIsxn, should_flip: bool) {
        let new_position: Either<_, AnnotatedIsxn> = Either::Right(next_isxn);
        let new_facing_along = if should_flip {
            self.facing_along.flip()
        } else {
            self.facing_along
        };
        let new_facing_along_segment_idx = match new_facing_along {
            On::OnB => next_isxn.b_idx,
            On::OnA => next_isxn.a_idx,
        };
        self.position = new_position;
        self.facing_along = new_facing_along;
        self.facing_along_segment_idx = new_facing_along_segment_idx;
    }
}
