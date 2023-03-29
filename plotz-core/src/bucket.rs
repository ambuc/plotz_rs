#[derive(Debug, PartialEq, Eq, Copy, Clone, Hash, PartialOrd, Ord)]
pub enum Area {
    Beach,
    Building,
    Business,
    Fun,
    NaturalRock,
    Park,
    Rail,
    Tree,
    Water,
}

#[derive(Debug, PartialEq, Eq, Copy, Clone, Hash, PartialOrd, Ord)]
pub enum Path {
    Highway1,
    Highway2,
    Highway3,
    Highway4,
    Cycleway,
    Pedestrian,
    Rail,
    Subway,
    Boundary,
}

#[derive(Debug, PartialEq, Eq, Copy, Clone, Hash, PartialOrd, Ord)]
pub enum Bucket {
    Frame,
    Area(Area),
    Path(Path),
}
