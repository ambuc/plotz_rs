use {
    crate::{
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
    #[error("could not construct a polygon, we cycled. check the logs please")]
    CycleError,
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

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
struct Isxn {
    frame_segment_idx: usize,
    self_segment_idx: usize,
    intersection: Intersection,
}
impl Isxn {
    pub fn pt_given_self_segs(&self, self_segs: &[(usize, Segment)]) -> Pt {
        let (_, seg) = self_segs[self.self_segment_idx];
        interpolate::extrapolate_2d(seg.i, seg.f, self.intersection.percent_along_self.0)
    }
}

#[derive(Debug, Copy, Clone)]
enum On {
    OnSelf,
    OnFrame,
}
impl On {
    pub fn flip(&self) -> On {
        match self {
            On::OnSelf => On::OnFrame,
            On::OnFrame => On::OnSelf,
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
    self_pts: &'a Vec<(usize, &'a Pt)>,
    #[derivative(Debug = "ignore")]
    self_pts_len: &'a usize,
    #[derivative(Debug = "ignore")]
    frame_pts: &'a Vec<(usize, &'a Pt)>,
    #[derivative(Debug = "ignore")]
    frame_pts_len: &'a usize,
    #[derivative(Debug = "ignore")]
    self_segments: &'a Vec<(usize, Segment)>,
}
impl<'a> Cursor<'a> {
    fn pt(&self) -> Pt {
        match &self.position {
            Either::Left(one_polygon) => match one_polygon.on_polygon {
                On::OnSelf => *self.self_pts[one_polygon.at_point_index].1,
                On::OnFrame => *self.frame_pts[one_polygon.at_point_index].1,
            },
            Either::Right(isxn) => isxn.pt_given_self_segs(self.self_segments),
        }
    }
    fn pts_len(&self, on: On) -> usize {
        match on {
            On::OnSelf => *self.self_pts_len,
            On::OnFrame => *self.frame_pts_len,
        }
    }
    fn march_to_next_point(&mut self) {
        let v = (match self.position {
            Either::Left(one_polygon) => one_polygon.at_point_index,
            Either::Right(isxn) => match self.facing_along {
                On::OnSelf => isxn.self_segment_idx,
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
            On::OnSelf => next_isxn.self_segment_idx,
        };
        self.position = new_position;
        self.facing_along = new_facing_along;
        self.facing_along_segment_idx = new_facing_along_segment_idx;
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
        let frame_segments: Vec<_> = frame.to_segments().into_iter().enumerate().collect();

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

            let all_frame_points_in_self = all(&frame_pts_in_self, |(_idx, isxn)| {
                !matches!(isxn, PointLoc::Outside)
            });
            let all_self_points_in_frame = all(&self_pts_in_frame, |(_idx, isxn)| {
                !matches!(isxn, PointLoc::Outside)
            });

            // Then either all of the frame points are inside self,
            if all_frame_points_in_self {
                // in which case we ought to return the frame unchanged,
                return Ok(vec![frame.clone()]);
                // or all of the self points are inside frame,
            } else if all_self_points_in_frame {
                // in which case we ought to return self unchanged.
                return Ok(vec![self.clone()]);
            }
        }

        let self_pts: Vec<_> = self.pts.iter().enumerate().collect();
        let frame_pts: Vec<_> = frame.pts.iter().enumerate().collect();

        let mut resultant_polygons: Vec<Polygon> = vec![];
        let mut resultant_pts: Vec<Pt> = vec![];

        let mut visited_pts = HashSet::<Pt>::new();

        let mut curr = Cursor {
            position: Either::Left(OnePolygon {
                on_polygon: On::OnSelf,
                at_point_index: 0,
            }),
            facing_along: On::OnSelf,
            facing_along_segment_idx: 0_usize,
            //
            self_pts: &self_pts,
            self_pts_len: &self_pts.len(),
            frame_pts: &frame_pts,
            frame_pts_len: &frame_pts.len(),
            self_segments: &self_segments,
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
                        On::OnSelf => isxn.self_segment_idx,
                        On::OnFrame => isxn.frame_segment_idx,
                    }) == curr.facing_along_segment_idx
                })
                // then collect them.
                .collect();

            relevant_isxns.sort_by(|a: &Isxn, b: &Isxn| match &curr.facing_along {
                On::OnSelf => a
                    .intersection
                    .percent_along_self
                    .partial_cmp(&b.intersection.percent_along_self)
                    .unwrap(),
                On::OnFrame => a
                    .intersection
                    .percent_along_other
                    .partial_cmp(&b.intersection.percent_along_other)
                    .unwrap(),
            });

            match curr.position {
                Either::Left(_) => {
                    let (_drained, v) =
                        relevant_isxns
                            .into_iter()
                            .partition(|isxn| match curr.facing_along {
                                On::OnSelf => isxn.intersection.percent_along_self.0 == 0.0,
                                On::OnFrame => isxn.intersection.percent_along_other.0 == 0.0,
                            });
                    relevant_isxns = v;
                }
                Either::Right(this_isxn) => {
                    let (_drained, v) =
                        relevant_isxns
                            .into_iter()
                            .partition(|isxn| match curr.facing_along {
                                On::OnSelf => {
                                    isxn.intersection.percent_along_self
                                        <= this_isxn.intersection.percent_along_self
                                }
                                On::OnFrame => {
                                    isxn.intersection.percent_along_other
                                        <= this_isxn.intersection.percent_along_other
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
        assert_eq!(crops, vec![expected.clone()]);
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
        assert_eq!(crops, vec![expected.clone()]);
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
        assert_eq!(crops, vec![expected.clone()]);
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
        assert_eq!(crops, vec![expected.clone()]);
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
        assert_eq!(crops, vec![expected.clone()]);
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
}
