#[derive(PartialEq, Clone, Debug)]
pub enum PointsSC {
    Same,
}

#[derive(PartialEq, Clone, Debug)]
pub enum SegmentsSC {
    Same,
    SameButReversed,
    Colinear,
}

#[derive(PartialEq, Clone, Debug)]
pub enum IsxnSC {
    Points(PointsSC),
    Segments(SegmentsSC),
}
