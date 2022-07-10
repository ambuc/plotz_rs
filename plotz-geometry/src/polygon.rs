use {
    crate::{point::Pt, segment::Segment},
    float_cmp::approx_eq,
    itertools::zip,
    num::Float,
    std::ops::{AddAssign, Sub},
    thiserror,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PolygonKind {
    Open,
    Closed,
}

/// A multiline is a list of points rendered with connecting line segments.
/// If constructed with PolygonKind::Open, this is a multiline (unshaded).
/// If constructed with PolygonKind::Closed, this is a closed, shaded polygon.
#[derive(Debug, Clone)]
pub struct Polygon<T> {
    pub pts: Vec<Pt<T>>,
    pub kind: PolygonKind,
}

#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum MultilineConstructorError {
    #[error("one or fewer points")]
    OneOrFewerPoints,
}

/// Constructor for multilines. Multilines must have at least one line, so they
/// must have two or more points. Constructing a multiline from one or fewer
/// points will result in a MultilineConstructorError.
#[allow(non_snake_case)]
pub fn Multiline<T>(
    a: impl IntoIterator<Item = Pt<T>>,
) -> Result<Polygon<T>, MultilineConstructorError> {
    let pts: Vec<Pt<T>> = a.into_iter().collect();
    if pts.len() <= 1 {
        return Err(MultilineConstructorError::OneOrFewerPoints);
    }
    Ok(Polygon {
        pts,
        kind: PolygonKind::Open,
    })
}

#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum PolygonConstructorError {
    #[error("two or fewer points")]
    TwoOrFewerPoints,
}

/// Constructor for polygons. Polygons must have inner area, so they must have
/// three or more points. Constructing a polygon from two or fewer points will
/// result in a PolygonConstructorErrorip
#[allow(non_snake_case)]
pub fn Polygon<T>(
    a: impl IntoIterator<Item = Pt<T>>,
) -> Result<Polygon<T>, PolygonConstructorError> {
    let pts: Vec<Pt<T>> = a.into_iter().collect();
    if pts.len() <= 2 {
        return Err(PolygonConstructorError::TwoOrFewerPoints);
    }
    Ok(Polygon {
        pts,
        kind: PolygonKind::Closed,
    })
}

#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum ContainsPointError {
    #[error("polygon is open, not closed; invalid to ask if it contains a point.")]
    PolygonIsOpen,
}

#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum CropToPolygonError {}

#[derive(Debug, PartialEq, Eq)]
pub enum ContainsResult {
    Outside,
    Inside,
    AlongBoundary,
}

impl<T> Polygon<T> {
    /// Returns the segments of a polygon, one at a time.
    ///
    /// If this is an open polygon, we return only the line segments without the
    /// final closure.
    ///
    /// If this is a closed polygon, we also generate the final closure.
    ///
    /// See test_multiline_to_segments() and test_polygon_to_segments() for
    /// examples.
    pub fn to_segments(&self) -> Vec<Segment<T>>
    where
        T: Copy,
    {
        match self.kind {
            PolygonKind::Open => zip(self.pts.iter(), self.pts.iter().skip(1))
                .map(|(x, y)| Segment(*x, *y))
                .collect(),
            PolygonKind::Closed => zip(self.pts.iter(), self.pts.iter().cycle().skip(1))
                .map(|(x, y)| Segment(*x, *y))
                .collect(),
        }
    }
    /// Returns true if any line segment from this polygon intersects any line
    /// segment from the other polygon.
    pub fn intersects(&self, other: &Polygon<T>) -> bool
    where
        T: Copy + Float + PartialOrd,
        Pt<T>: PartialEq,
    {
        for l1 in self.to_segments() {
            for l2 in other.to_segments() {
                if l1.intersects(&l2) {
                    return true;
                }
            }
        }
        false
    }

