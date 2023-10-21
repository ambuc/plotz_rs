//! A curve.

use crate::{
    bounded::{Bounded, Bounds},
    crop::{CropType, Croppable, PointLoc},
    interpolate::interpolate_2d_checked,
    shapes::{
        pg::{abp, Pg},
        pt::{PolarPt, Pt},
        sg::Sg,
    },
    *,
};
use anyhow::{anyhow, Result};
use float_cmp::approx_eq;
use float_ord::FloatOrd;
use std::{
    cmp::{max, min, Ordering},
    f64::consts::*,
    ops::*,
};

#[derive(Clone, Copy, Debug, PartialEq)]
/// A single curvearc, i.e. some section of a circle.
pub struct CurveArc {
    /// The center of the circle.
    pub ctr: Pt,
    /// The initial angle of the circle. 0 <= a <= TAU, angle_i <= angle_f.
    pub angle_i: f64,
    /// The final angle of the circle. 0 <= a <= TAU, angle_i <= angle_f.
    pub angle_f: f64,
    /// The radius of the circle.
    pub radius: f64,
}

impl CurveArc {
    fn pt_i(&self) -> Pt {
        self.ctr + PolarPt(self.radius, self.angle_i)
    }
    fn pt_f(&self) -> Pt {
        self.ctr + PolarPt(self.radius, self.angle_f)
    }
    fn angle_range(&self) -> RangeInclusive<f64> {
        self.angle_i..=self.angle_f
    }
    /// Iterator.
    pub fn iter(&self) -> impl Iterator<Item = &Pt> {
        std::iter::once(&self.ctr)
    }
    /// Mutable iterator.
    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut Pt> {
        std::iter::once(&mut self.ctr)
    }
}

impl Bounded for CurveArc {
    fn bounds(&self) -> crate::bounded::Bounds {
        Bounds {
            top_bound: self.ctr.y
                + self.radius
                    * if self.angle_range().contains(&FRAC_PI_2) {
                        1.0
                    } else {
                        max(FloatOrd(self.angle_i.sin()), FloatOrd(self.angle_f.sin())).0
                    },
            bottom_bound: self.ctr.y
                + self.radius
                    * if self.angle_range().contains(&(3.0 * FRAC_PI_2)) {
                        -1.0
                    } else {
                        min(FloatOrd(self.angle_i.sin()), FloatOrd(self.angle_f.sin())).0
                    },
            left_bound: self.ctr.x
                + self.radius
                    * if self.angle_range().contains(&PI) {
                        -1.0
                    } else {
                        min(FloatOrd(self.angle_i.cos()), FloatOrd(self.angle_f.cos())).0
                    },
            right_bound: self.ctr.x
                + self.radius
                    * if self.angle_range().contains(&TAU) {
                        1.0
                    } else {
                        max(FloatOrd(self.angle_i.cos()), FloatOrd(self.angle_f.cos())).0
                    },
        }
    }
}

#[allow(non_snake_case)]
/// A single curvearc, i.e. some section of a circle.
pub fn CurveArc(ctr: Pt, sweep: RangeInclusive<f64>, radius: f64) -> CurveArc {
    assert!(sweep.start() <= sweep.end(), "sweep: {:?}", sweep);
    assert!(
        (-1.0 * TAU..=TAU).contains(sweep.start()),
        "sweep: {:?}",
        sweep,
    );
    assert!(
        (-1.0 * TAU..=TAU).contains(sweep.end()),
        "sweep: {:?}",
        sweep,
    );
    CurveArc {
        ctr,
        angle_i: *sweep.start(),
        angle_f: *sweep.end(),
        radius,
    }
}

fn split_range(
    input: RangeInclusive<f64>,
    basis: RangeInclusive<f64>,
) -> Box<dyn Iterator<Item = RangeInclusive<f64>>> {
    let basis_width: f64 = basis.end() - basis.start();
    match (basis.contains(input.start()), basis.contains(input.end())) {
        (true, true) => Box::new(std::iter::once(input.clone())),
        (true, false) => {
            // normal case
            let new_input_start = *basis.start();
            let new_input_end = *input.end() - basis_width;
            let new_input = new_input_start..=new_input_end;

            Box::new(
                std::iter::once(*input.start()..=*basis.end()).chain(split_range(new_input, basis)),
            )
        }
        (false, _) => {
            let mut new_input = input;
            while new_input.start() < basis.start() {
                new_input = (new_input.start() + basis_width)..=(new_input.end() + basis_width);
            }
            while new_input.start() > basis.end() {
                new_input = (new_input.start() - basis_width)..=(new_input.end() - basis_width);
            }

            split_range(new_input, basis)
        }
    }
}

