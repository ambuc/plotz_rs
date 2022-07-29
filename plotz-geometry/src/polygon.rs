use crate::interpolate;

use {
    crate::{
        point::Pt,
        segment::{Contains, Intersection, IntersectionOutcome, Segment},
    },
    either::Either,
    float_cmp::approx_eq,
    itertools::{all, iproduct, zip},
    std::{
        cmp::{Eq, PartialEq},
        fmt::Debug,
        ops::Add,
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
pub struct Polygon {
    pub pts: Vec<Pt>,
    pub kind: PolygonKind,
}

impl PartialEq for Polygon {
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

#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum PolygonConstructorError {
    #[error("two or fewer points")]
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
struct IsxnOutcome {
    frame_segment_idx: usize,
    self_segment_idx: usize,
    outcome: IntersectionOutcome,
}
impl IsxnOutcome {
    fn to_isxn(&self) -> Option<Isxn> {
        match self.outcome {
            IntersectionOutcome::Yes(i) => Some(Isxn {
                frame_segment_idx: self.frame_segment_idx,
                self_segment_idx: self.self_segment_idx,
                intersection: i,
            }),
            _ => None,
        }
    }
}

#[derive(Debug, Copy, Clone)]
struct Isxn {
    frame_segment_idx: usize,
    self_segment_idx: usize,
    intersection: Intersection,
}
impl Isxn {
    pub fn to_pt_given_self_segs(&self, self_segs: &[(usize, Segment)]) -> Pt {
        let (_, seg) = self_segs[self.self_segment_idx];
        interpolate::extrapolate_2d(seg.i, seg.f, self.intersection.percent_along_self)
    }
}

#[derive(Debug, Copy, Clone)]
enum On {
    OnSelf,
    OnFrame,
}

#[derive(Debug, Copy, Clone)]
struct OnePolygon {
    on_polygon: On,
    at_point_index: usize,
}

#[derive(Debug)]
struct Cursor<'a> {
    // current position
    position: Either<OnePolygon, Isxn>,
    facing_along: (On, usize), // segment idx
    // context
    self_pts: &'a Vec<(usize, &'a Pt)>,
    self_pts_len: &'a usize,
    frame_pts: &'a Vec<(usize, &'a Pt)>,
    frame_pts_len: &'a usize,
    self_segments: &'a Vec<(usize, Segment)>,
    self_segments_len: &'a usize,
    frame_segments: &'a Vec<(usize, Segment)>,
    frame_segments_len: &'a usize,
}
impl<'a> Cursor<'a> {
    fn pt(&self) -> Pt {
        match &self.position {
            Either::Left(one_polygon) => match one_polygon.on_polygon {
                On::OnSelf => *self.self_pts[one_polygon.at_point_index].1,
                On::OnFrame => *self.frame_pts[one_polygon.at_point_index].1,
            },
            Either::Right(isxn) => isxn.to_pt_given_self_segs(&self.self_segments),
        }
    }
    fn march_to_next_point(&mut self) {
        match (&mut self.position, self.facing_along) {
            (Either::Left(ref mut one_polygon), (on, _)) => {
                println!("L");
                one_polygon.at_point_index += 1;
                if one_polygon.at_point_index
                    >= (match on {
                        On::OnSelf => *self.self_pts_len,
                        On::OnFrame => *self.frame_pts_len,
                    })
                {
                    one_polygon.at_point_index = 0;
                }
            }
            (Either::Right(isxn), (on, _)) => {
                println!("R, isxn {:?}", isxn);
                let mut new_index = match on {
                    On::OnSelf => isxn.self_segment_idx,
                    On::OnFrame => isxn.frame_segment_idx,
                } + 1;
                if new_index
                    >= match on {
                        On::OnSelf => *self.self_pts_len,
                        On::OnFrame => *self.frame_pts_len,
                    }
                {
                    new_index = 0;
                }

                self.position = Either::Left(OnePolygon {
                    on_polygon: on,
                    at_point_index: new_index,
                });
                self.facing_along.1 = new_index;
            }
        }
    }
}

