#![allow(unused)]
#![allow(missing_docs)]

use crate::{interpolate::interpolate_2d_checked, segment::Intersection};

use {
    crate::{
        bounded::Bounded,
        crop::{CropToPolygonError, Croppable, PointLoc},
        curve, interpolate,
        point::{PolarPt, Pt},
        polygon::abp,
        segment::Segment,
    },
    float_cmp::{approx_eq, assert_approx_eq},
    float_ord::FloatOrd,
    std::cmp::Ordering,
    std::f64::consts::*,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CurveArc {
    pub ctr: Pt,
    pub angle_1: FloatOrd<f64>,
    pub angle_2: FloatOrd<f64>,
    pub radius: FloatOrd<f64>,
}

#[allow(clippy::upper_case_acronyms)]
enum Quadrant {
    I,
    II,
    III,
    IV,
}

fn quadrant(angle: f64) -> Option<Quadrant> {
    if angle < 0.0 {
        None
    } else if angle <= PI / 2.0 {
        Some(Quadrant::I)
    } else if angle <= PI {
        Some(Quadrant::II)
    } else if angle <= 3.0 * PI / 2.0 {
        Some(Quadrant::III)
    } else if angle <= 2.0 * PI {
        Some(Quadrant::IV)
    } else {
        None
    }
}

impl Bounded for CurveArc {
    fn right_bound(&self) -> f64 {
        self.ctr.x.0
            + self.radius.0
                * if (self.angle_1.0..self.angle_2.0).contains(&TAU) {
                    1.0
                } else {
                    std::cmp::max(
                        FloatOrd(self.angle_1.0.cos()),
                        FloatOrd(self.angle_2.0.cos()),
                    )
                    .0
                }
    }
    fn left_bound(&self) -> f64 {
        self.ctr.x.0
            + self.radius.0
                * if (self.angle_1.0..self.angle_2.0).contains(&PI) {
                    -1.0
                } else {
                    std::cmp::min(
                        FloatOrd(self.angle_1.0.cos()),
                        FloatOrd(self.angle_2.0.cos()),
                    )
                    .0
                }
    }
    fn top_bound(&self) -> f64 {
        self.ctr.y.0
            + self.radius.0
                * if (self.angle_1.0..self.angle_2.0).contains(&FRAC_PI_2) {
                    1.0
                } else {
                    std::cmp::max(
                        FloatOrd(self.angle_1.0.sin()),
                        FloatOrd(self.angle_2.0.sin()),
                    )
                    .0
                }
    }
    fn bottom_bound(&self) -> f64 {
        self.ctr.y.0
            + self.radius.0
                * if (self.angle_1.0..self.angle_2.0).contains(&(3.0 * FRAC_PI_2)) {
                    -1.0
                } else {
                    std::cmp::min(
                        FloatOrd(self.angle_1.0.sin()),
                        FloatOrd(self.angle_2.0.sin()),
                    )
                    .0
                }
    }
}

impl CurveArc {
    fn pt_i(&self) -> Pt {
        self.ctr + PolarPt(self.radius.0, self.angle_1.0)
    }
    fn pt_f(&self) -> Pt {
        self.ctr + PolarPt(self.radius.0, self.angle_2.0)
    }
}

#[allow(non_snake_case)]
pub fn CurveArc(ctr: Pt, sweep: std::ops::Range<f64>, radius: f64) -> CurveArc {
    assert!(sweep.start <= sweep.end);
    CurveArc {
        ctr,
        angle_1: FloatOrd(sweep.start),
        angle_2: FloatOrd(sweep.end),
        radius: FloatOrd(radius),
    }
}

impl std::ops::Add<Pt> for CurveArc {
    type Output = Self;
    fn add(self, rhs: Pt) -> Self::Output {
        CurveArc {
            ctr: self.ctr + rhs,
            ..self
        }
    }
}

#[derive(Debug, PartialEq, Copy, Clone)]
enum SegmentLoc {
    I,
    M(f64), // percentage of the way along
    F,
}
impl SegmentLoc {
    fn as_f64(&self) -> f64 {
        match self {
            SegmentLoc::I => 0.0,
            SegmentLoc::M(f) => *f,
            SegmentLoc::F => 1.0,
        }
    }
    fn from_f64(f: f64) -> Option<SegmentLoc> {
        match f {
            f if approx_eq!(f64, f, 0.0) => Some(SegmentLoc::I),
            f if approx_eq!(f64, f, 1.0) => Some(SegmentLoc::F),
            f if !(0.0..=1.0).contains(&f) || f.is_nan() || f.is_infinite() => None,
            f => Some(SegmentLoc::M(f)),
        }
    }
}

#[derive(Debug, PartialEq, Copy, Clone, PartialOrd)]
enum CurveLoc {
    I,
    M(f64), // percentage of the way along
    F,
}
impl CurveLoc {
    fn as_f64(&self) -> f64 {
        match self {
            CurveLoc::I => 0.0,
            CurveLoc::M(f) => *f,
            CurveLoc::F => 1.0,
        }
    }
    fn from_f64(f: f64) -> Option<CurveLoc> {
        match f {
            f if approx_eq!(f64, f, 0.0) => Some(CurveLoc::I),
            f if approx_eq!(f64, f, 1.0) => Some(CurveLoc::F),
            f if !(0.0..=1.0).contains(&f) || f.is_nan() || f.is_infinite() => None,
            f => Some(CurveLoc::M(f)),
        }
    }
}

#[derive(Debug, PartialEq, Copy, Clone)]
struct PtLoc(Pt, SegmentLoc, CurveLoc);

#[derive(Debug, PartialEq)]
enum IntersectionResult {
    None,
    One(PtLoc),
    Two(PtLoc, PtLoc),
}

/// how to find intersection of segment and curve.
fn intersections_of_line_and_curvearc(
    segment: &Segment,
    curve_arc: &CurveArc,
) -> IntersectionResult {
    let (x_0, y_0) = (curve_arc.ctr.x.0, curve_arc.ctr.y.0);
    let (x_1, y_1) = (segment.i.x.0, segment.i.y.0);
    let (x_2, y_2) = (segment.f.x.0, segment.f.y.0);
    let r = curve_arc.radius.0;

    // d is distance to line. (see (14) in
    // https://mathworld.wolfram.com/Point-LineDistance2-Dimensional.html)
    let d: f64 = ((x_2 - x_1) * (y_1 - y_0) - (x_1 - x_0) * (y_2 - y_1)).abs()
        / ((x_2 - x_1).powi(2) + (y_2 - y_1).powi(2)).sqrt();

    match FloatOrd(d).cmp(&curve_arc.radius) {
        Ordering::Greater => IntersectionResult::None,
        Ordering::Equal => {
            let isxn =
                curve_arc.ctr + PolarPt(curve_arc.radius.0, segment.slope().atan() + FRAC_PI_2);

            let segment_loc = SegmentLoc::from_f64(
                interpolate_2d_checked(segment.i, segment.f, isxn).expect("interpolation failed"),
            );

            let curve_loc = CurveLoc::from_f64(
                abp(&curve_arc.ctr, &isxn, &curve_arc.pt_i())
                    / abp(&curve_arc.ctr, &curve_arc.pt_f(), &curve_arc.pt_i()),
            );

            match (segment_loc, curve_loc) {
                (Some(sl), Some(cl)) => IntersectionResult::One(PtLoc(isxn, sl, cl)),
                _ => IntersectionResult::None,
            }
        }
        Ordering::Less => {
            // also an option if d==0 is that the curve is centered _on_ the line.

            // possibly one intersection, if the curve crosses the line.
            // or, possibly two intersections.

            // calculate two possible intersections.
            let (isxn_1, isxn_2) = {
                // https://math.stackexchange.com/questions/228841/how-do-i-calculate-the-intersections-of-a-straight-line-and-a-circle
                //
                // here's how we do that: two equations:
                // (x - x_0)^2 + (y - y_0)^2 = r^2
                // L_A*x + L_B*y + L_C = 0
                // . where
                // . . L_A = y_2 - y_1
                // . . L_B = x_1 - x_2
                // . . L_C = x_2 * y_1 - x_1 * y^2
                let l_a = y_2 - y_1;
                let l_b = x_1 - x_2;

                let l_c = x_2 * y_1 - x_1 * y_2;

                // we can rearrange this into a big quadratic eqn:
                // ax^2 + bx + c = 0
                // . where
                // . . a = L_A^2 + L_B^2
                // . . b = 2 * L_A * L_C + 2 * L_A * L_B * y_0 - 2 * L_B^2 * x_0
                // . . c = L_C^2 + 2 * L_B * L_C * y_0 - L_B^2 * (r^2 - x_0^2 - y_0^2)
                let c_a = l_a.powi(2) + l_b.powi(2);
                let c_b = 2.0 * l_a * l_c + 2.0 * l_a * l_b * y_0 - 2.0 * l_b.powi(2) * x_0;
                let c_c = l_c.powi(2) + 2.0 * l_b * l_c * y_0
                    - l_b.powi(2) * (r.powi(2) - x_0.powi(2) - y_0.powi(2));

                // and finally a discriminant d, where
                // d = b^2 - 4 * a * c
                let c_d = c_b.powi(2) - 4.0 * c_a * c_c;

                // so,
                // . x = -(b +/- sqrt(d)) / (2 * a)
                // . y = - (A * x + C) / B
                // stupid matching array pattern... ugh
                match [true, false]
                    .into_iter()
                    .map(|is_neg| {
                        let x = (-1.0 * c_b + if is_neg { -1.0 } else { 1.0 } * (c_d).sqrt())
                            / (2.0 * c_a);

                        let y = {
                            let y_top = -1.0 * (l_a * x + l_c);
                            let y_bottom = l_b;

                            if approx_eq!(f64, y_top, 0.0) && approx_eq!(f64, y_bottom, 0.0) {
                                1.0
                            } else {
                                y_top / y_bottom
                            }
                        };

                        Pt(x, y)
                    })
                    .collect::<Vec<_>>()[..]
                {
                    [i, j] => (i, j),
                    _ => panic!(""),
                }
            };

            // both good options! But only one will have an segment and
            // curve interpolation value of between 0 and 1.

            // here pac == percent along curve, and pas == percent along segment.
            let full_curve_angle = curve_arc.angle_2.0 - curve_arc.angle_1.0;

            /// percent_along_segment to option<segment_location>, if valid
            fn pas_to_sl<E>(pas_result: Result<f64, E>) -> Option<SegmentLoc> {
                pas_result.ok().and_then(SegmentLoc::from_f64)
            }

            let pac1 = {
                let mut partial_angle = abp(&curve_arc.ctr, &curve_arc.pt_i(), &isxn_1);
                partial_angle += TAU;
                partial_angle %= TAU;
                partial_angle / full_curve_angle
            };

            let pac2 = {
                let mut partial_angle = abp(&curve_arc.ctr, &curve_arc.pt_i(), &isxn_2);
                partial_angle += TAU;
                partial_angle %= TAU;
                partial_angle / full_curve_angle
            };

            match (
                pas_to_sl(interpolate_2d_checked(segment.i, segment.f, isxn_1)), // sl1
                CurveLoc::from_f64(pac1),                                        // cl1
                pas_to_sl(interpolate_2d_checked(segment.i, segment.f, isxn_2)), // sl2
                CurveLoc::from_f64(pac2),                                        // cl2
            ) {
                (Some(sl1), Some(cl1), Some(sl2), Some(cl2)) => {
                    if sl1 == sl2 && cl1 == cl2 {
                        IntersectionResult::One(PtLoc(isxn_1, sl1, cl1))
                    } else {
                        IntersectionResult::Two(PtLoc(isxn_1, sl1, cl1), PtLoc(isxn_2, sl2, cl2))
                    }
                }
                (Some(sl1), Some(cl1), _, _) => IntersectionResult::One(PtLoc(isxn_1, sl1, cl1)),
                (_, _, Some(sl2), Some(cl2)) => IntersectionResult::One(PtLoc(isxn_2, sl2, cl2)),
                _ => {
                    // is this right?
                    IntersectionResult::None
                }
            }
        }
    }
}

impl Croppable for CurveArc {
    type Output = CurveArc;
    fn crop_to(
        &self,
        frame: &crate::polygon::Polygon,
    ) -> Result<Vec<Self::Output>, CropToPolygonError>
    where
        Self: Sized,
    {
        let mut isxns: Vec<PtLoc> = vec![];
        for frame_segment in frame.to_segments() {
            let discovered = match intersections_of_line_and_curvearc(&frame_segment, self) {
                IntersectionResult::None => {
                    vec![]
                }
                IntersectionResult::One(pl) => vec![pl],
                IntersectionResult::Two(ref pl1 @ PtLoc(_, _, cl1), ref pl2 @ PtLoc(_, _, cl2)) => {
                    // sort by cl
                    if cl1 < cl2 {
                        vec![*pl1, *pl2]
                    } else {
                        vec![*pl2, *pl1]
                    }
                }
            };
            isxns.extend(discovered);
        }

        // either the curve is totally within and has no overlaps, or is totally
        // without and has no overlaps.
        if isxns.is_empty() {
            //
            let contains_i = frame.contains_pt(&self.pt_i()).expect("contains pt");
            let contains_f = frame.contains_pt(&self.pt_f()).expect("contains pt");
            match (contains_i, contains_f) {
                (
                    PointLoc::Inside | PointLoc::OnSegment(_) | PointLoc::OnPoint(_),
                    PointLoc::Inside | PointLoc::OnSegment(_) | PointLoc::OnPoint(_),
                ) => {
                    return Ok(vec![*self]);
                }
                _ => {
                    // do nothing
                }
            }
        }

        let mut isxns_angles: Vec<FloatOrd<f64>> = isxns
            .into_iter()
            .map(|PtLoc(_, _, cl)| {
                FloatOrd(self.angle_1.0 + (self.angle_2.0 - self.angle_1.0) * cl.as_f64())
            })
            .collect::<Vec<_>>();
        isxns_angles.sort();

        let mut r = vec![];

        for (a1, a2) in isxns_angles.iter().zip(
            isxns_angles
                .iter()
                .map(|x| FloatOrd(x.0 + TAU))
                .cycle()
                .skip(1),
        ) {
            // THIS IS WRONG !!!!!
            // THIS IS WRONG !!!!!
            // THIS IS WRONG !!!!!
            // THIS IS WRONG !!!!!
            // THIS IS WRONG !!!!!
            // THIS IS WRONG !!!!!
            let midpt = self.ctr + PolarPt(self.radius.0, (a1.0 + a2.0) / 2.0);
            match frame.contains_pt(&midpt).expect("contains pt") {
                PointLoc::Outside => {
                    // do nothing
                }
                _ => {
                    r.push(CurveArc(self.ctr, a1.0..a2.0, self.radius.0));
                }
            }
        }

        Ok(r)
    }
}

#[cfg(test)]
mod test {
    use {
        super::*,
        crate::{polygon::Polygon, segment::Segment},
        assert_matches::assert_matches,
        std::f64::consts::*,
    };

    #[test]
    fn test_curve_zero_intersections() {
        assert_matches!(
            intersections_of_line_and_curvearc(
                &Segment(Pt(0.0, 0.0), Pt(3.0, 0.0)),
                &CurveArc(Pt(1.0, 1.0), 0.0..PI, 0.5)
            ),
            IntersectionResult::None
        );
    }

    #[test]
    fn test_curve_one_intersection_tangent() {
        let segment = Segment(Pt(0.0, 0.0), Pt(2.0, 0.0));

        for (curve_arc, expected_segment_loc, expected_curve_loc) in [
            // segment m, curve m
            (
                CurveArc(Pt(1.0, 1.0), 0.0..PI, 1.0),
                SegmentLoc::M(0.5),
                CurveLoc::M(0.5),
            ),
            // segment m, curve f
            (
                CurveArc(Pt(1.0, 1.0), -1.0 * FRAC_PI_2..FRAC_PI_2, 1.0),
                SegmentLoc::M(0.5),
                CurveLoc::F,
            ),
            // segment m, curve i
            (
                CurveArc(Pt(1.0, 1.0), FRAC_PI_2..3.0 * FRAC_PI_2, 1.0),
                SegmentLoc::M(0.5),
                CurveLoc::I,
            ),
            // segment i, curve f,
            (
                CurveArc(Pt(0.0, 1.0), -1.0 * FRAC_PI_2..FRAC_PI_2, 1.0),
                SegmentLoc::I,
                CurveLoc::F,
            ),
            // segment i, curve i
            (
                CurveArc(Pt(0.0, 1.0), FRAC_PI_2..3.0 * FRAC_PI_2, 1.0),
                SegmentLoc::I,
                CurveLoc::I,
            ),
            // segment i, curve m,
            (
                CurveArc(Pt(0.0, 1.0), 0.0..PI, 1.0),
                SegmentLoc::I,
                CurveLoc::M(0.5),
            ),
            // segment f, curve m,
            (
                CurveArc(Pt(2.0, 1.0), 0.0..PI, 1.0),
                SegmentLoc::F,
                CurveLoc::M(0.5),
            ),
            // segment f, curve i,
            (
                CurveArc(Pt(2.0, 1.0), FRAC_PI_2..3.0 * FRAC_PI_2, 1.0),
                SegmentLoc::F,
                CurveLoc::I,
            ),
            // segment f, curve f,
            (
                CurveArc(Pt(2.0, 1.0), -1.0 * FRAC_PI_2..FRAC_PI_2, 1.0),
                SegmentLoc::F,
                CurveLoc::F,
            ),
        ] {
            let (sl, cl) = assert_matches!(
                intersections_of_line_and_curvearc(&segment, &curve_arc),
                IntersectionResult::One(PtLoc(_, sl, cl)) => (sl, cl)
            );
            assert_eq!(sl, expected_segment_loc);
            assert_eq!(cl, expected_curve_loc);
        }
    }

    #[test]
    fn test_curve_one_intersection_crossing() {
        for (segment, curve_arc, (e_pl, e_sl, e_cl)) in [
            (
                Segment(Pt(0.0, 0.0), Pt(2.0, 0.0)),
                CurveArc(Pt(1.0, 0.0), FRAC_PI_2..3.0 * FRAC_PI_2, 0.5),
                (Pt(0.50, 0.0), SegmentLoc::M(0.25), CurveLoc::M(0.5)),
            ),
            (
                Segment(Pt(2.0, 0.0), Pt(2.0, 2.0)),
                CurveArc(Pt(2.0, 0.0), 0.0..3.0 * FRAC_PI_2, 1.0),
                (Pt(2.0, 1.0), SegmentLoc::M(0.5), CurveLoc::M(1.0 / 3.0)),
            ),
        ] {
            let (pl, sl, cl) = assert_matches!(
                intersections_of_line_and_curvearc(&segment, &curve_arc),
                IntersectionResult::One(PtLoc(pl, sl, cl)) => (pl, sl, cl)
            );
            assert_eq!(pl, e_pl);
            assert_eq!(sl, e_sl);
            assert_eq!(cl, e_cl);
        }
    }

    #[test]
    fn test_curve_two_intersections() {
        let segment = Segment(Pt(0.0, 0.0), Pt(3.0, 0.0));

        for (curve_arc, e_pl1, e_pl2) in [
            // segment m curve i, segment m curve f
            (
                CurveArc(Pt(1.5, 0.0), 0.0..PI, 0.5),
                PtLoc(Pt(1.0, 0.0), SegmentLoc::M(1.0 / 3.0), CurveLoc::F),
                PtLoc(Pt(2.0, 0.0), SegmentLoc::M(2.0 / 3.0), CurveLoc::I),
            ),
            // not sure if correct. TODO(write other tests.)
        ] {
            let (pl1, pl2) = assert_matches!(
                intersections_of_line_and_curvearc(&segment, &curve_arc),
                IntersectionResult::Two(pl1, pl2) => (pl1, pl2)
            );
            assert_eq!(pl1, e_pl1);
            assert_eq!(pl2, e_pl2);
        }
    }

    #[test]
    fn test_curvearc_crop() {
        for (rect, ca, e_cas) in [
            (
                Polygon([Pt(0.0, 0.0), Pt(2.0, 0.0), Pt(2.0, 2.0), Pt(0.0, 2.0)]).unwrap(),
                CurveArc(Pt(2.0, 0.0), 0.0..3.0 * FRAC_PI_2, 1.0),
                vec![CurveArc(Pt(2.0, 0.0), FRAC_PI_2..PI, 1.0)],
            ),
            // (
            //     // no intersections
            //     Polygon([Pt(0.0, 0.0), Pt(2.0, 0.0), Pt(2.0, 2.0), Pt(0.0, 2.0)]).unwrap(),
            //     CurveArc(Pt(1.0, 1.0), 0.0..TAU, 0.5),
            //     vec![CurveArc(Pt(1.0, 1.0), 0.0..TAU, 0.5)],
            // ),
            // (
            //     // four intersections but nothing serious
            //     Polygon([Pt(0.0, 0.0), Pt(2.0, 0.0), Pt(2.0, 2.0), Pt(0.0, 2.0)]).unwrap(),
            //     CurveArc(Pt(1.0, 1.0), 0.0..TAU, 1.0),
            //     vec![CurveArc(Pt(1.0, 1.0), 0.0..TAU, 1.0)],
            // ),
            // (
            //     // the same as above, four intersections but nothing serious.
            //     Polygon([
            //         Pt(100.00, 100.00),
            //         Pt(100.00, 600.00),
            //         Pt(750.00, 600.00),
            //         Pt(750.00, 100.00),
            //     ])
            //     .unwrap(),
            //     CurveArc(Pt(425.0, 350.0), 0.0..TAU, 250.0),
            //     vec![CurveArc(Pt(425.0, 350.0), 0.0..TAU, 250.0)],
            // ),
            // // hard case, splits into four (?)
            // (
            //     Polygon([
            //         Pt(100.00, 100.00),
            //         Pt(100.00, 600.00),
            //         Pt(750.00, 600.00),
            //         Pt(750.00, 100.00),
            //     ])
            //     .unwrap(),
            //     CurveArc(Pt(425.0, 350.0), 0.0..TAU, 330.0),
            //     vec![
            //         // nothing
            //     ],
            // ),
        ] {
            let cas = ca.crop_to(&rect).unwrap();
            assert_eq!(cas, e_cas);
        }
    }
}