#[allow(non_snake_case)]
/// Creates a new vector of CurveArcs. Why use this over |CurveArc(..)|? Simple:
/// If you ask for a CurveArc which circles more than once, it complains.
/// |CurveArcs(..)| will happily do the arithmetic and return a set of
/// |CurveArc| objects which circle the center the correct fractional number of
/// times. This is nice on a pen plotter.
pub fn CurveArcs(ctr: Pt, sweep: RangeInclusive<f64>, radius: f64) -> Vec<CurveArc> {
    split_range(sweep, 0.0..=TAU)
        .map(|r| CurveArc(ctr, r, radius))
        .collect::<Vec<_>>()
}

impl Add<Pt> for CurveArc {
    type Output = Self;
    fn add(self, rhs: Pt) -> Self::Output {
        CurveArc {
            ctr: self.ctr + rhs,
            ..self
        }
    }
}
impl AddAssign<Pt> for CurveArc {
    fn add_assign(&mut self, rhs: Pt) {
        self.ctr += rhs;
    }
}
impl Div<f64> for CurveArc {
    type Output = CurveArc;
    fn div(self, rhs: f64) -> Self::Output {
        CurveArc(self.ctr / rhs, self.angle_range(), self.radius / rhs)
    }
}
impl DivAssign<f64> for CurveArc {
    fn div_assign(&mut self, rhs: f64) {
        self.ctr /= rhs;
        self.radius /= rhs;
    }
}
impl Mul<f64> for CurveArc {
    type Output = CurveArc;
    fn mul(self, rhs: f64) -> Self::Output {
        CurveArc(self.ctr * rhs, self.angle_range(), self.radius * rhs)
    }
}
impl MulAssign<f64> for CurveArc {
    fn mul_assign(&mut self, rhs: f64) {
        self.ctr *= rhs;
        self.radius *= rhs;
    }
}
impl Sub<Pt> for CurveArc {
    type Output = Self;
    fn sub(self, rhs: Pt) -> Self::Output {
        CurveArc {
            ctr: self.ctr - rhs,
            ..self
        }
    }
}
impl SubAssign<Pt> for CurveArc {
    fn sub_assign(&mut self, rhs: Pt) {
        self.ctr -= rhs;
    }
}
impl RemAssign<Pt> for CurveArc {
    fn rem_assign(&mut self, rhs: Pt) {
        self.ctr %= rhs;
    }
}

