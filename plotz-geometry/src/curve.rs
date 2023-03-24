#![allow(unused)]
#![allow(missing_docs)]

use crate::interpolate::interpolate_2d_checked;

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

#[derive(Debug, PartialEq)]
enum SegmentLoc {
    I,
    M(f64), // percentage of the way along
    F,
}

#[derive(Debug, PartialEq)]
enum CurveLoc {
    I,
    M(f64), // percentage of the way along
    F,
}

#[derive(Debug, PartialEq)]
struct PtLoc(SegmentLoc, CurveLoc);

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
    // d is distance to line. (see (14) in
    // https://mathworld.wolfram.com/Point-LineDistance2-Dimensional.html)
    let x0 = curve_arc.ctr.x.0;
    let x1 = segment.i.x.0;
    let x2 = segment.f.x.0;
    let y0 = curve_arc.ctr.y.0;
    let y1 = segment.i.y.0;
    let y2 = segment.f.y.0;
    let d: f64 = ((x2 - x1) * (y1 - y0) - (x1 - x0) * (y2 - y1)).abs()
        / ((x2 - x1).powi(2) + (y2 - y1).powi(2)).sqrt();

    match FloatOrd(d).cmp(&curve_arc.radius) {
        Ordering::Greater => IntersectionResult::None,
        Ordering::Equal => {
            let isxn =
                curve_arc.ctr + PolarPt(curve_arc.radius.0, segment.slope().atan() + FRAC_PI_2);

            let segment_loc = {
                let percent_along = interpolate_2d_checked(segment.i, segment.f, isxn)
                    .expect("interpolation failed");

                if approx_eq!(f64, percent_along, 0.0) {
                    SegmentLoc::I
                } else if approx_eq!(f64, percent_along, 1.0) {
                    SegmentLoc::F
                } else {
                    SegmentLoc::M(percent_along)
                }
            };

            let curve_loc = {
                let percent_along = abp(&curve_arc.ctr, &isxn, &curve_arc.pt_i())
                    / abp(&curve_arc.ctr, &curve_arc.pt_f(), &curve_arc.pt_i());

                if approx_eq!(f64, percent_along, 0.0) {
                    CurveLoc::I
                } else if approx_eq!(f64, percent_along, 1.0) {
                    CurveLoc::F
                } else {
                    CurveLoc::M(percent_along)
                }
            };

            IntersectionResult::One(PtLoc(segment_loc, curve_loc))
        }
        Ordering::Less => {
            // possibly one intersection, if the curve crosses the line.
            // or, possibly two intersections.

            // does the curve (ci->cf) cross the line (si->sf)?
            // if so, then si->sf x si->ci will have a different sign from si->sf x si->cf.
            if segment.cross_z(&Segment(segment.i, curve_arc.pt_i()))
                * segment.cross_z(&Segment(segment.i, curve_arc.pt_f()))
                < 0.0
            {
                let r = curve_arc.radius.0;
                let x_0 = curve_arc.ctr.x.0;
                let x_1 = segment.i.x.0;
                let x_2 = segment.f.x.0;
                let y_0 = curve_arc.ctr.y.0;
                let y_1 = segment.i.y.0;
                let y_2 = segment.f.y.0;

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
                let isxn_1 = {
                    let x = (-1.0 * c_b + (c_d).sqrt()) / (2.0 * c_a);
                    let y = -1.0 * (l_a * x + l_c) / (l_b);
                    Pt(x, y)
                };

                let isxn_2 = {
                    let x = (-1.0 * c_b - (c_d).sqrt()) / (2.0 * c_a);
                    let y = -1.0 * (l_a * x + l_c) / (l_b);
                    Pt(x, y)
                };

                // here pac == percent along curve, and pas == percent along segment.

                let pac1 = abp(&curve_arc.ctr, &isxn_1, &curve_arc.pt_i())
                    / abp(&curve_arc.ctr, &curve_arc.pt_f(), &curve_arc.pt_i());
                let pac2 = abp(&curve_arc.ctr, &isxn_2, &curve_arc.pt_i())
                    / abp(&curve_arc.ctr, &curve_arc.pt_f(), &curve_arc.pt_i());

                // both good options! But only one will have an segment and
                // curve interpolation value of between 0 and 1.

                let (isxn, pas, pac) = match (
                    interpolate_2d_checked(segment.i, segment.f, isxn_1),
                    interpolate_2d_checked(segment.i, segment.f, isxn_2),
                ) {
                    (Ok(pas1), Ok(pas2)) => {
                        match (
                            FloatOrd(pac1).cmp(&FloatOrd(0.0)),
                            FloatOrd(pac2).cmp(&FloatOrd(0.0)),
                        ) {
                            (Ordering::Less, Ordering::Equal | Ordering::Greater) => {
                                (isxn_2, pas2, pac2)
                            }
                            (Ordering::Equal | Ordering::Greater, Ordering::Less) => {
                                (isxn_1, pas1, pac2)
                            }
                            _ => panic!("cannot have two valid intersection values"),
                        }
                    }
                    (Ok(pas1), Err(_)) => (isxn_1, pas1, pac1),
                    (Err(_), Ok(pas2)) => (isxn_2, pas2, pac2),
                    (Err(_), Err(_)) => panic!("cannot have zero valid intersection values."),
                };

                let segment_loc = {
                    if approx_eq!(f64, pas, 0.0) {
                        SegmentLoc::I
                    } else if approx_eq!(f64, pas, 1.0) {
                        SegmentLoc::F
                    } else {
                        SegmentLoc::M(pas)
                    }
                };

                let curve_loc = {
                    let pac = abp(&curve_arc.ctr, &isxn, &curve_arc.pt_i())
                        / abp(&curve_arc.ctr, &curve_arc.pt_f(), &curve_arc.pt_i());

                    if approx_eq!(f64, pac, 0.0) {
                        CurveLoc::I
                    } else if approx_eq!(f64, pac, 1.0) {
                        CurveLoc::F
                    } else {
                        CurveLoc::M(pac)
                    }
                };

                return IntersectionResult::One(PtLoc(segment_loc, curve_loc));
            } else {
                IntersectionResult::None
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
        Ok(vec![])
    }
}

#[cfg(test)]
mod test {
    use {super::*, crate::segment::Segment, assert_matches::assert_matches, std::f64::consts::PI};

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
                IntersectionResult::One(PtLoc(sl, cl)) => (sl, cl)
            );
            assert_eq!(sl, expected_segment_loc);
            assert_eq!(cl, expected_curve_loc);
        }
    }

    #[test]
    fn test_curve_one_intersection_crossing() {
        let segment = Segment(Pt(0.0, 0.0), Pt(2.0, 0.0));

        let curve_arc = CurveArc(Pt(1.0, 0.0), FRAC_PI_2..3.0 * FRAC_PI_2, 0.5);

        let (sl, cl) = assert_matches!(
            intersections_of_line_and_curvearc(&segment, &curve_arc),
            IntersectionResult::One(PtLoc(sl, cl)) => (sl, cl)
        );
        assert_eq!(sl, SegmentLoc::M(0.25));
        assert_eq!(cl, CurveLoc::M(0.5));
    }

    #[test]
    fn test_curve_two_intersections() {
        let segment = Segment(Pt(0.0, 0.0), Pt(2.0, 0.0));
        for (
            curve_arc,
            (expected_segment_loc_1, expected_curve_loc_1),
            (expected_segment_loc_2, expected_curve_loc_2),
        ) in [
            // foo

        ] {
            let ((sl1, cl1), (sl2, cl2)) = assert_matches!(
                intersections_of_line_and_curvearc(&segment, &curve_arc),
                IntersectionResult::Two(PtLoc(sl1, cl1), PtLoc(sl2, cl2)) => ((sl1, cl1), (sl2, cl2))

            );
            assert_eq!(sl1, expected_segment_loc_1);
            assert_eq!(cl1, expected_curve_loc_1);
            assert_eq!(sl2, expected_segment_loc_2);
            assert_eq!(cl2, expected_curve_loc_2);
        }
    }
}
