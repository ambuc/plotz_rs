//! A group of objects.

use {
    crate::{
        bounded::{Bounded, Bounds, BoundsCollector},
        crop::{CropType, Croppable},
        shapes::{pg2::Pg2, pt2::Pt2},
        styled_obj2::StyledObj2,
        traits::*,
    },
    std::ops::*,
};

#[derive(Debug, PartialEq, Clone)]
/// A group of objects.
pub struct Group<T>(Vec<T>);

impl<T> Group<T> {
    /// Creates a new group.
    pub fn new(objs: impl IntoIterator<Item = T>) -> Group<T> {
        Group(objs.into_iter().collect::<Vec<_>>())
    }

    /// Returns a boxed iterator of immutable Object2dInners, the members of this
    /// group.
    pub fn iter_objects(&self) -> Box<dyn Iterator<Item = &T> + '_> {
        Box::new(self.0.iter())
    }

    /// Mutates each point in each object in the group. See |Mutable|.
    pub fn mutate(&mut self, f: impl Fn(&mut Pt2))
    where
        T: Mutable,
    {
        for obj in &mut self.0 {
            obj.mutate(&f);
        }
    }
}

impl<T> YieldPoints for Group<T>
where
    T: YieldPoints,
{
    fn yield_pts(&self) -> Box<dyn Iterator<Item = &Pt2> + '_> {
        Box::new(self.0.iter().flat_map(|obj| obj.yield_pts()))
    }
}
impl<T> YieldPointsMut for Group<T>
where
    T: YieldPointsMut,
{
    fn yield_pts_mut(&mut self) -> Box<dyn Iterator<Item = &mut Pt2> + '_> {
        Box::new(self.0.iter_mut().flat_map(|obj| obj.yield_pts_mut()))
    }
}

impl<T> Bounded for Group<T>
where
    T: Bounded + YieldPoints,
{
    fn bounds(&self) -> Bounds {
        let mut bc = BoundsCollector::default();
        for pt in self.yield_pts() {
            bc.incorporate(pt);
        }
        bc.bounds()
    }
}

impl<T> AddAssign<Pt2> for Group<T>
where
    T: AddAssign<Pt2>,
{
    fn add_assign(&mut self, rhs: Pt2) {
        self.0.iter_mut().for_each(|o| {
            *o += rhs;
        });
    }
}

impl<T> SubAssign<Pt2> for Group<T>
where
    T: SubAssign<Pt2>,
{
    fn sub_assign(&mut self, rhs: Pt2) {
        self.0.iter_mut().for_each(|o| {
            *o -= rhs;
        });
    }
}

impl<T> Add<Pt2> for Group<T>
where
    T: Add<Pt2, Output = T>,
{
    type Output = Self;
    fn add(self, rhs: Pt2) -> Self::Output {
        Self::new(self.0.into_iter().map(|o| o + rhs))
    }
}
impl<T> Sub<Pt2> for Group<T>
where
    T: Sub<Pt2, Output = T>,
{
    type Output = Self;
    fn sub(self, rhs: Pt2) -> Self::Output {
        Self::new(self.0.into_iter().map(|o| o - rhs))
    }
}

impl<T> Mul<f64> for Group<T>
where
    T: Mul<f64, Output = T>,
{
    type Output = Self;
    fn mul(self, rhs: f64) -> Self::Output {
        Self::new(self.0.into_iter().map(|o| o * rhs))
    }
}

impl<T> MulAssign<f64> for Group<T>
where
    T: MulAssign<f64>,
{
    fn mul_assign(&mut self, rhs: f64) {
        self.0.iter_mut().for_each(|o| {
            *o *= rhs;
        })
    }
}

impl<T> Div<f64> for Group<T>
where
    T: Div<f64, Output = T>,
{
    type Output = Self;
    fn div(self, rhs: f64) -> Self::Output {
        Self::new(self.0.into_iter().map(|o| o / rhs))
    }
}

impl<T> DivAssign<f64> for Group<T>
where
    T: DivAssign<f64>,
{
    fn div_assign(&mut self, rhs: f64) {
        self.0.iter_mut().for_each(|o| {
            *o /= rhs;
        })
    }
}

impl<T> RemAssign<Pt2> for Group<T>
where
    T: RemAssign<Pt2>,
{
    fn rem_assign(&mut self, rhs: Pt2) {
        self.0.iter_mut().for_each(|o| *o %= rhs);
    }
}

impl<T> Translatable for Group<T> where
    T: Add<Pt2, Output = T> + AddAssign<Pt2> + Sub<Pt2, Output = T> + SubAssign<Pt2> + Sized
{
}

impl<T> Scalable<f64> for Group<T> where
    T: Mul<f64, Output = T> + MulAssign<f64> + Div<f64, Output = T> + DivAssign<f64> + Sized
{
}

impl<T> Croppable for Group<T>
where
    T: Croppable<Output = T>,
{
    type Output = Group<T>;
    fn crop(&self, frame: &Pg2, crop_type: CropType) -> Vec<Self::Output> {
        vec![Group::new(
            self.0
                .iter()
                .flat_map(|d_o| d_o.crop(frame, crop_type))
                .collect::<Vec<_>>(),
        )]
    }
    fn crop_excluding(&self, _other: &Pg2) -> Vec<Self::Output>
    where
        Self: Sized,
    {
        unimplemented!("TODO");
    }
}

impl<T> Nullable for Group<T> {
    fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

impl<T> Annotatable for Group<T>
where
    T: Annotatable,
{
    fn annotate(&self, settings: &AnnotationSettings) -> Vec<StyledObj2> {
        self.0.iter().flat_map(|o| o.annotate(settings)).collect()
    }
}
