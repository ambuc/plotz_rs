use {
    crate::{
        point::Pt,
        segment::{Contains, IntersectionOutcome, Segment},
    },
    float_cmp::approx_eq,
    itertools::{all, iproduct, zip},
    num::Float,
    std::{
        fmt::Debug,
        iter::Sum,
        ops::{Add, AddAssign, Mul, Sub},
    },
    thiserror,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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

impl<T> PartialEq for Polygon<T>
where
    Pt<T>: PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        self.pts == other.pts && self.kind == other.kind
    }
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
    #[error("could not construct a polygon.")]
    PolygonConstructorError(#[from] PolygonConstructorError),
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

#[derive(Debug)]
struct Isxn {
    _frame_idx: usize,
    self_idx: usize,
    outcome: IntersectionOutcome,
}

fn next_idx(idx: usize, len: usize) -> usize {
    if idx == len - 1 {
        0
    } else {
        idx + 1
    }
}

fn next<T: Copy>(idx: usize, pts: &[(usize, T)], len: usize) -> (usize, T) {
    let next_idx: usize = next_idx(idx, len);
    pts[next_idx]
}

enum WhichPolygon<'a, T> {
    WSelf((usize, &'a Pt<T>)),
    _WFrame((usize, &'a Pt<T>)),
}
impl<'a, T> WhichPolygon<'a, T> {
    fn inner(&'a self) -> (usize, &'a Pt<T>) {
        match self {
            WhichPolygon::WSelf(x) => *x,
            WhichPolygon::_WFrame(x) => *x,
        }
    }
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

    #[allow(dead_code)]
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

        let self_segs = self.to_segments();
        let frame_segs = frame.to_segments();

        let frame_pts_in_self: Vec<(usize, PointLoc)> = {
            let mut v = vec![];
            for (idx, pt) in frame.pts.iter().enumerate() {
                v.push((idx, self.contains_pt(pt)?));
            }
            Result::<_, ContainsPointError>::Ok(v)
        }?;
        let self_pts_in_frame: Vec<(usize, PointLoc)> = {
            let mut v = vec![];
            for (idx, pt) in self.pts.iter().enumerate() {
                v.push((idx, frame.contains_pt(pt)?));
            }
            Result::<_, ContainsPointError>::Ok(v)
        }?;

        let isxns: Vec<Isxn> =
            iproduct!(frame_segs.iter().enumerate(), self_segs.iter().enumerate())
                .filter_map(|((frame_idx, f_seg), (self_idx, s_seg))| {
                    f_seg.intersects(s_seg).map(|outcome| Isxn {
                        _frame_idx: frame_idx,
                        self_idx,
                        outcome,
                    })
                })
                .collect();

        // If there are no intersections,
        if isxns.is_empty() {
            // Then either all of the frame points are inside self,
            if all(&frame_pts_in_self, |(_idx, isxn)| {
                !matches!(isxn, PointLoc::Outside)
            }) {
                // in which case we ought to return the frame unchanged,
                return Ok(vec![frame.clone()]);
                // or all of the self points are inside frame,
            } else if all(&self_pts_in_frame, |(_idx, isxn)| {
                !matches!(isxn, PointLoc::Outside)
            }) {
                // in which case we ought to return self unchanged.
                return Ok(vec![self.clone()]);
            }
        }

        let self_pts: Vec<_> = self.pts.iter().enumerate().collect();
        let self_pts_len: usize = self_pts.len();
        let frame_pts: Vec<_> = frame.pts.iter().enumerate().collect();
        let frame_pts_len: usize = frame_pts.len();

        let next_self = |idx| next(idx, &self_pts, self_pts_len);
        let _next_frame = |idx| next(idx, &frame_pts, frame_pts_len);

        let mut resultant_polygons: Vec<Polygon<T>> = vec![];
        let mut resultant_pts: Vec<Pt<T>> = vec![];

        assert!(!self_pts_in_frame.is_empty());

        let mut curr: WhichPolygon<T> = WhichPolygon::WSelf(self_pts[0]);

        loop {
            let (curr_idx, curr_pt) = curr.inner();

            // If we've made a cycle,
            if resultant_pts.get(0) == Some(curr_pt) {
                // then break out of it.
                break;
            }

            match frame.contains_pt(curr_pt)? {
                PointLoc::Outside => {
                    unimplemented!("?");
                }
                PointLoc::Inside | PointLoc::OnPoint(_) | PointLoc::OnSegment(_) => {
                    resultant_pts.push(*curr_pt);

                    // If there are any intersections with
                    let relevant_isxns: Vec<_> = isxns
                        .iter()
                        .filter(|isxn| isxn.self_idx == curr_idx)
                        .filter(|isxn| matches!(isxn.outcome, IntersectionOutcome::Yes(intersection) if !intersection.on_points_of_either_polygon()))
                        .collect();

                    if relevant_isxns.is_empty() {
                        // no action necessary, proceed to next point.
                        curr = WhichPolygon::WSelf(next_self(curr_idx));
                    } else {
                        unimplemented!("{:?}", relevant_isxns);
                    }
                }
            }
        }

        // here, check that there aren't any unaccounted-for self points or
        // intersections which did not result in points of resultant polygons.
        // if there are, we need to find other resultants.
        // TODO
        
        resultant_polygons.push(Polygon(resultant_pts)?);

        Ok(resultant_polygons)
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

/// An add operation between a polygon and a point. This can be seen as
/// transposition by |rhs|.
///
/// ```
/// use plotz_geometry::{point::Pt, polygon::Polygon};
/// assert_eq!(
///       &Polygon([Pt(0,0), Pt(1,1), Pt(2,2)]).unwrap()
///     + Pt(1,0),  
///    // --------
///       Polygon([Pt(1,0), Pt(2,1), Pt(3,2)]).unwrap()
/// );
/// ```
impl<T> Add<Pt<T>> for &Polygon<T>
where
    T: Add<Output = T> + Copy,
{
    type Output = Polygon<T>;
    fn add(self, rhs: Pt<T>) -> Self::Output {
        Polygon(self.pts.iter().map(|p| *p + rhs)).unwrap()
    }
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
    fn test_crop_to_polygon_this_not_closed() {
        let p0_0 = Pt(0.0, 0.0);
        let p0_4 = Pt(0.0, 4.0);
        let p1_1 = Pt(1.0, 1.0);
        let p1_3 = Pt(1.0, 3.0);
        let p3_1 = Pt(3.0, 1.0);
        let p3_3 = Pt(3.0, 3.0);
        let p4_0 = Pt(4.0, 0.0);
        let p4_4 = Pt(4.0, 4.0);
        assert_eq!(
            Multiline([p1_1, p3_1, p3_3, p1_3])
                .unwrap()
                .crop_to_polygon(&Polygon([p0_0, p4_0, p4_4, p0_4]).unwrap())
                .unwrap_err(),
            CropToPolygonError::ThisPolygonNotClosed
        );
    }

    #[test]
    fn test_crop_to_polygon_that_not_closed() {
        let p0_0 = Pt(0.0, 0.0);
        let p0_4 = Pt(0.0, 4.0);
        let p1_1 = Pt(1.0, 1.0);
        let p1_3 = Pt(1.0, 3.0);
        let p3_1 = Pt(3.0, 1.0);
        let p3_3 = Pt(3.0, 3.0);
        let p4_0 = Pt(4.0, 0.0);
        let p4_4 = Pt(4.0, 4.0);
        assert_eq!(
            Polygon([p1_1, p3_1, p3_3, p1_3])
                .unwrap()
                .crop_to_polygon(&Multiline([p0_0, p4_0, p4_4, p0_4]).unwrap())
                .unwrap_err(),
            CropToPolygonError::ThatPolygonNotClosed
        );
    }

    #[test]
    fn test_crop_to_polygon_this_not_positively_oriented() {
        let p0_0 = Pt(0.0, 0.0);
        let p0_4 = Pt(0.0, 4.0);
        let p1_1 = Pt(1.0, 1.0);
        let p1_3 = Pt(1.0, 3.0);
        let p3_1 = Pt(3.0, 1.0);
        let p3_3 = Pt(3.0, 3.0);
        let p4_0 = Pt(4.0, 0.0);
        let p4_4 = Pt(4.0, 4.0);
        assert_eq!(
            Polygon([p1_1, p1_3, p3_3, p3_1])
                .unwrap()
                .crop_to_polygon(&Polygon([p0_0, p4_0, p4_4, p0_4]).unwrap())
                .unwrap_err(),
            CropToPolygonError::ThisPolygonNotPositivelyOriented
        );
    }

    #[test]
    fn test_crop_to_polygon_that_not_positively_oriented() {
        let p0_0 = Pt(0.0, 0.0);
        let p0_4 = Pt(0.0, 4.0);
        let p1_1 = Pt(1.0, 1.0);
        let p1_3 = Pt(1.0, 3.0);
        let p3_1 = Pt(3.0, 1.0);
        let p3_3 = Pt(3.0, 3.0);
        let p4_0 = Pt(4.0, 0.0);
        let p4_4 = Pt(4.0, 4.0);
        assert_eq!(
            Polygon([p1_1, p3_1, p3_3, p1_3])
                .unwrap()
                .crop_to_polygon(&Polygon([p0_0, p0_4, p4_4, p4_0]).unwrap())
                .unwrap_err(),
            CropToPolygonError::ThatPolygonNotPositivelyOriented
        );
    }

    #[test]
    fn test_crop_to_polygon_inner_equals_frame() {
        // ⬆️ y
        // ⬜⬜⬜⬜⬜
        // ⬜🟧🟧🟧⬜
        // ⬜🟧🟧🟧⬜
        // ⬜🟧🟧🟧⬜
        // ⬜⬜⬜⬜⬜ ➡️ x
        let p1_1 = Pt(1.0, 1.0);
        let p3_1 = Pt(3.0, 1.0);
        let p1_3 = Pt(1.0, 3.0);
        let p3_3 = Pt(3.0, 3.0);
        let inner = Polygon([p1_1, p3_1, p3_3, p1_3]).unwrap(); // 🟥
        let frame = Polygon([p1_1, p3_1, p3_3, p1_3]).unwrap(); // 🟨
        assert_eq!(inner, frame);
        let crops = inner.crop_to_polygon(&frame).unwrap(); // 🟧
        assert_eq!(crops, vec![inner]);
    }

    #[test]
    fn test_crop_to_polygon_inner_colinear_to_frame() {
        // ⬆️ y
        // ⬜⬜⬜⬜⬜
        // 🟨🟧🟧🟧⬜
        // 🟨🟧🟧🟧⬜
        // 🟨🟧🟧🟧⬜
        // 🟨🟨🟨🟨⬜ ➡️ x
        let p0_0 = Pt(0.0, 0.0);
        let p0_3 = Pt(0.0, 3.0);
        let p1_1 = Pt(1.0, 1.0);
        let p1_3 = Pt(1.0, 3.0);
        let p3_0 = Pt(3.0, 0.0);
        let p3_1 = Pt(3.0, 1.0);
        let p3_3 = Pt(3.0, 3.0);
        let inner = Polygon([p1_1, p3_1, p3_3, p1_3]).unwrap(); // 🟥
        let frame = Polygon([p0_0, p3_0, p3_3, p0_3]).unwrap(); // 🟨
        assert_eq!(inner.crop_to_polygon(&frame).unwrap()[0].pts, inner.pts);

        // ⬆️ y
        // ⬜⬜⬜⬜⬜
        // ⬜🟧🟧🟧🟨
        // ⬜🟧🟧🟧🟨
        // ⬜🟧🟧🟧🟨
        // ⬜🟨🟨🟨🟨 ➡️ x
        assert_eq!(
            inner.crop_to_polygon(&(&frame + Pt(1.0, 0.0))).unwrap()[0].pts,
            inner.pts
        );

        // ⬆️ y
        // 🟨🟨🟨🟨⬜
        // 🟨🟧🟧🟧⬜
        // 🟨🟧🟧🟧⬜
        // 🟨🟧🟧🟧⬜
        // ⬜⬜⬜⬜⬜ ➡ x
        assert_eq!(
            inner.crop_to_polygon(&(&frame + Pt(0.0, 1.0))).unwrap()[0].pts,
            inner.pts
        );

        // ⬆️ y
        // ⬜🟨🟨🟨🟨
        // ⬜🟧🟧🟧🟨
        // ⬜🟧🟧🟧🟨
        // ⬜🟧🟧🟧🟨
        // ⬜⬜⬜⬜⬜ ➡ x
        assert_eq!(
            inner.crop_to_polygon(&(&frame + Pt(1.0, 1.0))).unwrap()[0].pts,
            inner.pts
        );
    }

    #[test]
    fn test_crop_to_polygon_inner_totally_within_frame() {
        // ⬆️ y
        // 🟨🟨🟨🟨🟨
        // 🟨🟧🟧🟧🟨
        // 🟨🟧🟧🟧🟨
        // 🟨🟧🟧🟧🟨
        // 🟨🟨🟨🟨🟨 ➡️ x
        let p0_0 = Pt(0.0, 0.0);
        let p0_4 = Pt(0.0, 4.0);
        let p1_1 = Pt(1.0, 1.0);
        let p1_3 = Pt(1.0, 3.0);
        let p3_1 = Pt(3.0, 1.0);
        let p3_3 = Pt(3.0, 3.0);
        let p4_0 = Pt(4.0, 0.0);
        let p4_4 = Pt(4.0, 4.0);
        let inner = Polygon([p1_1, p3_1, p3_3, p1_3]).unwrap(); // 🟥
        let frame = Polygon([p0_0, p4_0, p4_4, p0_4]).unwrap(); // 🟨

        // inner /\ frame == inner
        let crops = inner.crop_to_polygon(&frame).unwrap(); // 🟧
        assert_eq!(crops, vec![inner.clone()]);
        // frame /\ inner = inner
        let crops = frame.crop_to_polygon(&inner).unwrap(); // 🟧
        assert_eq!(crops, vec![inner]);
    }

    #[test]
    fn test_crop_to_polygon_two_pivots() {
        // ⬆️ y
        // ⬜⬜⬜⬜⬜
        // ⬜🟥🟥🟥⬜
        // 🟨🟧🟧🟥⬜
        // 🟨🟧🟧🟥⬜
        // 🟨🟨🟨⬜⬜ ➡️ x
        let p0_0 = Pt(0.0, 0.0);
        let p0_3 = Pt(0.0, 3.0);
        let p1_1 = Pt(1.0, 1.0);
        let p1_3 = Pt(1.0, 3.0);
        let p1_4 = Pt(1.0, 4.0);
        let p3_0 = Pt(3.0, 0.0);
        let p3_1 = Pt(3.0, 1.0);
        let p3_3 = Pt(3.0, 3.0);
        let p4_1 = Pt(4.0, 1.0);
        let p4_4 = Pt(4.0, 4.0);
        let inner = Polygon([p1_1, p4_1, p4_4, p1_4]).unwrap(); // 🟥
        let frame = Polygon([p0_0, p3_0, p3_3, p0_3]).unwrap(); // 🟨
        let expected = Polygon([p1_1, p3_1, p3_3, p1_3]).unwrap(); // 🟧

        let crops = inner.crop_to_polygon(&frame).unwrap();
        assert_eq!(crops, vec![expected.clone()]);
        let crops = frame.crop_to_polygon(&inner).unwrap();
        assert_eq!(crops, vec![expected.clone()]);
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
