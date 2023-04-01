use {
    crate::{
        bounded::Bounded,
        crop::{CropToPolygonError, Croppable},
        draw_obj_inner::DrawObjInner,
        point::Pt,
        polygon::Polygon,
        traits::*,
    },
    float_ord::FloatOrd,
    std::ops::*,
};

#[derive(Debug, PartialEq, Clone)]
pub struct Group(Vec<DrawObjInner>);

impl Group {
    pub fn new(dois: impl IntoIterator<Item = DrawObjInner>) -> Group {
        Group(dois.into_iter().collect::<Vec<_>>())
    }

    pub fn iter_dois(&self) -> Box<dyn Iterator<Item = &DrawObjInner> + '_> {
        Box::new(self.0.iter())
    }

    pub fn mutate(&mut self, f: impl Fn(&mut Pt)) {
        for doi in &mut self.0 {
            doi.mutate(&f);
        }
    }
}

impl YieldPoints for Group {
    fn yield_pts(&self) -> Option<Box<dyn Iterator<Item = &Pt> + '_>> {
        Some(Box::new(
            self.0
                .iter()
                .flat_map(|doi| doi.inner_impl_yield_points().yield_pts())
                .flatten(),
        ))
    }
}
impl YieldPointsMut for Group {
    fn yield_pts_mut(&mut self) -> Option<Box<dyn Iterator<Item = &mut Pt> + '_>> {
        Some(Box::new(
            self.0
                .iter_mut()
                .flat_map(|doi| doi.inner_impl_yield_points_mut().yield_pts_mut())
                .flatten(),
        ))
    }
}

impl Bounded for Group {
    fn right_bound(&self) -> f64 {
        self.iter_dois()
            .map(|doi| FloatOrd(doi.right_bound()))
            .max()
            .unwrap()
            .0
    }
    fn left_bound(&self) -> f64 {
        self.iter_dois()
            .map(|doi| FloatOrd(doi.left_bound()))
            .min()
            .unwrap()
            .0
    }
    fn bottom_bound(&self) -> f64 {
        self.iter_dois()
            .map(|doi| FloatOrd(doi.bottom_bound()))
            .max()
            .unwrap()
            .0
    }
    fn top_bound(&self) -> f64 {
        self.iter_dois()
            .map(|doi| FloatOrd(doi.top_bound()))
            .min()
            .unwrap()
            .0
    }
}

impl AddAssign<Pt> for Group {
    fn add_assign(&mut self, rhs: Pt) {
        self.0.iter_mut().for_each(|o| {
            *o += rhs;
        });
    }
}

impl SubAssign<Pt> for Group {
    fn sub_assign(&mut self, rhs: Pt) {
        self.0.iter_mut().for_each(|o| {
            *o -= rhs;
        });
    }
}

impl Add<Pt> for Group {
    type Output = Self;
    fn add(self, rhs: Pt) -> Self::Output {
        Self::new(self.0.into_iter().map(|o| o + rhs))
    }
}
impl Sub<Pt> for Group {
    type Output = Self;
    fn sub(self, rhs: Pt) -> Self::Output {
        Self::new(self.0.into_iter().map(|o| o - rhs))
    }
}

impl Mul<f64> for Group {
    type Output = Self;
    fn mul(self, rhs: f64) -> Self::Output {
        Self::new(self.0.into_iter().map(|o| o * rhs))
    }
}

impl MulAssign<f64> for Group {
    fn mul_assign(&mut self, rhs: f64) {
        self.0.iter_mut().for_each(|o| {
            *o *= rhs;
        })
    }
}

impl Div<f64> for Group {
    type Output = Self;
    fn div(self, rhs: f64) -> Self::Output {
        Self::new(self.0.into_iter().map(|o| o / rhs))
    }
}

impl DivAssign<f64> for Group {
    fn div_assign(&mut self, rhs: f64) {
        self.0.iter_mut().for_each(|o| {
            *o /= rhs;
        })
    }
}

impl RemAssign<Pt> for Group {
    fn rem_assign(&mut self, rhs: Pt) {
        self.0.iter_mut().for_each(|o| *o %= rhs);
    }
}

impl Translatable for Group {}
impl Scalable<f64> for Group {}

impl Croppable for Group {
    type Output = Group;
    fn crop_to(&self, frame: &Polygon) -> Result<Vec<Self::Output>, CropToPolygonError> {
        Ok(vec![Group::new(
            self.0
                .iter()
                .flat_map(|d_o| d_o.crop_to(frame))
                .flatten()
                .collect::<Vec<_>>(),
        )])
    }
}