    /// Calculates whether a point is within, without, or along a closed polygon
    /// using the https://en.wikipedia.org/wiki/Winding_number method.
    pub fn contains_pt(&self, other: &Pt<T>) -> Result<ContainsResult, ContainsPointError>
    where
        T: Float + AddAssign + float_cmp::ApproxEq + std::fmt::Debug,
        Pt<T>: Sub<Output = Pt<T>> + AddAssign<Pt<T>>,
    {
        // If |self| is open, error out.
        if self.kind == PolygonKind::Open {
            return Err(ContainsPointError::PolygonIsOpen);
        }

        // If |other| is along any boundary segments, return AlongBoundary.
        if self
            .to_segments()
            .iter()
            .any(|seg| seg.line_segment_contains_pt(other))
        {
            return Ok(ContainsResult::AlongBoundary);
        }

        let mut theta: T = T::zero();
        for (i, j) in zip(self.pts.iter(), self.pts.iter().cycle().skip(1)) {
            theta += _abp(other, i, j)
        }

        Ok(match approx_eq!(T, theta, T::zero()) {
            true => ContainsResult::Outside,
            false => ContainsResult::Inside,
        })
    }

    pub fn _crop_to_polygon(&self, other: &Polygon<T>) -> Result<Polygon<T>, CropToPolygonError>
    where
        T: Copy,
    {
        let _segments_self: Vec<Segment<T>> = self.to_segments();
        let _segments_other: Vec<Segment<T>> = other.to_segments();
        unimplemented!("todo");
    }
}

// Angle between points. Projects OI onto OJ and finds the angle IOJ.
fn _abp<T>(o: &Pt<T>, i: &Pt<T>, j: &Pt<T>) -> T
where
    T: Float + std::fmt::Debug,
{
    let a: Pt<T> = *i - *o;
    let b: Pt<T> = *j - *o;
    T::atan2(
        /*det=*/ a.x * b.y - a.y * b.x,
        /*dot=*/ a.x * b.x + a.y * b.y,
    )
}

#[cfg(test)]
mod tests {
    use float_eq::assert_float_eq;

    use super::*;

    #[test]
    fn test_multiline_to_segments() {
        assert_eq!(
            Multiline([Pt(0, 0)]).unwrap_err(),
            MultilineConstructorError::OneOrFewerPoints
        );
        assert_eq!(
            Multiline([Pt(0, 0), Pt(0, 1)]).unwrap().to_segments(),
            [Segment(Pt(0, 0), Pt(0, 1)),]
        );
        assert_eq!(
            Multiline([Pt(0, 0), Pt(0, 1), Pt(0, 2)])
                .unwrap()
                .to_segments(),
            [Segment(Pt(0, 0), Pt(0, 1)), Segment(Pt(0, 1), Pt(0, 2)),]
        );
        assert_eq!(
            Multiline([Pt(0, 0), Pt(0, 1), Pt(0, 2), Pt(0, 3)])
                .unwrap()
                .to_segments(),
            [
                Segment(Pt(0, 0), Pt(0, 1)),
                Segment(Pt(0, 1), Pt(0, 2)),
                Segment(Pt(0, 2), Pt(0, 3)),
            ]
        );
    }

    #[test]
    fn test_polygon_to_segments() {
        assert_eq!(
            Polygon([Pt(0, 0), Pt(0, 1)]).unwrap_err(),
            PolygonConstructorError::TwoOrFewerPoints,
        );

        assert_eq!(
            Polygon([Pt(0, 0), Pt(0, 1), Pt(0, 2)])
                .unwrap()
                .to_segments(),
            [
                Segment(Pt(0, 0), Pt(0, 1)),
                Segment(Pt(0, 1), Pt(0, 2)),
                Segment(Pt(0, 2), Pt(0, 0)),
            ]
        );

        assert_eq!(
            Polygon([Pt(0, 0), Pt(0, 1), Pt(0, 2), Pt(0, 3)])
                .unwrap()
                .to_segments(),
            [
                Segment(Pt(0, 0), Pt(0, 1)),
                Segment(Pt(0, 1), Pt(0, 2)),
                Segment(Pt(0, 2), Pt(0, 3)),
                Segment(Pt(0, 3), Pt(0, 0)),
            ]
        );
    }

