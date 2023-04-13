//! Crop logic for polygons.

use {
    crate::{
        isxn::{Intersection, Which},
        point::Pt,
    },
    derivative::Derivative,
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
    pub facing_along: Which,
    pub facing_along_segment_idx: usize, // segment index
    // context
    #[derivative(Debug = "ignore")]
    pub a_pts: &'a Vec<(usize, &'a Pt)>,
    #[derivative(Debug = "ignore")]
    pub b_pts: &'a Vec<(usize, &'a Pt)>,
}

impl<'a> Cursor<'a> {
    fn pts(&self, which: Which) -> &'a Vec<(usize, &'a Pt)> {
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
            }) => *self.pts(*on_polygon)[*at_point_index].1,
            Position::OnIsxn(AnnotatedIsxn { intersection, .. }) => intersection.pt(),
        }
    }
    pub fn march_to_next_point(&mut self) {
        let v = (match self.position {
            Position::OnPolygon(OnPolygon { at_point_index, .. }) => at_point_index,
            Position::OnIsxn(isxn) => isxn.idx(self.facing_along),
        } + 1)
            % self.pts(self.facing_along).len();
        self.position = Position::OnPolygon(OnPolygon {
            on_polygon: self.facing_along,
            at_point_index: v,
        });
        self.facing_along_segment_idx = v;
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
