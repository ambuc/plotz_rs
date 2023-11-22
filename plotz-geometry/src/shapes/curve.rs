//! A curve.
#![allow(missing_docs)]

use crate::{
    bounded::{Bounded, Bounds},
    crop::{CropType, Croppable, PointLocation},
    interpolate::interpolate_2d_checked,
    obj2::ObjType2d,
    shapes::{
        point::{Point, PolarPt},
        polygon::{abp, Polygon},
        segment::Segment,
    },
    utils::Percent,
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
    pub ctr: Point,
    /// The initial angle of the circle. 0 <= a <= TAU, angle_i <= angle_f.
    pub angle_i: f64,
    /// The final angle of the circle. 0 <= a <= TAU, angle_i <= angle_f.
    pub angle_f: f64,
    /// The radius of the circle.
    pub radius: f64,
}

impl CurveArc {
    fn pt_i(&self) -> Point {
        self.ctr + PolarPt(self.radius, self.angle_i)
    }
    fn pt_f(&self) -> Point {
        self.ctr + PolarPt(self.radius, self.angle_f)
    }
    fn angle_range(&self) -> RangeInclusive<f64> {
        self.angle_i..=self.angle_f
    }
}

impl Bounded for CurveArc {
    fn bounds(&self) -> Result<Bounds> {
        Ok(Bounds {
            y_max: self.ctr.y
                + self.radius
                    * if self.angle_range().contains(&FRAC_PI_2) {
                        1.0
                    } else {
                        max(FloatOrd(self.angle_i.sin()), FloatOrd(self.angle_f.sin())).0
                    },
            y_min: self.ctr.y
                + self.radius
                    * if self.angle_range().contains(&(3.0 * FRAC_PI_2)) {
                        -1.0
                    } else {
                        min(FloatOrd(self.angle_i.sin()), FloatOrd(self.angle_f.sin())).0
                    },
            x_min: self.ctr.x
                + self.radius
                    * if self.angle_range().contains(&PI) {
                        -1.0
                    } else {
                        min(FloatOrd(self.angle_i.cos()), FloatOrd(self.angle_f.cos())).0
                    },
            x_max: self.ctr.x
                + self.radius
                    * if self.angle_range().contains(&TAU) {
                        1.0
                    } else {
                        max(FloatOrd(self.angle_i.cos()), FloatOrd(self.angle_f.cos())).0
                    },
        })
    }
}

