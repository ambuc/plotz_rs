use crate::{shapes::point::Point, utils::Percent};

#[derive(PartialEq, Clone, Debug)]
pub enum Opinion {
    Point,
    Segment(
        // The point at which it occurred.
        Point,
        // The percentage of the way along this segment which it occurred.
        Percent,
    ),
    Multiline(
        // A list of possible collisions --
        // The index of the segment, and
        // The segment collision details themselves.
        Vec<(usize, Opinion)>,
    ),
    Polygon(),
}
