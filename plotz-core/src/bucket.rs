use plotz_color::ColorRGB;

#[derive(Debug, PartialEq, Eq, Copy, Clone, Hash, PartialOrd, Ord)]
pub enum Area {
    Beach,
    Building,
    Land,
    Fun,
    NaturalRock,
    Park,
    Parking,
    Rail,
    Tree,
    Water,
}

#[derive(Debug, PartialEq, Eq, Copy, Clone, Hash, PartialOrd, Ord)]
pub enum Path {
    Barrier,
    Boundary,
    Bridge,
    Bus,
    Cable,
    Cycleway,
    Highway(Highway),
    Pedestrian,
    Rail,
    Subway(Subway),
}

#[derive(Debug, PartialEq, Eq, Copy, Clone, Hash, PartialOrd, Ord)]
pub enum Highway {
    Elevator,
    MotorwayLink,
    Path,
    Platform,
    Primary,
    PrimaryLink,
    Road,
    RoadMarking,
    Secondary,
    SecondaryLink,
    Service,
    Tertiary,
    TertiaryLink,
    Track,
    Unclassified,
}

#[derive(Debug, PartialEq, Eq, Copy, Clone, Hash, PartialOrd, Ord)]
pub enum Subway {
    Other,
    _ACE,
    _BDFM,
    _G,
    _L,
    _JZ,
    _NQRW,
    _123,
    _456,
    _7,
    _T,
    _S,
}

#[derive(Debug, PartialEq, Eq, Copy, Clone, Hash, PartialOrd, Ord)]
pub enum Bucket {
    Frame,
    Area(Area),
    Path(Path),
    Color(ColorRGB),
}