#[derive(Debug, PartialEq, Copy, Clone)]
enum SegmentLoc {
    I,
    M(f64), // percentage of the way along
    F,
}
impl SegmentLoc {
    #[cfg(test)]
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
    segment: &Sg,
    curve_arc: &CurveArc,
) -> Result<IntersectionResult> {
    let (x_0, y_0) = (curve_arc.ctr.x, curve_arc.ctr.y);
    let (x_1, y_1) = (segment.i.x, segment.i.y);
    let (x_2, y_2) = (segment.f.x, segment.f.y);
    let r = curve_arc.radius;

    // d is distance to line. (see (14) in
    // https://mathworld.wolfram.com/Point-LineDistance2-Dimensional.html)
    let d: f64 = ((x_2 - x_1) * (y_1 - y_0) - (x_1 - x_0) * (y_2 - y_1)).abs()
        / ((x_2 - x_1).powi(2) + (y_2 - y_1).powi(2)).sqrt();

    match FloatOrd(d).cmp(&FloatOrd(curve_arc.radius)) {
        Ordering::Greater => Ok(IntersectionResult::None),
        Ordering::Equal => {
            let isxn =
                curve_arc.ctr + PolarPt(curve_arc.radius, segment.slope().atan() + FRAC_PI_2);

            if let Ok(f) = interpolate_2d_checked(segment.i, segment.f, isxn) {
                if let Some(segment_loc) = SegmentLoc::from_f64(f) {
                    if let Some(curve_loc) = CurveLoc::from_f64(
                        abp(&curve_arc.ctr, &isxn, &curve_arc.pt_i())
                            / abp(&curve_arc.ctr, &curve_arc.pt_f(), &curve_arc.pt_i()),
                    ) {
                        return Ok(IntersectionResult::One(PtLoc(isxn, segment_loc, curve_loc)));
                    }
                }
            }
            Ok(IntersectionResult::None)
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
                // . . L_C = x_2 * y_1 - x_1 * y_2
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
                        let one = if is_neg { -1.0 } else { 1.0 };
                        let x = (-1.0 * c_b + one * (c_d.abs()).sqrt()) / (2.0 * c_a);

                        let y = if approx_eq!(f64, l_b, 0.0) {
                            // if l_b == 0.0, then the line is vertical -- and
                            // we have to get the value of |y| from the circle,
                            // not from the line (since the equation for a line
                            // is just x=c. y is not involved, it's not a
                            // function)

                            // (x - x_0)^2 + (y - y_0)^2 = r^2 ==>
                            // y = +/- sqrt(r^2 - (x-x_0)^2) + y_0
                            one * ((curve_arc.radius).powi(2) - (x - curve_arc.ctr.x).powi(2))
                                .abs()
                                .sqrt()
                                + curve_arc.ctr.y
                        } else {
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
                    _ => return Err(anyhow!("not sure what's going on here.")),
                }
            };

            // both good options! But only one will have an segment and
            // curve interpolation value of between 0 and 1.

            // here pac == percent along curve, and pas == percent along segment.
            let full_curve_angle = curve_arc.angle_f - curve_arc.angle_i;

            let sl1 = interpolate_2d_checked(segment.i, segment.f, isxn_1)
                .ok()
                .and_then(SegmentLoc::from_f64);
            let cl1 = CurveLoc::from_f64(
                ((abp(&curve_arc.ctr, &curve_arc.pt_i(), &isxn_1) + TAU) % TAU) / full_curve_angle,
            );
            let sl2 = interpolate_2d_checked(segment.i, segment.f, isxn_2)
                .ok()
                .and_then(SegmentLoc::from_f64);
            let cl2 = CurveLoc::from_f64(
                ((abp(&curve_arc.ctr, &curve_arc.pt_i(), &isxn_2) + TAU) % TAU) / full_curve_angle,
            );

            match (sl1, cl1, sl2, cl2) {
                (Some(sl1), Some(cl1), Some(sl2), Some(cl2)) if sl1 == sl2 && cl1 == cl2 => {
                    Ok(IntersectionResult::One(PtLoc(isxn_1, sl1, cl1)))
                }
                (Some(sl1), Some(cl1), Some(sl2), Some(cl2)) => Ok(IntersectionResult::Two(
                    PtLoc(isxn_1, sl1, cl1),
                    PtLoc(isxn_2, sl2, cl2),
                )),
                (Some(sl1), Some(cl1), _, _) => {
                    Ok(IntersectionResult::One(PtLoc(isxn_1, sl1, cl1)))
                }
                (_, _, Some(sl2), Some(cl2)) => {
                    Ok(IntersectionResult::One(PtLoc(isxn_2, sl2, cl2)))
                }
                _ => Err(anyhow!("is this right?")),
            }
        }
    }
}

