//! A group of like things in 3d.

use {crate::shapes::pt3::Pt3, std::ops::*};

#[derive(Debug, Clone)]
pub struct Group<T> {
    pub items: Vec<T>,
}

#[allow(non_snake_case)]
pub fn Group<T>(a: impl IntoIterator<Item = T>) -> Group<T> {
    Group {
        items: a.into_iter().collect(),
    }
}

impl<T> IntoIterator for Group<T> {
    type Item = T;
    type IntoIter = std::vec::IntoIter<T>;

    fn into_iter(self) -> Self::IntoIter {
        self.items.into_iter()
    }
}

impl<T> Add<Pt3> for Group<T>
where
    T: Add<Pt3, Output = T>,
{
    type Output = Group<T>;
    fn add(self, rhs: Pt3) -> Self::Output {
        Group {
            items: self.items.into_iter().map(|i| i + rhs).collect(),
            ..self
        }
    }
}
impl<T> AddAssign<Pt3> for Group<T>
where
    T: AddAssign<Pt3>,
{
    fn add_assign(&mut self, rhs: Pt3) {
        self.items.iter_mut().for_each(|p| *p += rhs);
    }
}
impl<T> Div<Pt3> for Group<T>
where
    T: Div<Pt3, Output = T>,
{
    type Output = Group<T>;
    fn div(self, rhs: Pt3) -> Self::Output {
        Group {
            items: self.items.into_iter().map(|p| p / rhs).collect(),
            ..self
        }
    }
}
impl<T> Div<f64> for Group<T>
where
    T: Div<f64, Output = T>,
{
    type Output = Group<T>;
    fn div(self, rhs: f64) -> Self::Output {
        Group {
            items: self.items.into_iter().map(|p| p / rhs).collect(),
            ..self
        }
    }
}
impl<T> DivAssign<Pt3> for Group<T>
where
    T: DivAssign<Pt3>,
{
    fn div_assign(&mut self, rhs: Pt3) {
        self.items.iter_mut().for_each(|p| *p /= rhs);
    }
}
impl<T> DivAssign<f64> for Group<T>
where
    T: DivAssign<f64>,
{
    fn div_assign(&mut self, rhs: f64) {
        self.items.iter_mut().for_each(|p| *p /= rhs);
    }
}
impl<T> Mul<Pt3> for Group<T>
where
    T: Mul<Pt3, Output = T>,
{
    type Output = Group<T>;
    fn mul(self, rhs: Pt3) -> Self::Output {
        Group {
            items: self.items.into_iter().map(|p| p * rhs).collect(),
            ..self
        }
    }
}
impl<T> Mul<f64> for Group<T>
where
    T: Mul<f64, Output = T>,
{
    type Output = Group<T>;
    fn mul(self, rhs: f64) -> Self::Output {
        Group {
            items: self.items.into_iter().map(|i| i * rhs).collect(),
            ..self
        }
    }
}
impl<T> MulAssign<Pt3> for Group<T>
where
    T: MulAssign<Pt3>,
{
    fn mul_assign(&mut self, rhs: Pt3) {
        self.items.iter_mut().for_each(|p| *p *= rhs);
    }
}
impl<T> MulAssign<f64> for Group<T>
where
    T: MulAssign<f64>,
{
    fn mul_assign(&mut self, rhs: f64) {
        self.items.iter_mut().for_each(|p| *p *= rhs);
    }
}
impl<T> Sub<Pt3> for Group<T>
where
    T: Sub<Pt3, Output = T>,
{
    type Output = Group<T>;
    fn sub(self, rhs: Pt3) -> Self::Output {
        Group {
            items: self.items.into_iter().map(|p| p - rhs).collect(),
            ..self
        }
    }
}
impl<T> SubAssign<Pt3> for Group<T>
where
    T: SubAssign<Pt3>,
{
    fn sub_assign(&mut self, rhs: Pt3) {
        self.items.iter_mut().for_each(|p| *p -= rhs);
    }
}