    #[test]
    fn test_intersects() {
        //   ^
        //   |
        //   A  B  C
        //   |
        //   D  E  F
        //   |
        // --G--H--I->
        //   |
        let a = Pt(0.0, 2.0);
        let b = Pt(1.0, 2.0);
        let c = Pt(2.0, 2.0);
        let d = Pt(0.0, 1.0);
        let e = Pt(1.0, 1.0);
        let f = Pt(2.0, 1.0);
        let g = Pt(0.0, 0.0);
        let h = Pt(1.0, 0.0);
        let i = Pt(2.0, 0.0);

        // Positive area intersection.
        assert!(Polygon([a, c, i, g])
            .unwrap()
            .intersects(&Polygon([b, f, h, d]).unwrap()));
        assert!(Polygon([a, c, i, g])
            .unwrap()
            .intersects(&Polygon([a, b, e, d]).unwrap()));
        assert!(Polygon([a, c, i, g])
            .unwrap()
            .intersects(&Polygon([e, f, i, h]).unwrap()));

        // Shares a corner.
        assert!(Polygon([a, b, e, d])
            .unwrap()
            .intersects(&Polygon([e, f, i, h]).unwrap()));
        assert!(Polygon([a, b, e, d])
            .unwrap()
            .intersects(&Polygon([b, c, f, e]).unwrap()));

        // No intersection.
        assert!(!Polygon([a, b, d])
            .unwrap()
            .intersects(&Polygon([e, f, h]).unwrap()));
        assert!(!Polygon([a, b, d])
            .unwrap()
            .intersects(&Polygon([f, h, i]).unwrap()));
    }

    #[test]
    fn test_angle_between_points() {
        use std::f64::consts::PI;
        //   ^
        //   |
        //   A  B  C
        //   |
        //   D  E  F
        //   |
        // --G--H--I->
        //   |
        let a = Pt(0.0, 2.0);
        let b = Pt(1.0, 2.0);
        let c = Pt(2.0, 2.0);
        let d = Pt(0.0, 1.0);
        let e = Pt(1.0, 1.0);
        let f = Pt(2.0, 1.0);
        let g = Pt(0.0, 0.0);
        let h = Pt(1.0, 0.0);
        let i = Pt(2.0, 0.0);

        // circle around E. (quadrants 1, 2, 3, 4)
        assert_float_eq!(_abp(&e, &f, &b), PI / 2.0, ulps <= 10);
        assert_float_eq!(_abp(&e, &f, &d), PI, ulps <= 10);
        assert_float_eq!(_abp(&e, &f, &h), -1.0 * PI / 2.0, ulps <= 10);
        assert_float_eq!(_abp(&e, &f, &f), 0.0, ulps <= 10);

        // circle around E, inverse. (quadrants 1, 2, 3, 4)
        assert_float_eq!(_abp(&e, &f, &h), -1.0 * PI / 2.0, ulps <= 10);
        assert_float_eq!(_abp(&e, &f, &d), PI, ulps <= 10);
        assert_float_eq!(_abp(&e, &f, &b), PI / 2.0, ulps <= 10);
        assert_float_eq!(_abp(&e, &f, &f), 0.0, ulps <= 10);

        // circle around G. (quadrant 1)
        assert_float_eq!(_abp(&g, &i, &i), 0.0, ulps <= 10);
        assert_float_eq!(_abp(&g, &i, &h), 0.0, ulps <= 10);
        assert_float_eq!(_abp(&g, &i, &f), 0.5.atan(), ulps <= 10);
        assert_float_eq!(_abp(&g, &i, &e), 1.0.atan(), ulps <= 10);
        assert_float_eq!(_abp(&g, &i, &c), 1.0.atan(), ulps <= 10);
        assert_float_eq!(_abp(&g, &i, &b), 2.0.atan(), ulps <= 10);
        assert_float_eq!(_abp(&g, &i, &d), PI / 2.0, ulps <= 10);
        assert_float_eq!(_abp(&g, &i, &a), PI / 2.0, ulps <= 10);

        // circle around H (quadrants 1, 2)
        assert_float_eq!(_abp(&h, &i, &i), 0.0, ulps <= 10);
        assert_float_eq!(_abp(&h, &i, &b), PI / 2.0, ulps <= 10);
        assert_float_eq!(_abp(&h, &i, &a), PI / 2.0 + 0.5.atan(), ulps <= 10);
        assert_float_eq!(_abp(&h, &i, &d), PI / 2.0 + 1.0.atan(), ulps <= 10);
        assert_float_eq!(_abp(&h, &i, &g), PI, ulps <= 10);

        // circle around B (quadrants 3, 4)
        assert_float_eq!(_abp(&b, &c, &c), 0.0, ulps <= 10);
        assert_float_eq!(_abp(&b, &c, &f), -1.0.atan(), ulps <= 10);
        assert_float_eq!(_abp(&b, &c, &i), -2.0.atan(), ulps <= 10);
        assert_float_eq!(_abp(&b, &c, &e), -1.0 * PI / 2.0, ulps <= 10);
        assert_float_eq!(_abp(&b, &c, &h), -1.0 * PI / 2.0, ulps <= 10);
        assert_float_eq!(_abp(&b, &c, &g), -1.0 * PI / 2.0 - 0.5.atan(), ulps <= 10);
        assert_float_eq!(_abp(&b, &c, &d), -3.0 * PI / 4.0, ulps <= 10);
    }