#[allow(non_snake_case)]
/// A single curvearc, i.e. some section of a circle.
pub fn CurveArc(ctr: Point, sweep: RangeInclusive<f64>, radius: f64) -> CurveArc {
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
pub fn CurveArcs(ctr: Point, sweep: RangeInclusive<f64>, radius: f64) -> Vec<CurveArc> {
    split_range(sweep, 0.0..=TAU)
        .map(|r| CurveArc(ctr, r, radius))
        .collect::<Vec<_>>()
}

impl Add<Point> for CurveArc {
    type Output = Self;
    fn add(self, rhs: Point) -> Self::Output {
        CurveArc {
            ctr: self.ctr + rhs,
            ..self
        }
    }
}
impl AddAssign<Point> for CurveArc {
    fn add_assign(&mut self, rhs: Point) {
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
impl Sub<Point> for CurveArc {
    type Output = Self;
    fn sub(self, rhs: Point) -> Self::Output {
        CurveArc {
            ctr: self.ctr - rhs,
            ..self
        }
    }
}
impl SubAssign<Point> for CurveArc {
    fn sub_assign(&mut self, rhs: Point) {
        self.ctr -= rhs;
    }
}
impl RemAssign<Point> for CurveArc {
    fn rem_assign(&mut self, rhs: Point) {
        self.ctr %= rhs;
    }
}

#[derive(Debug, PartialEq, Copy, Clone)]
struct PtLoc(
    Point,
    /*SegmentLoc*/ Percent,
    /*CurveLoc*/ Percent,
);

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

            if let Ok(segment_loc) = interpolate_2d_checked(segment.i, segment.f, isxn) {
                if let Ok(curve_loc) = Percent::new(
                    abp(&curve_arc.ctr, &isxn, &curve_arc.pt_i())
                        / abp(&curve_arc.ctr, &curve_arc.pt_f(), &curve_arc.pt_i()),
                ) {
                    return Ok(IntersectionResult::One(PtLoc(isxn, segment_loc, curve_loc)));
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

                        Point(x, y)
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

            let sl1 = interpolate_2d_checked(segment.i, segment.f, isxn_1);
            let cl1 = Percent::new(
                ((abp(&curve_arc.ctr, &curve_arc.pt_i(), &isxn_1) + TAU) % TAU) / full_curve_angle,
            );
            let sl2 = interpolate_2d_checked(segment.i, segment.f, isxn_2);
            let cl2 = Percent::new(
                ((abp(&curve_arc.ctr, &curve_arc.pt_i(), &isxn_2) + TAU) % TAU) / full_curve_angle,
            );

            match (sl1, cl1, sl2, cl2) {
                (Ok(sl1), Ok(cl1), Ok(sl2), Ok(cl2)) if sl1 == sl2 && cl1 == cl2 => {
                    Ok(IntersectionResult::One(PtLoc(isxn_1, sl1, cl1)))
                }
                (Ok(sl1), Ok(cl1), Ok(sl2), Ok(cl2)) => Ok(IntersectionResult::Two(
                    PtLoc(isxn_1, sl1, cl1),
                    PtLoc(isxn_2, sl2, cl2),
                )),
                (Ok(sl1), Ok(cl1), _, _) => Ok(IntersectionResult::One(PtLoc(isxn_1, sl1, cl1))),
                (_, _, Ok(sl2), Ok(cl2)) => Ok(IntersectionResult::One(PtLoc(isxn_2, sl2, cl2))),
                _ => Err(anyhow!("is this right?")),
            }
        }
    }
}

impl Croppable for CurveArc {
    type Output = CurveArc;
    fn crop(&self, frame: &Polygon, crop_type: CropType) -> Result<Vec<Self::Output>>
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
            let contains_i = frame.contains_pt_deprecated(&self.pt_i());
            let contains_f = frame.contains_pt_deprecated(&self.pt_f());
            if let (
                Ok(PointLocation::Inside | PointLocation::OnSegment(_) | PointLocation::OnPoint(_)),
                Ok(PointLocation::Inside | PointLocation::OnSegment(_) | PointLocation::OnPoint(_)),
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
        if !matches!(
            frame.contains_pt_deprecated(&self.pt_i()),
            Ok(PointLocation::Outside)
        ) {
            isxns_angles.insert(0, FloatOrd(self.angle_i));
        }
        if !matches!(
            frame.contains_pt_deprecated(&self.pt_f()),
            Ok(PointLocation::Outside)
        ) {
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
            if !matches!(
                frame.contains_pt_deprecated(&mdpt),
                Ok(PointLocation::Outside)
            ) {
                r.push(CurveArc(self.ctr, a1..=a2, self.radius));
            }
        }

        Ok(r)
    }
}

impl Object for CurveArc {
    fn objtype(&self) -> ObjType2d {
        ObjType2d::CurveArc2d
    }

    fn iter(&self) -> Box<dyn Iterator<Item = &Point> + '_> {
        // NB: this can't impl. bounded, it's not the whole picture.
        Box::new(std::iter::once(&self.ctr))
    }

    fn iter_mut(&mut self) -> Box<dyn Iterator<Item = &mut Point> + '_> {
        // NB: this can't impl. mutable transforms, it's not the whole picture.
        Box::new(std::iter::once(&mut self.ctr))
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::shapes::{
        polygon::{Polygon, Rect},
        segment::Segment,
    };
    use assert_matches::assert_matches;
    use float_cmp::assert_approx_eq;
    use test_case::test_case;

    #[test]
    fn test_curve_zero_intersections() -> Result<()> {
        assert_matches!(
            intersections_of_line_and_curvearc(
                &Segment((0, 0), (3, 0)),
                &CurveArc(Point(1, 1), 0.0..=PI, 0.5)
            )?,
            IntersectionResult::None
        );
        Ok(())
    }

    #[test_case(
        CurveArc(Point(1, 1), 0.0..=PI, 1.0), Percent::Val(0.5),
        Percent::Val(0.5); "segment m, curve m"
    )]
    #[test_case(
        CurveArc(Point(1, 1), -1.0 * FRAC_PI_2..=FRAC_PI_2, 1.0), Percent::Val(0.5),
        Percent::One; "segment m, curve f"
    )]
    #[test_case(
        CurveArc(Point(1, 1), FRAC_PI_2..=3.0 * FRAC_PI_2, 1.0), Percent::Val(0.5),
        Percent::Zero; "segment m, curve i"
    )]
    #[test_case(
        CurveArc(Point(0, 1), -1.0 * FRAC_PI_2..=FRAC_PI_2, 1.0), Percent::Zero,
        Percent::One; "segment i, curve f"
    )]
    #[test_case(
        CurveArc(Point(0, 1), FRAC_PI_2..=3.0 * FRAC_PI_2, 1.0), Percent::Zero,
        Percent::Zero; "segment i, curve i"
    )]
    #[test_case(
        CurveArc(Point(0, 1), 0.0..=PI, 1.0), Percent::Zero,
        Percent::Val(0.5); "segment i, curve m"
    )]
    #[test_case(
        CurveArc(Point(2, 1), 0.0..=PI, 1.0), Percent::One,
        Percent::Val(0.5); "segment f, curve m"
    )]
    #[test_case(
        CurveArc(Point(2, 1), FRAC_PI_2..=3.0 * FRAC_PI_2, 1.0), Percent::One,
        Percent::Zero; "segment f, curve i"
    )]
    #[test_case(
        CurveArc(Point(2, 1), -1.0 * FRAC_PI_2..=FRAC_PI_2, 1.0), Percent::One,
        Percent::One; "segment f, curve f"
    )]
    fn test_curve_one_intersection_tangent(
        curve_arc: CurveArc,
        expected_segment_loc: Percent,
        expected_curve_loc: Percent,
    ) -> Result<()> {
        let segment = Segment((0, 0), (2, 0));

        let (sl, cl) = assert_matches!(
            intersections_of_line_and_curvearc(&segment, &curve_arc)?,
            IntersectionResult::One(PtLoc(_, sl, cl)) => (sl, cl)
        );
        assert_eq!(sl, expected_segment_loc);
        assert_eq!(cl, expected_curve_loc);
        Ok(())
    }

    #[test_case(
        Segment((0, 0), (2, 0)),
        CurveArc(Point(1, 0), FRAC_PI_2..=3.0 * FRAC_PI_2, 0.5),
        (Point(0.50, 0), Percent::Val(0.25), Percent::Val(0.5));
        "intersection 1"
    )]
    #[test_case(
        Segment((2, 0), (2, 2)),
        CurveArc(Point(2, 0), 0.0..=3.0 * FRAC_PI_2, 1.0),
        (Point(2, 1), Percent::Val(0.5), Percent::Val(1.0 / 3.0));
        "intersection 2"
    )]
    fn test_curve_one_intersection_crossing(
        segment: Segment,
        curve_arc: CurveArc,
        (expected_point_loc, expected_segment_loc, expected_curve_loc): (Point, Percent, Percent),
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
        Segment((0., 0.), (3., 0.)),
        CurveArc(Point(1.5, 0), 0.0..=PI, 0.5),
        PtLoc(Point(1, 0), Percent::Val(1.0 / 3.0), Percent::One),
        PtLoc(Point(2, 0), Percent::Val(2.0 / 3.0), Percent::Zero);
        "segment m curve i, segment m curve f"
    )]
    #[test_case(
        Segment((0, 2), (0, 0.18)),
        CurveArc(Point(1, 1), 0.0..=TAU, 1.1),
        PtLoc(Point(0, 0.5417424305044158), Percent::Val(0.8012404227997715), Percent::Val(0.5683888259129364)),
        PtLoc(Point(0, 1.4582575694955842), Percent::Val(0.29766067610132735), Percent::Val(0.4316111740870635));
        "vertical")
    ]
    fn test_curve_two_intersections(
        segment: Segment,
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
        CurveArc(Point(2, 0), 0.0..=3.0 * FRAC_PI_2, 1.0),
        vec![
            CurveArc(Point(2, 0), FRAC_PI_2..=PI, 1.0)
        ];
        "two intersections, one resultant"
    )]
    #[test_case(
        Rect((0, 0), (2, 2)).unwrap(),
        CurveArc(Point(1, 1), 0.0..=TAU, 0.5),
        vec![
            CurveArc(Point(1.0, 1.0), 0.0..=TAU, 0.5)
        ];
        "no intersections"
    )]
    #[test_case(
        Rect((0, 0), (2, 2)).unwrap(),
        CurveArc(Point(1, 1), 0.0..=TAU, 1.0),
        vec![
            CurveArc(Point(1, 1), 0.0..=TAU, 1.0)
        ];
        "four intersections, all tangent"
    )]
    #[test_case(
        Rect((0, 0), (2, 2)).unwrap(),
        CurveArc(Point(1, 1), 0.0..=TAU, 1.1),
        vec![
            CurveArc(Point(1, 1), 0.4296996661514249..=1.141096660643471, 1.1),
            CurveArc(Point(1, 1), 2.0004959929463215..=2.711892987438368, 1.1),
            CurveArc(Point(1, 1), 3.5712923197412176..=4.282689314233265, 1.1),
            CurveArc(Point(1, 1), 5.1420886465361150..=5.853485641028161, 1.1),
        ];
        "four intersections, all passthrough"
    )]
    fn test_curvearc_crop(
        rect: Polygon,
        curvearc: CurveArc,
        expected_curvearcs: Vec<CurveArc>,
    ) -> Result<()> {
        let actual_curvearcs = curvearc.crop_to(&rect)?;
        assert_eq!(actual_curvearcs.len(), expected_curvearcs.len());

        for (actual, expected) in actual_curvearcs.iter().zip(expected_curvearcs.iter()) {
            assert_eq!(actual.ctr, expected.ctr);
            assert_approx_eq!(f64, actual.angle_i, expected.angle_i);
            assert_approx_eq!(f64, actual.angle_f, expected.angle_f);
            assert_eq!(actual.radius, expected.radius);
        }
        Ok(())
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