impl Croppable for CurveArc {
    type Output = CurveArc;
    fn crop(&self, frame: &Pg, crop_type: CropType) -> Result<Vec<Self::Output>>
    where
        Self: Sized,
    {
        assert_eq!(crop_type, CropType::Inclusive);

        let mut isxns: Vec<PtLoc> = vec![];
        for frame_segment in frame.to_segments() {
            let discovered = match intersections_of_line_and_curvearc(&frame_segment, self)? {
                IntersectionResult::None => vec![],
                IntersectionResult::One(pl) => vec![pl],
                IntersectionResult::Two(pl1, pl2) => vec![pl1, pl2],
            };
            isxns.extend(discovered);
        }

        // either the curve is totally within and has no overlaps, or is totally
        // without and has no overlaps.
        if isxns.is_empty() {
            //
            let contains_i = frame.contains_pt(&self.pt_i());
            let contains_f = frame.contains_pt(&self.pt_f());
            if let (
                Ok(PointLoc::Inside | PointLoc::OnSegment(_) | PointLoc::OnPoint(_)),
                Ok(PointLoc::Inside | PointLoc::OnSegment(_) | PointLoc::OnPoint(_)),
            ) = (contains_i, contains_f)
            {
                return Ok(vec![*self]);
            }
        }

        let mut isxns_angles: Vec<FloatOrd<f64>> = isxns
            .into_iter()
            .map(|PtLoc(_, _, cl)| {
                FloatOrd(self.angle_i + (self.angle_f - self.angle_i) * cl.as_f64())
            })
            .collect::<Vec<_>>();
        if !matches!(frame.contains_pt(&self.pt_i()), Ok(PointLoc::Outside)) {
            isxns_angles.insert(0, FloatOrd(self.angle_i));
        }
        if !matches!(frame.contains_pt(&self.pt_f()), Ok(PointLoc::Outside)) {
            isxns_angles.insert(0, FloatOrd(self.angle_f));
        }
        isxns_angles.sort();

        let mut r = vec![];

        for (a1, a2) in isxns_angles
            .iter()
            .zip(isxns_angles.iter().skip(1))
            .map(|(a1, a2)| {
                if a1 > a2 {
                    (a1.0, a2.0 + TAU)
                } else {
                    (a1.0, a2.0)
                }
            })
        {
            let mdpt = self.ctr + PolarPt(self.radius, (a1 + a2) / 2.0);
            if !matches!(frame.contains_pt(&mdpt), Ok(PointLoc::Outside)) {
                r.push(CurveArc(self.ctr, a1..=a2, self.radius));
            }
        }

        Ok(r)
    }
    fn crop_excluding(&self, _other: &Pg) -> Result<Vec<Self::Output>>
    where
        Self: Sized,
    {
        unimplemented!("TODO");
    }
}

impl Translatable for CurveArc {}
impl Scalable<f64> for CurveArc {}

