use {
    crate::{
        point::Pt,
        segment::{Contains, Intersect, Intersection, Segment},
    },
    float_cmp::approx_eq,
    itertools::zip,
    num::Float,
    std::{
        fmt::Debug,
        iter::Sum,
        ops::{AddAssign, Mul, Sub},
    },
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
pub enum CropToPolygonError {
    #[error("this polygon is not closed; invalid to crop.")]
    ThisPolygonNotClosed,
    #[error("that polygon is not closed; invalid to crop.")]
    ThatPolygonNotClosed,
    #[error("this polygon is not positively oriented; invalid to crop.")]
    ThisPolygonNotPositivelyOriented,
    #[error("that polygon is not positively oriented; invalid to crop.")]
    ThatPolygonNotPositivelyOriented,
    #[error("could not compute a .contains_pt().")]
    ContainsPointError(#[from] ContainsPointError),
}

#[derive(Debug, PartialEq, Eq)]
pub enum CurveOrientation {
    Negative, // clockwise
    Positive, // counter-clockwise
}

#[derive(Debug, PartialEq, Eq)]
pub enum PointLoc {
    Outside,
    Inside,
    OnPoint(usize),
    OnSegment(usize),
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
        T: Copy + Float + PartialOrd + float_cmp::ApproxEq + std::fmt::Debug,
        Pt<T>: PartialEq,
        f64: From<T>,
    {
        for l1 in self.to_segments() {
            for l2 in other.to_segments() {
                if l1.intersects(&l2).is_some() {
                    return true;
                }
            }
        }
        false
    }

    /// Calculates whether a point is within, without, or along a closed polygon
    /// using the https://en.wikipedia.org/wiki/Winding_number method.
    pub fn contains_pt(&self, other: &Pt<T>) -> Result<PointLoc, ContainsPointError>
    where
        T: Float + AddAssign + float_cmp::ApproxEq + Debug,
        Pt<T>: Sub<Output = Pt<T>> + AddAssign<Pt<T>> + PartialEq,
    {
        // If |self| is open, error out.
        if self.kind == PolygonKind::Open {
            return Err(ContainsPointError::PolygonIsOpen);
        }

        for (idx, pt) in self.pts.iter().enumerate() {
            if other == pt {
                return Ok(PointLoc::OnPoint(idx));
            }
        }
        for (idx, seg) in self.to_segments().iter().enumerate() {
            match seg.line_segment_contains_pt(other) {
                Some(Contains::Within) => {
                    return Ok(PointLoc::OnSegment(idx));
                }
                Some(Contains::AtStart | Contains::AtEnd) => {
                    panic!("?");
                }
                _ => {}
            }
        }

        let mut theta: T = T::zero();
        for (i, j) in zip(self.pts.iter(), self.pts.iter().cycle().skip(1)) {
            theta += _abp(other, i, j)
        }

        Ok(match approx_eq!(T, theta, T::zero()) {
            true => PointLoc::Outside,
            false => PointLoc::Inside,
        })
    }

    fn get_curve_orientation(&self) -> CurveOrientation
    where
        T: Float + AddAssign + Mul + Sum,
    {
        if self
            .to_segments()
            .iter()
            .map(|segment| (segment.f.x - segment.i.x) * (segment.f.y + segment.i.y))
            .sum::<T>()
            >= T::zero()
        {
            return CurveOrientation::Negative;
        }
        CurveOrientation::Positive
    }

    fn orient_curve(&mut self)
    where
        T: Float + AddAssign + Mul + Sum,
    {
        if self.get_curve_orientation() == CurveOrientation::Positive {
            self.pts.reverse();
        }
    }

    // NB: Polygons must be closed and positively oriented.
    pub fn crop_to_polygon(&self, frame: &Polygon<T>) -> Result<Vec<Polygon<T>>, CropToPolygonError>
    where
        T: Copy + Float + AddAssign + Mul + Sum + float_cmp::ApproxEq + Debug,
        Pt<T>: PartialEq,
        f64: From<T>,
    {
        if self.kind != PolygonKind::Closed {
            return Err(CropToPolygonError::ThisPolygonNotClosed);
        }
        if frame.kind != PolygonKind::Closed {
            return Err(CropToPolygonError::ThatPolygonNotClosed);
        }
        if self.get_curve_orientation() != CurveOrientation::Positive {
            return Err(CropToPolygonError::ThisPolygonNotPositivelyOriented);
        }
        if frame.get_curve_orientation() != CurveOrientation::Positive {
            return Err(CropToPolygonError::ThatPolygonNotPositivelyOriented);
        }

        // TODO
        // what if self is totally inside self and there are no intersections?
        // just return self.

        // TODO
        // what if frame is totally outside of self and there are no
        // intersections? just return self.

        let self_segs = self.to_segments();
        let frame_segs = frame.to_segments();

        let self_pts = self.pts.iter().enumerate();

        let mut resultant_pts: Vec<Pt<T>> = vec![];

        for (curr_self_pt_idx, curr_self_pt) in self_pts {
            if frame.contains_pt(curr_self_pt)? != PointLoc::Outside {
                resultant_pts.push(curr_self_pt.clone());
                let self_seg = self_segs[curr_self_pt_idx];
                let isxns = frame_segs
                    .iter()
                    .enumerate()
                    .filter_map(|(idx, frame_seg)| match frame_seg.intersects(&self_seg) {
                        Some(isxn @ Intersect::Yes(i)) if !i.on_points_of_either() => {
                            Some((idx, isxn))
                        }
                        _ => None,
                    })
                    .collect::<Vec<_>>();
                if isxns.is_empty() {
                    // no action necessary
                } else {
                    unimplemented!("{:?}", isxns);
                }
            }
        }
        println!("resultant pts: {:?}", resultant_pts);
        Ok(vec![Polygon(resultant_pts).unwrap()])

        // for (i, self_seg) in self.to_segments().iter().enumerate() {
        // for (j, frame_segment) in frame.to_segments().iter().enumerate() {

        // find intersections of outer segments with inner segments.
        // I believe we need a way to know whether the points travel cw or ccw around the polygon.
        // TODO(ambuc)
    }
}

// Angle between points. Projects OI onto OJ and finds the angle IOJ.
fn _abp<T>(o: &Pt<T>, i: &Pt<T>, j: &Pt<T>) -> T
where
    T: Float + Debug,
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
    use super::*;
    use float_eq::assert_float_eq;

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
            assert_eq!(frame1.contains_pt(&p).unwrap(), PointLoc::Inside);
        }
        assert_eq!(frame1.contains_pt(&a).unwrap(), PointLoc::OnPoint(0));
        assert_eq!(frame1.contains_pt(&c).unwrap(), PointLoc::OnPoint(1));
        assert_eq!(frame1.contains_pt(&i).unwrap(), PointLoc::OnPoint(2));
        assert_eq!(frame1.contains_pt(&g).unwrap(), PointLoc::OnPoint(3));

        assert_eq!(frame1.contains_pt(&b).unwrap(), PointLoc::OnSegment(0));
        assert_eq!(frame1.contains_pt(&f).unwrap(), PointLoc::OnSegment(1));
        assert_eq!(frame1.contains_pt(&h).unwrap(), PointLoc::OnSegment(2));
        assert_eq!(frame1.contains_pt(&d).unwrap(), PointLoc::OnSegment(3));

        // frame [a,b,e,d] should contain a, b, d, e...
        let frame2 = Polygon([a, b, e, d]).unwrap();
        assert_eq!(frame2.contains_pt(&a).unwrap(), PointLoc::OnPoint(0));
        assert_eq!(frame2.contains_pt(&b).unwrap(), PointLoc::OnPoint(1));
        assert_eq!(frame2.contains_pt(&e).unwrap(), PointLoc::OnPoint(2));
        assert_eq!(frame2.contains_pt(&d).unwrap(), PointLoc::OnPoint(3));
        for p in [c, f, i, h, g] {
            assert_eq!(frame2.contains_pt(&p).unwrap(), PointLoc::Outside);
        }

        let frame3 = Polygon([b, f, h, d]).unwrap();
        assert_eq!(frame3.contains_pt(&b).unwrap(), PointLoc::OnPoint(0));
        assert_eq!(frame3.contains_pt(&f).unwrap(), PointLoc::OnPoint(1));
        assert_eq!(frame3.contains_pt(&h).unwrap(), PointLoc::OnPoint(2));
        assert_eq!(frame3.contains_pt(&d).unwrap(), PointLoc::OnPoint(3));
        assert_eq!(frame3.contains_pt(&e).unwrap(), PointLoc::Inside);
        for p in [a, c, g, i] {
            assert_eq!(frame3.contains_pt(&p).unwrap(), PointLoc::Outside);
        }
    }

    #[test]
    fn test_crop_to_polygon() {
        //   ^
        //   |
        //   |     |     |     |     |
        // -0,4---1,4---2,4---3,4---4,4--
        //   |     |     |     |     |
        // -0,3---1,3---2,3---3,3---4,3--
        //   |     |     |     |     |
        // -0,2---1,2---2,2---3,2---4,2--
        //   |     |     |     |     |
        // -0,1---1,1---2,1---3,1---4,1->
        //   |     |     |     |     |
        // -0,0---1,0---2,0---3,0---4,0-->
        //   |
        let p0_0 = Pt(0.0, 0.0);
        let p1_0 = Pt(1.0, 0.0);
        let p2_0 = Pt(2.0, 0.0);
        let p3_0 = Pt(3.0, 0.0);
        let p4_0 = Pt(4.0, 0.0);
        let p0_1 = Pt(0.0, 1.0);
        let p1_1 = Pt(1.0, 1.0);
        let p2_1 = Pt(2.0, 1.0);
        let p3_1 = Pt(3.0, 1.0);
        let p4_1 = Pt(4.0, 1.0);
        let p0_2 = Pt(0.0, 2.0);
        let p1_2 = Pt(1.0, 2.0);
        let p2_2 = Pt(2.0, 2.0);
        let p3_2 = Pt(3.0, 2.0);
        let p4_2 = Pt(4.0, 2.0);
        let p0_3 = Pt(0.0, 3.0);
        let p1_3 = Pt(1.0, 3.0);
        let p2_3 = Pt(2.0, 3.0);
        let p3_3 = Pt(3.0, 3.0);
        let p4_3 = Pt(4.0, 3.0);
        let p0_4 = Pt(0.0, 4.0);
        let p1_4 = Pt(1.0, 4.0);
        let p2_4 = Pt(2.0, 4.0);
        let p3_4 = Pt(3.0, 4.0);
        let p4_4 = Pt(4.0, 4.0);

        // total colinearity. but still totally inside.
        assert_eq!(
            Polygon([p1_1, p3_1, p3_3, p1_3])
                .unwrap()
                .crop_to_polygon(&Polygon([p1_1, p3_1, p3_3, p1_3]).unwrap())
                .unwrap()[0]
                .pts,
            Polygon([p1_1, p3_1, p3_3, p1_3]).unwrap().pts,
        );

        // some colinearity but still totally inside.
        assert_eq!(
            Polygon([p1_1, p3_1, p3_3, p1_3])
                .unwrap()
                .crop_to_polygon(&Polygon([p0_0, p3_0, p3_3, p0_3]).unwrap())
                .unwrap()[0]
                .pts,
            Polygon([p1_1, p3_1, p3_3, p1_3]).unwrap().pts,
        );
        assert_eq!(
            Polygon([p1_1, p3_1, p3_3, p1_3])
                .unwrap()
                .crop_to_polygon(&Polygon([p1_1, p4_1, p4_4, p1_4]).unwrap())
                .unwrap()[0]
                .pts,
            Polygon([p1_1, p3_1, p3_3, p1_3]).unwrap().pts,
        );

        // always inside case
        assert_eq!(
            Polygon([p1_1, p3_1, p3_3, p1_3])
                .unwrap()
                .crop_to_polygon(&Polygon([p0_0, p4_0, p4_4, p0_4]).unwrap())
                .unwrap()[0]
                .pts,
            Polygon([p1_1, p3_1, p3_3, p1_3]).unwrap().pts,
        );

        // induce ThisPolygonNotClosed
        assert_eq!(
            Multiline([p1_1, p3_1, p3_3, p1_3])
                .unwrap()
                .crop_to_polygon(&Polygon([p0_0, p4_0, p4_4, p0_4]).unwrap())
                .unwrap_err(),
            CropToPolygonError::ThisPolygonNotClosed
        );

        // induce ThatPolygonNotClosed
        assert_eq!(
            Polygon([p1_1, p3_1, p3_3, p1_3])
                .unwrap()
                .crop_to_polygon(&Multiline([p0_0, p4_0, p4_4, p0_4]).unwrap())
                .unwrap_err(),
            CropToPolygonError::ThatPolygonNotClosed
        );

        // induce ThisPolygonNotPositivelyOriented
        assert_eq!(
            Polygon([p1_1, p1_3, p3_3, p3_1])
                .unwrap()
                .crop_to_polygon(&Polygon([p0_0, p4_0, p4_4, p0_4]).unwrap())
                .unwrap_err(),
            CropToPolygonError::ThisPolygonNotPositivelyOriented
        );

        // induce ThatPolygonNotPositivelyOriented
        assert_eq!(
            Polygon([p1_1, p3_1, p3_3, p1_3])
                .unwrap()
                .crop_to_polygon(&Polygon([p0_0, p0_4, p4_4, p4_0]).unwrap())
                .unwrap_err(),
            CropToPolygonError::ThatPolygonNotPositivelyOriented
        );
    }

    #[test]
    fn test_polygon_get_curve_orientation() {
        //   ^
        //   |
        //   A  B  C
        //   |
        //   D  E  F
        //   |
        // --G--H--I->
        //   |
        let a = Pt(0.0, 2.0);
        let c = Pt(2.0, 2.0);
        let g = Pt(0.0, 0.0);
        let i = Pt(2.0, 0.0);

        assert_eq!(
            Polygon([a, c, i, g]).unwrap().get_curve_orientation(),
            CurveOrientation::Negative
        );
        assert_eq!(
            Polygon([a, g, i, c]).unwrap().get_curve_orientation(),
            CurveOrientation::Positive
        );
    }

    #[test]
    fn test_polygon_orient_curve() {
        //   ^
        //   |
        //   A  B  C
        //   |
        //   D  E  F
        //   |
        // --G--H--I->
        //   |
        let a = Pt(0.0, 2.0);
        let c = Pt(2.0, 2.0);
        let g = Pt(0.0, 0.0);
        let i = Pt(2.0, 0.0);
        let mut p = Polygon([a, g, i, c]).unwrap();
        assert_eq!(p.get_curve_orientation(), CurveOrientation::Positive);
        p.orient_curve();
        assert_eq!(p.get_curve_orientation(), CurveOrientation::Negative);
    }
}
