#[derive(Debug, PartialEq, Eq, Copy, Clone, Hash)]
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

#[derive(Debug, PartialEq, Eq, Copy, Clone, Hash)]
pub enum Path {
    Highway1,
    Highway2,
    Highway3,
    Highway4,
    Cycleway,
    Pedestrian,
    Rail,
    Boundary,
}

#[derive(Debug, PartialEq, Eq, Copy, Clone, Hash)]
pub enum Bucket {
    Area(Area),
    Path(Path),
}
