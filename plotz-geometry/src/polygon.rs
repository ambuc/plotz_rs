//! A 2D polygon (or multi&line).

use {
    crate::{
        bounded::Bounded,
        interpolate,
        point::Pt,
        segment::{Contains, Intersection, IntersectionOutcome, Segment},
    },
    derivative::Derivative,
    either::Either,
    float_cmp::approx_eq,
    itertools::{all, iproduct, zip},
    std::{
        cmp::{Eq, PartialEq},
        collections::HashSet,
        fmt::Debug,
        ops::{Add, AddAssign, MulAssign, Sub, SubAssign},
    },
    thiserror::Error,
};

/// Whether a polygon is open (there should be no line drawn between its last
/// and first points) or closed (a line should be drawn between its last and
/// first points).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PolygonKind {
    /// A polygon is open.
    Open,
    /// A polygon is closed.
    Closed,
}

/// A multiline is a list of points rendered with connecting line segments.
/// If constructed with PolygonKind::Open, this is a multiline (unshaded).
/// If constructed with PolygonKind::Closed, this is a closed, shaded polygon.
#[derive(Debug, Clone)]
pub struct Polygon {
    /// The points which describe a polygon or multiline.
    pub pts: Vec<Pt>,
    /// Whether this polygon is open or closed.
    pub kind: PolygonKind,
}

impl PartialEq for Polygon {
    fn eq(&self, other: &Self) -> bool {
        let self_idx_of_min = self
            .pts
            .iter()
            .enumerate()
            .max_by(|(_, a), (_, b)| a.cmp(b))
            .map(|(idx, _)| idx)
            .unwrap();
        let self_new_pts: Vec<_> = self
            .pts
            .iter()
            .cycle()
            .skip(self_idx_of_min)
            .take(self.pts.len())
            .collect();

        let other_idx_of_min = other
            .pts
            .iter()
            .enumerate()
            .max_by(|(_, a), (_, b)| a.cmp(b))
            .map(|(idx, _)| idx)
            .unwrap();
        let other_new_pts: Vec<_> = other
            .pts
            .iter()
            .cycle()
            .skip(other_idx_of_min)
            .take(other.pts.len())
            .collect();

        self_new_pts == other_new_pts && self.kind == other.kind
    }
}

/// A general error arising from trying to construct a Multiline.
#[derive(Debug, Error, PartialEq, Eq)]
pub enum MultilineConstructorError {
    /// It is not possible to construct a multiline from one or fewer points.
    #[error("It is not possible to construct a multiline from one or fewer points.")]
    OneOrFewerPoints,
}

/// Constructor for multilines. Multilines must have at least one line, so they
/// must have two or more points. Constructing a multiline from one or fewer
/// points will result in a MultilineConstructorError.
#[allow(non_snake_case)]
pub fn Multiline(a: impl IntoIterator<Item = Pt>) -> Result<Polygon, MultilineConstructorError> {
    let pts: Vec<Pt> = a.into_iter().collect();
    if pts.len() <= 1 {
        return Err(MultilineConstructorError::OneOrFewerPoints);
    }
    Ok(Polygon {
        pts,
        kind: PolygonKind::Open,
    })
}

/// A general error arising from trying to construct a Polygon.
#[derive(Debug, Error, PartialEq, Eq)]
pub enum PolygonConstructorError {
    /// It is not possible to construct a polygon from two or fewer points.
    #[error("It is not possible to construct a polygon from two or fewer points.")]
    TwoOrFewerPoints,
}

/// Constructor for polygons. Polygons must have inner area, so they must have
/// three or more points. Constructing a polygon from two or fewer points will
/// result in a PolygonConstructorErrorip
#[allow(non_snake_case)]
pub fn Polygon(a: impl IntoIterator<Item = Pt>) -> Result<Polygon, PolygonConstructorError> {
    let pts: Vec<Pt> = a.into_iter().collect();
    if pts.len() <= 2 {
        return Err(PolygonConstructorError::TwoOrFewerPoints);
    }
    Ok(Polygon {
        pts,
        kind: PolygonKind::Closed,
    })
}

/// A general error arising from trying to inspect whether a point lies in a
/// polygon.
#[derive(Debug, Error, PartialEq, Eq)]
pub enum ContainsPointError {
    /// The bounding polygon is Open (not Closed) and so it is underspecified to
    /// ask whether it contains a point.
    #[error("The bounding polygon is Open (not Closed) and so it is underspecified to ask whether it contains a point.")]
    PolygonIsOpen,
}

