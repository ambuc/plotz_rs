#[derive(PartialEq, Clone, Debug)]
pub enum TwoPoints {
    Same,
}

#[derive(PartialEq, Clone, Debug)]
pub enum TwoSegments {
    Same,
    SameButReversed,
    Colinear,
}

#[derive(PartialEq, Clone, Debug)]
pub enum General {
    TwoPoints(TwoPoints),
    TwoSegments(TwoSegments),
}