impl Polygon {
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

        Ok(match approx_eq!(f64, theta, 0_f64) {
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

    // NB: Polygons must be closed and positively oriented.
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

        let self_segments: Vec<_> = self.to_segments().into_iter().enumerate().collect();
        let self_segments_len: usize = self_segments.len();
        let frame_segments: Vec<_> = frame.to_segments().into_iter().enumerate().collect();
        let frame_segments_len: usize = frame_segments.len();

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

        let isxn_outcomes: Vec<IsxnOutcome> = iproduct!(&self_segments, &frame_segments)
            .filter_map(|((self_idx, s_seg), (frame_idx, f_seg))| {
                s_seg.intersects(f_seg).map(|outcome| IsxnOutcome {
                    frame_segment_idx: *frame_idx,
                    self_segment_idx: *self_idx,
                    outcome,
                })
            })
            .collect();

        // If there are no intersections,
        if isxn_outcomes.is_empty() {
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

        let mut resultant_polygons: Vec<Polygon> = vec![];
        let mut resultant_pts: Vec<Pt> = vec![];

        assert!(!self_pts_in_frame.is_empty());

        let mut curr = Cursor {
            position: Either::Left(OnePolygon {
                on_polygon: On::OnSelf,
                at_point_index: 0,
            }),
            facing_along: (On::OnSelf, /*pt_idx*/ 0),
            self_pts: &self_pts,
            self_pts_len: &self_pts_len,
            frame_pts: &frame_pts,
            frame_pts_len: &frame_pts_len,
            self_segments: &self_segments,
            self_segments_len: &self_segments_len,
            frame_segments: &frame_segments,
            frame_segments_len: &frame_segments_len,
        };

        'outer: loop {
            let curr_pt: Pt = curr.pt();

            // If we've made a cycle,
            if let Some(pt) = resultant_pts.get(0) {
                if *pt == curr_pt {
                    // then break out of it.
                    println!("Detected a cycle, breaking out.");
                    break 'outer;
                }
            }

            resultant_pts.push(curr_pt);
            println!("Current point: {:?}", curr_pt);

            match frame.contains_pt(&curr_pt)? {
                PointLoc::Outside => {
                    println!("\tThe current point is outside of the frame.");
                    unimplemented!("{curr_pt:#?}");
                }
                PointLoc::Inside | PointLoc::OnPoint(_) | PointLoc::OnSegment(_) => {
                    println!(
                        "\tThe current point is inside of the frame (or on a point or segment)."
                    );
                    // If there are any intersections which
                    let mut relevant_isxns: Vec<Isxn> = isxn_outcomes
                        .iter()
                        .filter_map(|isxn_outcome| isxn_outcome.to_isxn())
                        // (a) intersect our line (that's line[curr_idx] from pt[curr_idx] to pt[curr_idx]+1)
                        .filter(|isxn| {
                            (match curr.facing_along {
                                (On::OnSelf, _) => isxn.self_segment_idx,
                                (On::OnFrame, _) => isxn.frame_segment_idx,
                            }) == curr.facing_along.1
                        })
                        // (b) is a genuine intersection which does not intersect at a point,
                        .filter(|isxn| !isxn.intersection.on_points_of_either_polygon())
                        // then collect them.
                        .collect();

                    if !relevant_isxns.is_empty() {
                        println!("\tFound some intersections marching forwards.");
                        // must take the list of relevant isxns and sort them by
                        // distance along current segment, and discard the ones
                        // behind self.
                        // TODO
                        relevant_isxns.sort_by(|a: &Isxn, b: &Isxn| match &curr.facing_along {
                            (On::OnSelf, _) => a
                                .intersection
                                .percent_along_self
                                .partial_cmp(&b.intersection.percent_along_self)
                                .unwrap(),
                            (On::OnFrame, _) => a
                                .intersection
                                .percent_along_other
                                .partial_cmp(&b.intersection.percent_along_other)
                                .unwrap(),
                        });
                        match curr.position {
                            Either::Left(one_polygon) => {
                                println!("\tSince our current point is on a polygon,");
                                // Since we're currently at a point on just one
                                // polygon, we should march towards the very first
                                // relevant_isxn.
                                println!("relevant isxns: {:?}", relevant_isxns);
                                let next_isxn: &Isxn = relevant_isxns
                                    .get(0)
                                    .expect("I thought you said it wasn't empty?");
                                let _next_pt = next_isxn.to_pt_given_self_segs(&curr.self_segments);
                                // add to resultant pts? or will  this happen enxt time around?

                                println!("\tSetting new next point based on pivot, not marching.");
                                println!("\t\tfound isxn {:?}", next_isxn);
                                println!("\t\tcurrently at {:?}", curr_pt);
                                println!("\t\tcurrently facing {:?}", curr.facing_along);
                                let new_position: Either<_, Isxn> = Either::Right(*next_isxn);
                                let new_facing_along = match one_polygon.on_polygon {
                                    On::OnSelf => (On::OnFrame, next_isxn.frame_segment_idx),
                                    On::OnFrame => (On::OnSelf, next_isxn.self_segment_idx),
                                };
                                // if next_isxn.intersection.on_points_of_either_polygon() {
                                //     println!("\t\tbut since we are at a corner, gotta bump the index.");
                                //
                                //     let (_, mut idx) = new_facing_along;
                                //     idx += 1;
                                // }

                                // TODO ambuc, we need to bump the segment index IF this point is an corner.

                                curr.position = new_position;
                                curr.facing_along = new_facing_along;
                                println!("\t\tmarched to {:?}", curr.pt());
                                println!("\t\tnow marching along {:?}", curr.facing_along);
                            }
                            Either::Right(this_isxn) => {
                                println!("\tsince our current point is on an intersection,");
                                // if we're currently  at an intersection, we
                                // should march towards the next relevant_ixsn which is along our current along, but which is ahead of us.
                                relevant_isxns.drain_filter(|isxn| match curr.facing_along {
                                    (On::OnSelf, _) => {
                                        isxn.intersection.percent_along_self
                                            <= this_isxn.intersection.percent_along_self
                                    }
                                    (On::OnFrame, _) => {
                                        isxn.intersection.percent_along_other
                                            <= this_isxn.intersection.percent_along_other
                                    }
                                });
                                if let Some(next_isxn) = relevant_isxns.get(0) {
                                    let _next_pt =
                                        next_isxn.to_pt_given_self_segs(&curr.self_segments);

                                    let new_position: Either<_, Isxn> = Either::Right(*next_isxn);
                                    let new_facing_along = match curr.facing_along {
                                        (On::OnSelf, _) => {
                                            (On::OnFrame, next_isxn.frame_segment_idx)
                                        }
                                        (On::OnFrame, _) => {
                                            (On::OnSelf, next_isxn.self_segment_idx)
                                        }
                                    };
                                    curr.position = new_position;
                                    curr.facing_along = new_facing_along;
                                }
                            }
                        }
                    }

                    if relevant_isxns.is_empty() {
                        println!("\tSince no intersections, marching fwd.");
                        println!("\t\tcurrently at {:?}", curr.pt());
                        println!("\t\tcurrently facing {:?}", curr.facing_along);
                        // else if there aren't any intersections, then pt[curr_idx] runs
                        // along line[curr_idx] to pt[curr_idx]+1 uninterrupted, and
                        // pt[curr_idx]+1 is also in frame.
                        // No action necessary, so increment |curr| and loop.
                        curr.march_to_next_point();
                        println!("\t\tnow at {:?}", curr.pt());
                        println!("\t\tnow facing {:?}", curr.facing_along);
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
impl Add<Pt> for &Polygon {
    type Output = Polygon;
    fn add(self, rhs: Pt) -> Self::Output {
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
        // let crops = frame.crop_to_polygon(&inner).unwrap();
        // assert_eq!(crops, vec![expected.clone()]);
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
