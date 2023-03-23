#![allow(unused)]
#![allow(missing_docs)]

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

#[derive(Debug, PartialEq, Eq)]
enum SegmentLoc {
    I,
    M(FloatOrd<f64>), // percentage of the way along
    F,
}

#[derive(Debug, PartialEq, Eq)]
enum CurveLoc {
    I,
    M(FloatOrd<f64>), // percentage of the way along
    F,
}

#[derive(Debug, PartialEq, Eq)]
struct PtLoc(SegmentLoc, CurveLoc);

#[derive(Debug, PartialEq, Eq)]
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
                let percent_along = interpolate::interpolate_2d_checked(segment.i, segment.f, isxn)
                    .expect("interpolation failed");

                if approx_eq!(f64, percent_along, 0.0) {
                    SegmentLoc::I
                } else if approx_eq!(f64, percent_along, 1.0) {
                    SegmentLoc::F
                } else {
                    SegmentLoc::M(FloatOrd(percent_along))
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
                    CurveLoc::M(FloatOrd(percent_along))
                }
            };

            IntersectionResult::One(PtLoc(segment_loc, curve_loc))
        }
        Ordering::Less => {
            // possibly two intersections.
            
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
    fn test_curve_one_intersection() {
        let segment = Segment(Pt(0.0, 0.0), Pt(2.0, 0.0));

        // segment m, curve m
        {
            let (sl, cl) = assert_matches!(
                intersections_of_line_and_curvearc(
                    &segment,
                    &CurveArc(Pt(1.0, 1.0), 0.0..PI, 1.0)
                ),
                IntersectionResult::One(PtLoc(sl, cl)) => (sl, cl)
            );
            assert_eq!(sl, SegmentLoc::M(FloatOrd(0.5)));
            assert_eq!(cl, CurveLoc::M(FloatOrd(0.5)));
        }
        // segment m, curve f
        {
            let sl = assert_matches!(
                intersections_of_line_and_curvearc(
                    &segment,
                    &CurveArc(Pt(1.0, 1.0), -1.0*FRAC_PI_2..FRAC_PI_2, 1.0)
                ),
                IntersectionResult::One(PtLoc(sl, CurveLoc::F)) => sl
            );
            assert_eq!(sl, SegmentLoc::M(FloatOrd(0.5)));
        }
        // segment m, curve i
        {
            let sl = assert_matches!(
                intersections_of_line_and_curvearc(
                    &segment,
                    &CurveArc(Pt(1.0, 1.0), FRAC_PI_2..3.0*FRAC_PI_2, 1.0)
                ),
                IntersectionResult::One(PtLoc(sl, CurveLoc::I)) => sl
            );
            assert_eq!(sl, SegmentLoc::M(FloatOrd(0.5)));
        }
        // segment i, curve f
        {
            assert_matches!(
                intersections_of_line_and_curvearc(
                    &segment,
                    &CurveArc(Pt(0.0, 1.0), -1.0 * FRAC_PI_2..FRAC_PI_2, 1.0)
                ),
                IntersectionResult::One(PtLoc(SegmentLoc::I, CurveLoc::F))
            );
        }
        // segment i, curve i
        {
            assert_matches!(
                intersections_of_line_and_curvearc(
                    &segment,
                    &CurveArc(Pt(0.0, 1.0), FRAC_PI_2..3.0 * FRAC_PI_2, 1.0)
                ),
                IntersectionResult::One(PtLoc(SegmentLoc::I, CurveLoc::I))
            );
        }
    }

    #[test]
    fn test_curve_two_intersections() {
        let segment = Segment(Pt(0.0, 0.0), Pt(2.0, 0.0));

        //
    }
}
