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
pub enum MultilineAndSegment {
    SegmentInMultiline {
        // Why special case? is the segment the {same, same but reversed, or
        // colinear to} an existing multiline subsegment?
        sc: TwoSegments,
        // and if so, which one?
        index: usize,
    },
}

#[derive(PartialEq, Clone, Debug)]
pub enum General {
    TwoPoints(TwoPoints),
    TwoSegments(TwoSegments),
    MultilineAndSegment(MultilineAndSegment),
}