/// A general error arising from trying to crop something to a bounding polygon.
#[derive(Debug, Error, PartialEq, Eq)]
pub enum CropToPolygonError {
    /// The frame polygon is not closed.
    #[error("The frame polygon is not closed.")]
    ThisPolygonNotClosed,
    /// The inner polygon is not closed.
    #[error("The inner polygon is not closed.")]
    ThatPolygonNotClosed,
    /// The frame polygon is not positively oriented.
    #[error("The frame polygon is not positively oriented.")]
    ThisPolygonNotPositivelyOriented,
    /// The inner polygon is not positively oriented.
    #[error("The inner polygon is not positively oriented.")]
    ThatPolygonNotPositivelyOriented,
    /// Some inspection of whether a point lies in a polygon failed.
    #[error("Some inspection of whether a point lies in a polygon failed.")]
    ContainsPointError(#[from] ContainsPointError),
    /// Some Polygon construction failed.
    #[error("Some Polygon construction failed.")]
    PolygonConstructorError(#[from] PolygonConstructorError),
    /// Constructing a resultant polygon failed because we encountered a cycle.
    #[error("Constructing a resultant polygon failed because we encountered a cycle.")]
    CycleError,
}

/// Whether a curve is positively or negatively oriented (whether its points are
/// listed in clockwise or counter-clockwise order).
#[derive(Debug, PartialEq, Eq)]
pub enum CurveOrientation {
    /// Negatively oriented, i.e. points listed in clockwise order.
    Negative,
    /// Positively oriented, i.e. points listed in counter-clockwise order.
    Positive,
}

/// Whether a point lies outside, inside, or on a vertex or edge of a polygon.
#[derive(Debug, PartialEq, Eq)]
pub enum PointLoc {
    /// A point lies outside a polygon.
    Outside,
    /// A point lies inside a polygon.
    Inside,
    /// A point lies on the nth point of a polygon.
    OnPoint(usize),
    /// A point lies on the nth segment of a polygon.
    OnSegment(usize),
}

#[derive(Debug)]
struct IsxnOutcome {
    frame_segment_idx: usize,
    inner_segment_idx: usize,
    outcome: IntersectionOutcome,
}
impl IsxnOutcome {
    fn to_isxn(&self) -> Option<Isxn> {
        match self.outcome {
            IntersectionOutcome::Yes(i) => Some(Isxn {
                frame_segment_idx: self.frame_segment_idx,
                inner_segment_idx: self.inner_segment_idx,
                intersection: i,
            }),
            _ => None,
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
struct Isxn {
    frame_segment_idx: usize,
    inner_segment_idx: usize,
    intersection: Intersection,
}
impl Isxn {
    pub fn pt_given_self_segs(&self, self_segs: &[(usize, Segment)]) -> Pt {
        let (_, seg) = self_segs[self.inner_segment_idx];
        interpolate::extrapolate_2d(seg.i, seg.f, self.intersection.percent_along_inner.0)
    }
}

#[derive(Debug, Copy, Clone)]
enum On {
    OnInner,
    OnFrame,
}
impl On {
    pub fn flip(&self) -> On {
        match self {
            On::OnInner => On::OnFrame,
            On::OnFrame => On::OnInner,
        }
    }
}

#[derive(Debug, Copy, Clone)]
struct OnePolygon {
    on_polygon: On,
    at_point_index: usize,
}

#[derive(Derivative)]
#[derivative(Debug)]
struct Cursor<'a> {
    // current position
    position: Either<OnePolygon, Isxn>,
    facing_along: On,
    facing_along_segment_idx: usize, // segment index
    // context
    #[derivative(Debug = "ignore")]
    inner_pts: &'a Vec<(usize, &'a Pt)>,
    #[derivative(Debug = "ignore")]
    inner_pts_len: &'a usize,
    #[derivative(Debug = "ignore")]
    frame_pts: &'a Vec<(usize, &'a Pt)>,
    #[derivative(Debug = "ignore")]
    frame_pts_len: &'a usize,
    #[derivative(Debug = "ignore")]
    inner_segments: &'a Vec<(usize, Segment)>,
}
impl<'a> Cursor<'a> {
    fn pt(&self) -> Pt {
        match &self.position {
            Either::Left(one_polygon) => match one_polygon.on_polygon {
                On::OnInner => *self.inner_pts[one_polygon.at_point_index].1,
                On::OnFrame => *self.frame_pts[one_polygon.at_point_index].1,
            },
            Either::Right(isxn) => isxn.pt_given_self_segs(self.inner_segments),
        }
    }
    fn pts_len(&self, on: On) -> usize {
        match on {
            On::OnInner => *self.inner_pts_len,
            On::OnFrame => *self.frame_pts_len,
        }
    }
    fn march_to_next_point(&mut self) {
        let v = (match self.position {
            Either::Left(one_polygon) => one_polygon.at_point_index,
            Either::Right(isxn) => match self.facing_along {
                On::OnInner => isxn.inner_segment_idx,
                On::OnFrame => isxn.frame_segment_idx,
            },
        } + 1)
            % self.pts_len(self.facing_along);
        self.position = Either::Left(OnePolygon {
            on_polygon: self.facing_along,
            at_point_index: v,
        });
        self.facing_along_segment_idx = v;
    }

    fn march_to_isxn(&mut self, next_isxn: Isxn, should_flip: bool) {
        let new_position: Either<_, Isxn> = Either::Right(next_isxn);
        let new_facing_along = if should_flip {
            self.facing_along.flip()
        } else {
            self.facing_along
        };
        let new_facing_along_segment_idx = match new_facing_along {
            On::OnFrame => next_isxn.frame_segment_idx,
            On::OnInner => next_isxn.inner_segment_idx,
        };
        self.position = new_position;
        self.facing_along = new_facing_along;
        self.facing_along_segment_idx = new_facing_along_segment_idx;
    }
}

impl Polygon {
    /// Inverts the polygon across the x-axis.
    pub fn flip_x(&mut self) {
        self.pts.iter_mut().for_each(|p| p.x.0 *= -1.0);
    }
    /// Inverts the polygon across the y-axis.
    pub fn flip_y(&mut self) {
        self.pts.iter_mut().for_each(|p| p.y.0 *= -1.0);
    }
    /// Returns the segments of a polygon, one at a time.
    ///
    /// If this is an open polygon, we return only the line segments without the
    /// final closure.
    ///
    /// If this is a closed polygon, we also generate the final closure.
    ///
    /// See test_multiline_to_segments() and test_polygon_to_segments() for
    /// examples.
    pub fn to_segments(&self) -> Vec<Segment> {
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
    pub fn intersects(&self, other: &Polygon) -> bool {
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
    pub fn contains_pt(&self, other: &Pt) -> Result<PointLoc, ContainsPointError> {
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

        let mut theta = 0_f64;
        for (i, j) in zip(self.pts.iter(), self.pts.iter().cycle().skip(1)) {
            theta += _abp(other, i, j)
        }

        Ok(match approx_eq!(f64, theta, 0_f64, epsilon = 0.00001) {
            true => PointLoc::Outside,
            false => PointLoc::Inside,
        })
    }

    fn get_curve_orientation(&self) -> CurveOrientation {
        if self
            .to_segments()
            .iter()
            .map(|segment| (segment.f.x.0 - segment.i.x.0) * (segment.f.y.0 + segment.i.y.0))
            .sum::<f64>()
            >= 0_f64
        {
            return CurveOrientation::Negative;
        }
        CurveOrientation::Positive
    }

    #[allow(dead_code)]
    fn orient_curve(&mut self) {
        if self.get_curve_orientation() == CurveOrientation::Positive {
            self.pts.reverse();
        }
    }

    /// Treating this polygon as a frame, crop an |inner| polygon to it.
    pub fn as_frame_to_polygon(&self, inner: &Polygon) -> Result<Vec<Polygon>, CropToPolygonError> {
        inner.crop_to_polygon(self)
    }

    /// Treating this polygon as a frame, crop an |inner| segment to it.
    pub fn as_frame_to_segment(&self, inner: &Segment) -> Result<Vec<Segment>, CropToPolygonError> {
        let frame_segments = self.to_segments();
        let mut resultants: Vec<Segment> = vec![];
        let mut curr_pt = inner.i;
        let mut curr_pen_down = !matches!(self.contains_pt(&inner.i)?, PointLoc::Outside);
        loop {
            if curr_pt == inner.f {
                break;
            }

            let mut isxns = frame_segments
                .iter()
                .filter_map(|f| inner.intersects(f))
                .filter_map(|isxn_outcome| match isxn_outcome {
                    IntersectionOutcome::Yes(isxn) => Some(isxn),
                    _ => None,
                })
                .collect::<Vec<Intersection>>();
            isxns.sort_by(|i, j| i.percent_along_inner.cmp(&j.percent_along_inner));
            let (_, vs) = isxns.into_iter().partition(|i| {
                i.percent_along_inner.0
                    <= interpolate::interpolate_2d_checked(inner.i, inner.f, curr_pt)
                        .unwrap_or_else(|_| {
                            panic!(
                                "interpolate failed: a: {:?}, b: {:?}, i: {:?}",
                                inner.i, inner.f, curr_pt,
                            )
                        })
            });
            isxns = vs;

            match isxns.get(0) {
                Some(intersection) => {
                    let new_pt = interpolate::extrapolate_2d(
                        inner.i,
                        inner.f,
                        intersection.percent_along_inner.0,
                    );
                    if !matches!(self.contains_pt(&new_pt)?, PointLoc::Outside) && curr_pen_down {
                        resultants.push(Segment(curr_pt, new_pt));
                    }
                    curr_pt = new_pt;
                    curr_pen_down = !curr_pen_down;
                }
                None => {
                    return Ok(resultants);
                }
            }
        }

        Ok(resultants)
    }

    /// Crop this polygon to some frame. Returns a list of resultant polygons.
    /// Both polygons must already be closed and positively oriented.
    ///
    /// Known bug: If multiple resultant polygons are present, this will return
    /// only one.
    pub fn crop_to_polygon(&self, frame: &Polygon) -> Result<Vec<Polygon>, CropToPolygonError> {
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

        let inner_segments: Vec<_> = self.to_segments().into_iter().enumerate().collect();
        let frame_segments: Vec<_> = frame.to_segments().into_iter().enumerate().collect();

        let isxn_outcomes: Vec<IsxnOutcome> = iproduct!(&inner_segments, &frame_segments)
            .filter_map(|((inner_idx, i_seg), (frame_idx, f_seg))| {
                i_seg.intersects(f_seg).map(|outcome| IsxnOutcome {
                    frame_segment_idx: *frame_idx,
                    inner_segment_idx: *inner_idx,
                    outcome,
                })
            })
            .collect();

        // If there are no intersections,
        if isxn_outcomes.is_empty() {
            let frame_pts_in_inner: Vec<(usize, PointLoc)> = {
                let mut v = vec![];
                for (idx, pt) in frame.pts.iter().enumerate() {
                    v.push((idx, self.contains_pt(pt)?));
                }
                Result::<_, ContainsPointError>::Ok(v)
            }?;
            let inner_pts_in_frame: Vec<(usize, PointLoc)> = {
                let mut v = vec![];
                for (idx, pt) in self.pts.iter().enumerate() {
                    v.push((idx, frame.contains_pt(pt)?));
                }
                Result::<_, ContainsPointError>::Ok(v)
            }?;

            let all_frame_points_in_inner = all(&frame_pts_in_inner, |(_idx, isxn)| {
                !matches!(isxn, PointLoc::Outside)
            });
            let all_inner_points_in_frame = all(&inner_pts_in_frame, |(_idx, isxn)| {
                !matches!(isxn, PointLoc::Outside)
            });

            // Then either all of the frame points are inside inner,
            if all_frame_points_in_inner {
                // in which case we ought to return the frame unchanged,
                return Ok(vec![frame.clone()]);
                // or all of the inner points are inside frame,
            } else if all_inner_points_in_frame {
                // in which case we ought to return inner unchanged.
                return Ok(vec![self.clone()]);
            }
        }

        let inner_pts: Vec<_> = self.pts.iter().enumerate().collect();
        let frame_pts: Vec<_> = frame.pts.iter().enumerate().collect();

        let mut resultant_polygons: Vec<Polygon> = vec![];
        let mut resultant_pts: Vec<Pt> = vec![];

        let mut visited_pts = HashSet::<Pt>::new();

        let mut curr = Cursor {
            position: Either::Left(OnePolygon {
                on_polygon: On::OnInner,
                at_point_index: 0,
            }),
            facing_along: On::OnInner,
            facing_along_segment_idx: 0_usize,
            //
            inner_pts: &inner_pts,
            inner_pts_len: &inner_pts.len(),
            frame_pts: &frame_pts,
            frame_pts_len: &frame_pts.len(),
            inner_segments: &inner_segments,
        };

        'outer: loop {
            let curr_pt: Pt = curr.pt();

            // If we've made a cycle,
            if let Some(pt) = resultant_pts.get(0) {
                if *pt == curr_pt {
                    // then break out of it.
                    break 'outer;
                }
            }

            // If we've revisited a point otherwise, it is an error.
            if !visited_pts.insert(curr_pt) {
                return Err(CropToPolygonError::CycleError);
            }

            let mut relevant_isxns: Vec<Isxn> = isxn_outcomes
                .iter()
                .filter_map(|isxn_outcome| isxn_outcome.to_isxn())
                .filter(|isxn| {
                    (match curr.facing_along {
                        On::OnInner => isxn.inner_segment_idx,
                        On::OnFrame => isxn.frame_segment_idx,
                    }) == curr.facing_along_segment_idx
                })
                // then collect them.
                .collect();

            relevant_isxns.sort_by(|a: &Isxn, b: &Isxn| match &curr.facing_along {
                On::OnInner => a
                    .intersection
                    .percent_along_inner
                    .partial_cmp(&b.intersection.percent_along_inner)
                    .unwrap(),
                On::OnFrame => a
                    .intersection
                    .percent_along_frame
                    .partial_cmp(&b.intersection.percent_along_frame)
                    .unwrap(),
            });

            match curr.position {
                Either::Left(_) => {
                    let (_drained, v) =
                        relevant_isxns
                            .into_iter()
                            .partition(|isxn| match curr.facing_along {
                                On::OnInner => isxn.intersection.percent_along_inner.0 == 0.0,
                                On::OnFrame => isxn.intersection.percent_along_frame.0 == 0.0,
                            });
                    relevant_isxns = v;
                }
                Either::Right(this_isxn) => {
                    let (_drained, v) =
                        relevant_isxns
                            .into_iter()
                            .partition(|isxn| match curr.facing_along {
                                On::OnInner => {
                                    isxn.intersection.percent_along_inner
                                        <= this_isxn.intersection.percent_along_inner
                                }
                                On::OnFrame => {
                                    isxn.intersection.percent_along_frame
                                        <= this_isxn.intersection.percent_along_frame
                                }
                            });
                    relevant_isxns = v;
                }
            }

            if !matches!(frame.contains_pt(&curr_pt)?, PointLoc::Outside) {
                resultant_pts.push(curr_pt);
            }

            if relevant_isxns.is_empty() {
                curr.march_to_next_point();
            } else {
                match curr.position {
                    Either::Left(_) => {
                        let next_isxn = relevant_isxns.first().expect("?");
                        curr.march_to_isxn(
                            *next_isxn, /*should_flip */
                            !matches!(frame.contains_pt(&curr_pt)?, PointLoc::Outside),
                        );
                    }
                    Either::Right(_) => {
                        match relevant_isxns.get(0) {
                            Some(next_isxn) => {
                                curr.march_to_isxn(*next_isxn, /*should_flip */ true);
                            }
                            None => {
                                curr.march_to_next_point();
                            }
                        }
                    }
                }
            }
        }

        // here, check that there aren't any unaccounted-for inner points or
        // intersections which did not result in points of resultant polygons.
        // if there are, we need to find other resultants.
        // TODO

        resultant_polygons.push(Polygon(resultant_pts)?);

        Ok(resultant_polygons)
    }
}

// Angle between points. Projects OI onto OJ and finds the angle IOJ.
fn _abp(o: &Pt, i: &Pt, j: &Pt) -> f64 {
    let a: Pt = *i - *o;
    let b: Pt = *j - *o;
    f64::atan2(
        /*det=*/ a.x.0 * b.y.0 - a.y.0 * b.x.0,
        /*dot=*/ a.x.0 * b.x.0 + a.y.0 * b.y.0,
    )
}

/// An add operation between a polygon and a point. This can be seen as
/// transposition by |rhs|.
impl Add<Pt> for &Polygon {
    type Output = Polygon;
    fn add(self, rhs: Pt) -> Self::Output {
        Polygon(self.pts.iter().map(|p| *p + rhs)).unwrap()
    }
}
impl AddAssign<Pt> for Polygon {
    fn add_assign(&mut self, rhs: Pt) {
        self.pts.iter_mut().for_each(|p| *p += rhs);
    }
}
impl Sub<Pt> for &Polygon {
    type Output = Polygon;
    fn sub(self, rhs: Pt) -> Self::Output {
        Polygon(self.pts.iter().map(|p| *p - rhs)).unwrap()
    }
}
impl SubAssign<Pt> for Polygon {
    fn sub_assign(&mut self, rhs: Pt) {
        self.pts.iter_mut().for_each(|p| *p -= rhs);
    }
}
impl MulAssign<f64> for Polygon {
    fn mul_assign(&mut self, rhs: f64) {
        self.pts.iter_mut().for_each(|p| *p *= rhs);
    }
}

impl Bounded for Polygon {
    fn top_bound(&self) -> f64 {
        self.pts.iter().map(|p| p.y).max().expect("not empty").0
    }
    fn bottom_bound(&self) -> f64 {
        self.pts.iter().map(|p| p.y).min().expect("not empty").0
    }
    fn left_bound(&self) -> f64 {
        self.pts.iter().map(|p| p.x).min().expect("not empty").0
    }
    fn right_bound(&self) -> f64 {
        self.pts.iter().map(|p| p.x).max().expect("not empty").0
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
        let a = Pt(0, 2);
        let b = Pt(1, 2);
        let c = Pt(2, 2);
        let d = Pt(0, 1);
        let e = Pt(1, 1);
        let f = Pt(2, 1);
        let g = Pt(0, 0);
        let h = Pt(1, 0);
        let i = Pt(2, 0);

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
        let a = Pt(0, 2);
        let b = Pt(1, 2);
        let c = Pt(2, 2);
        let d = Pt(0, 1);
        let e = Pt(1, 1);
        let f = Pt(2, 1);
        let g = Pt(0, 0);
        let h = Pt(1, 0);
        let i = Pt(2, 0);

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
        assert_float_eq!(_abp(&g, &i, &f), 0.5_f64.atan(), ulps <= 10);
        assert_float_eq!(_abp(&g, &i, &e), 1.0_f64.atan(), ulps <= 10);
        assert_float_eq!(_abp(&g, &i, &c), 1.0_f64.atan(), ulps <= 10);
        assert_float_eq!(_abp(&g, &i, &b), 2.0_f64.atan(), ulps <= 10);
        assert_float_eq!(_abp(&g, &i, &d), PI / 2.0, ulps <= 10);
        assert_float_eq!(_abp(&g, &i, &a), PI / 2.0, ulps <= 10);

        // circle around H (quadrants 1, 2)
        assert_float_eq!(_abp(&h, &i, &i), 0.0, ulps <= 10);
        assert_float_eq!(_abp(&h, &i, &b), PI / 2.0, ulps <= 10);
        assert_float_eq!(_abp(&h, &i, &a), PI / 2.0 + 0.5_f64.atan(), ulps <= 10);
        assert_float_eq!(_abp(&h, &i, &d), PI / 2.0 + 1.0_f64.atan(), ulps <= 10);
        assert_float_eq!(_abp(&h, &i, &g), PI, ulps <= 10);

        // circle around B (quadrants 3, 4)
        assert_float_eq!(_abp(&b, &c, &c), 0.0, ulps <= 10);
        assert_float_eq!(_abp(&b, &c, &f), -1.0_f64.atan(), ulps <= 10);
        assert_float_eq!(_abp(&b, &c, &i), -2.0_f64.atan(), ulps <= 10);
        assert_float_eq!(_abp(&b, &c, &e), -1.0 * PI / 2.0, ulps <= 10);
        assert_float_eq!(_abp(&b, &c, &h), -1.0 * PI / 2.0, ulps <= 10);
        assert_float_eq!(
            _abp(&b, &c, &g),
            -1.0 * PI / 2.0 - 0.5_f64.atan(),
            ulps <= 10
        );
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
        let a = Pt(0, 2);
        let b = Pt(1, 2);
        let c = Pt(2, 2);
        let d = Pt(0, 1);
        let e = Pt(1, 1);
        let f = Pt(2, 1);
        let g = Pt(0, 0);
        let h = Pt(1, 0);
        let i = Pt(2, 0);

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
    fn test_contains_pt_regression() {
        let frame = Polygon([
            Pt(228.17, 202.35),
            Pt(231.21, 212.64),
            Pt(232.45, 228.76),
            Pt(231.67, 257.09),
            Pt(230.63, 265.17),
            Pt(263.66, 335.37),
            Pt(261.85, 336.27),
            Pt(295.65, 404.87),
            Pt(298.24, 409.14),
            Pt(302.39, 413.67),
            Pt(305.92, 412.20),
            Pt(309.33, 417.90),
            Pt(311.03, 417.06),
            Pt(312.99, 420.06),
            Pt(318.55, 420.99),
            Pt(322.66, 420.45),
            Pt(325.57, 419.13),
            Pt(343.70, 406.83),
            Pt(336.17, 404.87),
            Pt(230.61, 185.93),
            Pt(228.83, 189.47),
            Pt(227.19, 195.84),
            Pt(228.17, 202.35),
        ])
        .unwrap();
        let suspicious_pt = Pt(228, 400);
        assert_eq!(frame.contains_pt(&suspicious_pt), Ok(PointLoc::Outside));
    }

    #[test]
    fn test_crop_to_polygon_this_not_closed() {
        assert_eq!(
            Multiline([Pt(1, 1), Pt(3, 1), Pt(3, 3), Pt(1, 3)])
                .unwrap()
                .crop_to_polygon(&Polygon([Pt(0, 0), Pt(4, 0), Pt(4, 4), Pt(0, 4)]).unwrap())
                .unwrap_err(),
            CropToPolygonError::ThisPolygonNotClosed
        );
    }

    #[test]
    fn test_crop_to_polygon_that_not_closed() {
        assert_eq!(
            Polygon([Pt(1, 1), Pt(3, 1), Pt(3, 3), Pt(1, 3)])
                .unwrap()
                .crop_to_polygon(&Multiline([Pt(0, 0), Pt(4, 0), Pt(4, 4), Pt(0, 4)]).unwrap())
                .unwrap_err(),
            CropToPolygonError::ThatPolygonNotClosed
        );
    }

    #[test]
    fn test_crop_to_polygon_this_not_positively_oriented() {
        assert_eq!(
            Polygon([Pt(1, 1), Pt(1, 3), Pt(3, 3), Pt(3, 1)])
                .unwrap()
                .crop_to_polygon(&Polygon([Pt(0, 0), Pt(4, 0), Pt(4, 4), Pt(0, 4)]).unwrap())
                .unwrap_err(),
            CropToPolygonError::ThisPolygonNotPositivelyOriented
        );
    }

    #[test]
    fn test_crop_to_polygon_that_not_positively_oriented() {
        assert_eq!(
            Polygon([Pt(1, 1), Pt(3, 1), Pt(3, 3), Pt(1, 3)])
                .unwrap()
                .crop_to_polygon(&Polygon([Pt(0, 0), Pt(0, 4), Pt(4, 4), Pt(4, 0)]).unwrap())
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
        let inner = Polygon([Pt(1, 1), Pt(3, 1), Pt(3, 3), Pt(1, 3)]).unwrap(); // 🟥
        let frame = Polygon([Pt(1, 1), Pt(3, 1), Pt(3, 3), Pt(1, 3)]).unwrap(); // 🟨
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
        let inner = Polygon([Pt(1, 1), Pt(3, 1), Pt(3, 3), Pt(1, 3)]).unwrap(); // 🟥
        let frame = Polygon([Pt(0, 0), Pt(3, 0), Pt(3, 3), Pt(0, 3)]).unwrap(); // 🟨
        assert_eq!(inner.crop_to_polygon(&frame).unwrap()[0].pts, inner.pts);

        // ⬆️ y
        // ⬜⬜⬜⬜⬜
        // ⬜🟧🟧🟧🟨
        // ⬜🟧🟧🟧🟨
        // ⬜🟧🟧🟧🟨
        // ⬜🟨🟨🟨🟨 ➡️ x
        assert_eq!(
            inner.crop_to_polygon(&(&frame + Pt(1, 0))).unwrap()[0].pts,
            inner.pts
        );

        // ⬆️ y
        // 🟨🟨🟨🟨⬜
        // 🟨🟧🟧🟧⬜
        // 🟨🟧🟧🟧⬜
        // 🟨🟧🟧🟧⬜
        // ⬜⬜⬜⬜⬜ ➡ x
        assert_eq!(
            inner.crop_to_polygon(&(&frame + Pt(0, 1))).unwrap()[0].pts,
            inner.pts
        );

        // ⬆️ y
        // ⬜🟨🟨🟨🟨
        // ⬜🟧🟧🟧🟨
        // ⬜🟧🟧🟧🟨
        // ⬜🟧🟧🟧🟨
        // ⬜⬜⬜⬜⬜ ➡ x
        assert_eq!(
            inner.crop_to_polygon(&(&frame + Pt(1, 1))).unwrap()[0].pts,
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
        let inner = Polygon([Pt(1, 1), Pt(3, 1), Pt(3, 3), Pt(1, 3)]).unwrap(); // 🟥
        let frame = Polygon([Pt(0, 0), Pt(4, 0), Pt(4, 4), Pt(0, 4)]).unwrap(); // 🟨

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
        let inner = Polygon([Pt(1, 1), Pt(4, 1), Pt(4, 4), Pt(1, 4)]).unwrap(); // 🟥
        let frame = Polygon([Pt(0, 0), Pt(3, 0), Pt(3, 3), Pt(0, 3)]).unwrap(); // 🟨
        let expected = Polygon([Pt(1, 1), Pt(3, 1), Pt(3, 3), Pt(1, 3)]).unwrap(); // 🟧

        let crops = inner.crop_to_polygon(&frame).unwrap();
        assert_eq!(crops, vec![expected.clone()]);

        let crops = frame.crop_to_polygon(&inner).unwrap();
        assert_eq!(crops, vec![expected]);
    }

    #[test]
    fn test_crop_to_polygon_two_pivots_02() {
        // ⬆️ y
        // ⬜⬜⬜⬜⬜
        // 🟨🟨🟨⬜⬜
        // 🟨🟧🟧🟥⬜
        // 🟨🟧🟧🟥⬜
        // ⬜🟥🟥🟥⬜ ➡️ x
        let inner = Polygon([Pt(1, 0), Pt(4, 0), Pt(4, 3), Pt(1, 3)]).unwrap(); // 🟥
        let frame = Polygon([Pt(0, 1), Pt(3, 1), Pt(3, 4), Pt(0, 4)]).unwrap(); // 🟨
        let expected = Polygon([Pt(1, 1), Pt(3, 1), Pt(3, 3), Pt(1, 3)]).unwrap(); // 🟧

        let crops = inner.crop_to_polygon(&frame).unwrap();
        assert_eq!(crops, vec![expected.clone()]);

        let crops = frame.crop_to_polygon(&inner).unwrap();
        assert_eq!(crops, vec![expected]);
    }

    #[test]
    fn test_crop_to_polygon_many_pivots_01() {
        // ⬆️ y
        // ⬜🟥⬜🟥⬜
        // 🟨🟧🟨🟧🟨
        // 🟨🟧🟧🟧🟨
        // 🟨🟧🟨🟧🟨
        // ⬜🟥⬜🟥⬜
        let inner = Polygon([
            Pt(1, 0),
            Pt(2, 0),
            Pt(2, 2),
            Pt(3, 2),
            Pt(3, 0),
            Pt(4, 0),
            Pt(4, 5),
            Pt(3, 5),
            Pt(3, 3),
            Pt(2, 3),
            Pt(2, 5),
            Pt(1, 5),
        ])
        .unwrap(); // 🟥
        let frame = Polygon([Pt(0, 1), Pt(5, 1), Pt(5, 4), Pt(0, 4)]).unwrap(); // 🟨
        let expected = Polygon([
            Pt(1, 1),
            Pt(2, 1),
            Pt(2, 2),
            Pt(3, 2),
            Pt(3, 1),
            Pt(4, 1),
            Pt(4, 4),
            Pt(3, 4),
            Pt(3, 3),
            Pt(2, 3),
            Pt(2, 4),
            Pt(1, 4),
        ])
        .unwrap(); // 🟧

        let crops = inner.crop_to_polygon(&frame).unwrap();
        assert_eq!(crops, vec![expected.clone()]);

        let crops = frame.crop_to_polygon(&inner).unwrap();
        assert_eq!(crops, vec![expected]);
    }

    #[test]
    fn test_crop_to_polygon_many_pivots_02() {
        // ⬆️ y
        // ⬜⬜⬜⬜⬜
        // 🟨🟧🟨🟧🟨
        // 🟨🟧🟧🟧🟨
        // 🟨🟧🟨🟧🟨
        // ⬜⬜⬜⬜⬜
        let inner = Polygon([
            Pt(1, 1),
            Pt(2, 1),
            Pt(2, 2),
            Pt(3, 2),
            Pt(3, 1),
            Pt(4, 1),
            Pt(4, 4),
            Pt(3, 4),
            Pt(3, 3),
            Pt(2, 3),
            Pt(2, 4),
            Pt(1, 4),
        ])
        .unwrap(); // 🟥
        let frame = Polygon([Pt(0, 1), Pt(5, 1), Pt(5, 4), Pt(0, 4)]).unwrap(); // 🟨
        let expected = inner.clone();
        let crops = inner.crop_to_polygon(&frame).unwrap();
        assert_eq!(crops, vec![expected.clone()]);
        let crops = frame.crop_to_polygon(&inner).unwrap();
        assert_eq!(crops, vec![expected]);
    }

    #[test]
    fn test_crop_to_polygon_many_pivots_03() {
        // ⬆️ y
        // ⬜⬜⬜⬜⬜
        // ⬜🟧🟨🟧⬜
        // ⬜🟧🟧🟧⬜
        // ⬜🟧🟨🟧⬜
        // ⬜⬜⬜⬜⬜
        let inner = Polygon([
            Pt(1, 1),
            Pt(2, 1),
            Pt(2, 2),
            Pt(3, 2),
            Pt(3, 1),
            Pt(4, 1),
            Pt(4, 4),
            Pt(3, 4),
            Pt(3, 3),
            Pt(2, 3),
            Pt(2, 4),
            Pt(1, 4),
        ])
        .unwrap(); // 🟥
        let frame = Polygon([Pt(1, 1), Pt(4, 1), Pt(4, 4), Pt(1, 4)]).unwrap(); // 🟨
        let expected = inner.clone();
        let crops = inner.crop_to_polygon(&frame).unwrap();
        assert_eq!(crops, vec![expected.clone()]);
        let crops = frame.crop_to_polygon(&inner).unwrap();
        assert_eq!(crops, vec![expected]);
    }

    #[test]
    #[ignore]
    fn test_crop_to_polygon_concavities_01() {
        // ⬆️ y
        // ⬜🟨🟨🟨⬜
        // ⬜🟨⬜🟨⬜
        // 🟥🟧🟥🟧🟥
        // 🟥🟧🟥🟧🟥
        // ⬜🟨⬜🟨⬜
        let inner = Polygon([
            Pt(1, 0),
            Pt(2, 0),
            Pt(2, 4),
            Pt(3, 4),
            Pt(3, 0),
            Pt(4, 0),
            Pt(4, 5),
            Pt(1, 5),
        ])
        .unwrap();
        let frame = Polygon([Pt(0, 1), Pt(5, 1), Pt(5, 3), Pt(0, 3)]).unwrap();
        let expected = vec![
            Polygon([Pt(1, 1), Pt(2, 1), Pt(2, 3), Pt(1, 3)]).unwrap(),
            Polygon([Pt(3, 1), Pt(4, 1), Pt(4, 3), Pt(3, 3)]).unwrap(),
        ];
        let crops = inner.crop_to_polygon(&frame).unwrap();
        assert_eq!(crops.len(), 2);
        assert_eq!(crops[0], expected[0]);
        assert_eq!(crops[1], expected[1]);
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
        let a = Pt(0, 2);
        let c = Pt(2, 2);
        let g = Pt(0, 0);
        let i = Pt(2, 0);

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
        let a = Pt(0, 2);
        let c = Pt(2, 2);
        let g = Pt(0, 0);
        let i = Pt(2, 0);
        let mut p = Polygon([a, g, i, c]).unwrap();
        assert_eq!(p.get_curve_orientation(), CurveOrientation::Positive);
        p.orient_curve();
        assert_eq!(p.get_curve_orientation(), CurveOrientation::Negative);
    }

    #[test]
    fn test_add() {
        assert_eq!(
            &Polygon([Pt(0, 0), Pt(1, 1), Pt(2, 2)]).unwrap() + Pt(1, 0),
            Polygon([Pt(1, 0), Pt(2, 1), Pt(3, 2)]).unwrap()
        );
    }

    #[test]
    fn test_sub() {
        assert_eq!(
            &Polygon([Pt(0, 0), Pt(1, 1), Pt(2, 2)]).unwrap() - Pt(1, 0),
            Polygon([Pt(-1, 0), Pt(0, 1), Pt(1, 2)]).unwrap()
        );
    }

    #[test]
    fn test_bounded() {
        //   ^
        //   |
        //   A  B  C
        //   |
        //   D  E  F
        //   |
        // --G--H--I->
        //   |
        let h = Pt(1, 0);
        let f = Pt(2, 1);
        let b = Pt(1, 2);
        let d = Pt(0, 1);
        let p = Polygon([h, f, b, d]).unwrap();
        assert_eq!(p.top_bound(), 2.0);
        assert_eq!(p.bottom_bound(), 0.0);
        assert_eq!(p.left_bound(), 0.0);
        assert_eq!(p.right_bound(), 2.0);
        assert_eq!(p.tl_bound(), Pt(0, 2));
        assert_eq!(p.bl_bound(), Pt(0, 0));
        assert_eq!(p.tr_bound(), Pt(2, 2));
        assert_eq!(p.br_bound(), Pt(2, 0));
        assert_eq!(
            p.bbox(),
            Ok(Polygon([Pt(0, 2), Pt(2, 2), Pt(2, 0), Pt(0, 0)]).unwrap())
        );
    }

    #[test]
    fn test_frame_to_segment_many_outputs() {
        // ^ y
        // |
        // 4 - - + - - + - - + - - + - - +
        // |xxxxx|xxxxx|xxxxx| .   |xxxxx|
        // |xxxxx|xxxxx|xxxxx| .   |xxxxx|
        // 3 - - + - - + - - + - - + - - +
        // |xxxxx| .   |xxxxx| .   |xxxxx|
        // |xxxxx| .   |xxxxx| .   |xxxxx|
        // 2OOOOOOOOOOOOOOOOOOOOOOOOOOOOOO
        // |xxxxx| .   |xxxxx| .   |xxxxx|
        // |xxxxx| .   |xxxxx| .   |xxxxx|
        // 1 - - + - - + - - + - - + - - +
        // |xxxxx| .   |xxxxx|xxxxx|xxxxx|
        // |xxxxx| .   |xxxxx|xxxxx|xxxxx|
        // 0 - - 1 - - 2 - - 3 - - 4 - - 5 -> x

        let frame = Polygon([
            Pt(0, 0),
            Pt(1, 0),
            Pt(1, 3),
            Pt(2, 3),
            Pt(2, 0),
            Pt(5, 0),
            Pt(5, 4),
            Pt(4, 4),
            Pt(4, 1),
            Pt(3, 1),
            Pt(3, 5),
            Pt(0, 5),
        ])
        .unwrap();
        let segment = Segment(Pt(0, 2), Pt(5, 2));
        assert_eq!(
            frame.as_frame_to_segment(&segment).unwrap(),
            vec![
                Segment(Pt(0, 2), Pt(1, 2)),
                Segment(Pt(2, 2), Pt(3, 2)),
                Segment(Pt(4, 2), Pt(5, 2)),
            ]
        );
    }

    #[test]
    fn test_frame_to_segment_crop() {
        let frame = Polygon([Pt(1, 0), Pt(2, 1), Pt(1, 2), Pt(0, 1)]).unwrap();
        assert_eq!(
            frame.as_frame_to_segment(&Segment(Pt(0, 2), Pt(2, 0))),
            Ok(vec![Segment(Pt(0.5, 1.5), Pt(1.5, 0.5))])
        );
    }
    #[test]
    fn test_frame_to_segment_crop_02() {
        let frame = Polygon([Pt(1, 0), Pt(2, 1), Pt(1, 2), Pt(0, 1)]).unwrap();
        assert_eq!(
            frame.as_frame_to_segment(&Segment(Pt(0, 0), Pt(2, 2))),
            Ok(vec![Segment(Pt(0.5, 0.5), Pt(1.5, 1.5))])
        );
    }
    #[test]
    fn test_frame_to_segment_crop_empty() {
        let frame = Polygon([Pt(1, 0), Pt(2, 1), Pt(1, 2), Pt(0, 1)]).unwrap();
        assert_eq!(
            frame.as_frame_to_segment(&Segment(Pt(0, 2), Pt(2, 2))),
            Ok(vec![])
        );
    }
    #[test]
    fn test_frame_to_segment_crop_unchanged() {
        let frame = Polygon([Pt(1, 0), Pt(2, 1), Pt(1, 2), Pt(0, 1)]).unwrap();
        assert_eq!(
            frame.as_frame_to_segment(&Segment(Pt(0, 1), Pt(2, 1))),
            Ok(vec![Segment(Pt(0, 1), Pt(2, 1))])
        );
    }
}