    #[test]
    fn test_contains_pt() {
        //   ^
        //   |
        //   A  B  C
        //   |
        //   D  E  F
        //   |
        // --G--H--I->
        //   |
        let a = Pt(0.0, 2.0);
        let b = Pt(1.0, 2.0);
        let c = Pt(2.0, 2.0);
        let d = Pt(0.0, 1.0);
        let e = Pt(1.0, 1.0);
        let f = Pt(2.0, 1.0);
        let g = Pt(0.0, 0.0);
        let h = Pt(1.0, 0.0);
        let i = Pt(2.0, 0.0);

        // frame [a,c,i,g] should contain a, b, c, d, e, f, g, h, and i.
        let frame1 = Polygon([a, c, i, g]).unwrap();
        {
            let p = e;
            assert_eq!(frame1.contains_pt(&p).unwrap(), ContainsResult::Inside);
        }
        for p in [a, b, c, d, f, g, h, i] {
            assert_eq!(
                frame1.contains_pt(&p).unwrap(),
                ContainsResult::AlongBoundary,
            );
        }

        // frame [a,b,e,d] should contain a, b, d, e...
        let frame2 = Polygon([a, b, e, d]).unwrap();
        for p in [a, b, d, e] {
            assert_eq!(
                frame2.contains_pt(&p).unwrap(),
                ContainsResult::AlongBoundary
            );
        }
        for p in [c, f, i, h, g] {
            // broken
            assert_eq!(frame2.contains_pt(&p).unwrap(), ContainsResult::Outside,);
        }

        let frame3 = Polygon([b, f, h, d]).unwrap();
        for p in [b, f, d, h] {
            assert_eq!(
                frame3.contains_pt(&p).unwrap(),
                ContainsResult::AlongBoundary
            );
        }
        {
            let p = e;
            assert_eq!(frame3.contains_pt(&p).unwrap(), ContainsResult::Inside);
        }
        for p in [a, c, g, i] {
            assert_eq!(frame3.contains_pt(&p).unwrap(), ContainsResult::Outside);
        }
    }
}
