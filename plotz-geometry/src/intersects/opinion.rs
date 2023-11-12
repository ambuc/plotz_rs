use crate::{shapes::point::Point, utils::Percent};

#[derive(PartialEq, Clone, Debug)]
pub enum Opinion {
    Point,
    Segment {
        at_point: Point,
        percent_along: Percent,
    },
    Multiline(
        // A list of possible collisions --
        // The index of the segment, and
        // The segment collision details themselves.
        Vec<(usize, Opinion)>,
    ),
    Polygon(),
}
