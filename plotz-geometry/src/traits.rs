//! Traits
#![allow(dead_code)]
#![allow(unused)]
#![allow(missing_docs)]

use {crate::point::Pt, std::ops::*};

pub trait YieldPoints {
    fn yield_pts(&self) -> Option<Box<dyn Iterator<Item = &Pt> + '_>>;
}

pub trait YieldPointsMut {
    fn yield_pts_mut(&mut self) -> Option<Box<dyn Iterator<Item = &mut Pt> + '_>>;
}

pub trait Mutable: YieldPointsMut {
    fn mutate(&mut self, f: impl Fn(&mut Pt)) -> bool {
        if let Some(yp) = self.yield_pts_mut() {
            yp.for_each(f);
            return true;
        }
        false
    }
}

pub trait Translatable: Add<Pt> + AddAssign<Pt> + Sub<Pt> + SubAssign<Pt> + Sized {}

pub trait Scalable<T>: Mul<T> + MulAssign<T> + Div<T> + DivAssign<T> + Sized {}

pub trait TranslatableAssign: AddAssign<Pt> + SubAssign<Pt> {}
pub trait ScalableAssign: MulAssign<f64> + DivAssign<f64> {}

pub trait Roundable {
    fn round_to_nearest(&mut self, f: f64);
}