impl Nullable for CurveArc {
    fn is_empty(&self) -> bool {
        false
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::shapes::{
        pg::{Pg, Rect},
        sg::Sg,
    };
    use assert_matches::assert_matches;
    use float_cmp::assert_approx_eq;
    use test_case::test_case;

    #[test]
    fn test_curve_zero_intersections() -> Result<()> {
        assert_matches!(
            intersections_of_line_and_curvearc(
                &Sg((0, 0), (3, 0)),
                &CurveArc(Pt(1, 1), 0.0..=PI, 0.5)
            )?,
            IntersectionResult::None
        );
        Ok(())
    }

    #[test_case(
        CurveArc(Pt(1, 1), 0.0..=PI, 1.0), SegmentLoc::M(0.5),
        CurveLoc::M(0.5); "segment m, curve m"
    )]
    #[test_case(
        CurveArc(Pt(1, 1), -1.0 * FRAC_PI_2..=FRAC_PI_2, 1.0), SegmentLoc::M(0.5),
        CurveLoc::F; "segment m, curve f"
    )]
    #[test_case(
        CurveArc(Pt(1, 1), FRAC_PI_2..=3.0 * FRAC_PI_2, 1.0), SegmentLoc::M(0.5),
        CurveLoc::I; "segment m, curve i"
    )]
    #[test_case(
        CurveArc(Pt(0, 1), -1.0 * FRAC_PI_2..=FRAC_PI_2, 1.0), SegmentLoc::I,
        CurveLoc::F; "segment i, curve f"
    )]
    #[test_case(
        CurveArc(Pt(0, 1), FRAC_PI_2..=3.0 * FRAC_PI_2, 1.0), SegmentLoc::I,
        CurveLoc::I; "segment i, curve i"
    )]
    #[test_case(
        CurveArc(Pt(0, 1), 0.0..=PI, 1.0), SegmentLoc::I,
        CurveLoc::M(0.5); "segment i, curve m"
    )]
    #[test_case(
        CurveArc(Pt(2, 1), 0.0..=PI, 1.0), SegmentLoc::F,
        CurveLoc::M(0.5); "segment f, curve m"
    )]
    #[test_case(
        CurveArc(Pt(2, 1), FRAC_PI_2..=3.0 * FRAC_PI_2, 1.0), SegmentLoc::F,
        CurveLoc::I; "segment f, curve i"
    )]
    #[test_case(
        CurveArc(Pt(2, 1), -1.0 * FRAC_PI_2..=FRAC_PI_2, 1.0), SegmentLoc::F,
        CurveLoc::F; "segment f, curve f"
    )]
    fn test_curve_one_intersection_tangent(
        curve_arc: CurveArc,
        expected_segment_loc: SegmentLoc,
        expected_curve_loc: CurveLoc,
    ) -> Result<()> {
        let segment = Sg((0, 0), (2, 0));

        let (sl, cl) = assert_matches!(
            intersections_of_line_and_curvearc(&segment, &curve_arc)?,
            IntersectionResult::One(PtLoc(_, sl, cl)) => (sl, cl)
        );
        assert_eq!(sl, expected_segment_loc);
        assert_eq!(cl, expected_curve_loc);
        Ok(())
    }

    #[test_case(
        Sg((0, 0), (2, 0)),
        CurveArc(Pt(1, 0), FRAC_PI_2..=3.0 * FRAC_PI_2, 0.5),
        (Pt(0.50, 0), SegmentLoc::M(0.25), CurveLoc::M(0.5));
        "intersection 1"
    )]
    #[test_case(
        Sg((2, 0), (2, 2)),
        CurveArc(Pt(2, 0), 0.0..=3.0 * FRAC_PI_2, 1.0),
        (Pt(2, 1), SegmentLoc::M(0.5), CurveLoc::M(1.0 / 3.0));
        "intersection 2"
    )]
    fn test_curve_one_intersection_crossing(
        segment: Sg,
        curve_arc: CurveArc,
        (expected_point_loc, expected_segment_loc, expected_curve_loc): (Pt, SegmentLoc, CurveLoc),
    ) -> Result<()> {
        let (pl, sl, cl) = assert_matches!(
            intersections_of_line_and_curvearc(&segment, &curve_arc)?,
            IntersectionResult::One(PtLoc(pl, sl, cl)) => (pl, sl, cl)
        );
        assert_eq!(pl, expected_point_loc);
        assert_eq!(sl, expected_segment_loc);
        assert_eq!(cl, expected_curve_loc);
        Ok(())
    }

    #[test_case(
        Sg((0., 0.), (3., 0.)),
        CurveArc(Pt(1.5, 0), 0.0..=PI, 0.5),
        PtLoc(Pt(1, 0), SegmentLoc::M(1.0 / 3.0), CurveLoc::F),
        PtLoc(Pt(2, 0), SegmentLoc::M(2.0 / 3.0), CurveLoc::I);
        "segment m curve i, segment m curve f"
    )]
    #[test_case(
        Sg((0, 2), (0, 0.18)),
        CurveArc(Pt(1, 1), 0.0..=TAU, 1.1),
        PtLoc(Pt(0, 0.5417424305044158), SegmentLoc::M(0.8012404227997715), CurveLoc::M(0.5683888259129364)),
        PtLoc(Pt(0, 1.4582575694955842), SegmentLoc::M(0.29766067610132735), CurveLoc::M(0.4316111740870635));
        "vertical")
    ]
    fn test_curve_two_intersections(
        segment: Sg,
        curve_arc: CurveArc,
        e_pl1: PtLoc,
        e_pl2: PtLoc,
    ) -> Result<()> {
        let (pl1, pl2) = assert_matches!(
            intersections_of_line_and_curvearc(&segment, &curve_arc)?,
            IntersectionResult::Two(pl1, pl2) => (pl1, pl2)
        );

        let PtLoc(pt1, sl1, cl1) = pl1;
        let PtLoc(pt2, sl2, cl2) = pl2;
        let PtLoc(e_pt1, e_sl1, e_cl1) = e_pl1;
        let PtLoc(e_pt2, e_sl2, e_cl2) = e_pl2;

        assert_approx_eq!(f64, pt1.x, e_pt1.x);
        assert_approx_eq!(f64, pt1.y, e_pt1.y);
        assert_approx_eq!(f64, sl1.as_f64(), e_sl1.as_f64());
        assert_approx_eq!(f64, cl1.as_f64(), e_cl1.as_f64());

        assert_approx_eq!(f64, pt2.x, e_pt2.x);
        assert_approx_eq!(f64, pt2.y, e_pt2.y);
        assert_approx_eq!(f64, sl2.as_f64(), e_sl2.as_f64());
        assert_approx_eq!(f64, cl2.as_f64(), e_cl2.as_f64());
        Ok(())
    }

    #[test_case(
        Rect((0, 0), (2, 2)).unwrap(),
        CurveArc(Pt(2, 0), 0.0..=3.0 * FRAC_PI_2, 1.0),
        vec![
            CurveArc(Pt(2, 0), FRAC_PI_2..=PI, 1.0)
        ];
        "two intersections, one resultant"
    )]
    #[test_case(
        Rect((0, 0), (2, 2)).unwrap(),
        CurveArc(Pt(1, 1), 0.0..=TAU, 0.5),
        vec![
            CurveArc(Pt(1.0, 1.0), 0.0..=TAU, 0.5)
        ];
        "no intersections"
    )]
    #[test_case(
        Rect((0, 0), (2, 2)).unwrap(),
        CurveArc(Pt(1, 1), 0.0..=TAU, 1.0),
        vec![
            CurveArc(Pt(1, 1), 0.0..=TAU, 1.0)
        ];
        "four intersections, all tangent"
    )]
    #[test_case(
        Rect((0, 0), (2, 2)).unwrap(),
        CurveArc(Pt(1, 1), 0.0..=TAU, 1.1),
        vec![
            CurveArc(Pt(1, 1), 0.4296996661514249..=1.141096660643471, 1.1),
            CurveArc(Pt(1, 1), 2.0004959929463215..=2.711892987438368, 1.1),
            CurveArc(Pt(1, 1), 3.5712923197412176..=4.282689314233265, 1.1),
            CurveArc(Pt(1, 1), 5.1420886465361150..=5.853485641028161, 1.1),
        ];
        "four intersections, all passthrough"
    )]
    fn test_curvearc_crop(rect: Pg, curvearc: CurveArc, expected_curvearcs: Vec<CurveArc>) {
        let actual_curvearcs = curvearc.crop_to(&rect).expect("crop");
        assert_eq!(actual_curvearcs.len(), expected_curvearcs.len());

        for (actual, expected) in actual_curvearcs.iter().zip(expected_curvearcs.iter()) {
            assert_eq!(actual.ctr, expected.ctr);
            assert_approx_eq!(f64, actual.angle_i, expected.angle_i);
            assert_approx_eq!(f64, actual.angle_f, expected.angle_f);
            assert_eq!(actual.radius, expected.radius);
        }
    }

    #[test_case(0.0..=1.0, 0.0..=1.0, vec![0.0..=1.0]; "no-op")]
    #[test_case(0.0..=0.9, 0.0..=1.0, vec![0.0..=0.9]; "input ends early")]
    #[test_case(0.1..=1.0, 0.0..=1.0, vec![0.1..=1.0]; "input starts late")]
    #[test_case(0.1..=0.9, 0.0..=1.0, vec![0.1..=0.9]; "input within basis")]
    #[test_case(0.0..=1.5, 0.0..=1.0, vec![0.0..=1.0, 0.0..=0.5]; "one point five width")]
    #[test_case(0.0..=2.0, 0.0..=1.0, vec![0.0..=1.0, 0.0..=1.0]; "double width")]
    #[test_case(0.0..=3.0, 0.0..=1.0, vec![0.0..=1.0, 0.0..=1.0, 0.0..=1.0]; "triple width")]
    #[test_case(0.1..=2.9, 0.0..=1.0, vec![0.1..=1.0, 0.0..=1.0, 0.0..=0.9]; "triple width, precropped")]
    fn test_split_range(
        input: RangeInclusive<f64>,
        basis: RangeInclusive<f64>,
        expected: Vec<RangeInclusive<f64>>,
    ) {
        let actual = split_range(input, basis);
        for (actual, expected) in actual.zip(expected.iter()) {
            assert_approx_eq!(f64, *actual.start(), *expected.start());
            assert_approx_eq!(f64, *actual.end(), *expected.end());
        }
    }
}
