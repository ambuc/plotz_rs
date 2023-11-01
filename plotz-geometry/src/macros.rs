//! Macros.
#![allow(missing_docs)]

/// impl<T> Trait<T> for Name
#[macro_export]
macro_rules! ops_generic {
    ($name:ident, $trait:ident, $fn:ident) => {
        impl<T> $trait<T> for $name
        where
            T: Into<Pt>,
        {
            type Output = Self;
            fn $fn(self, rhs: T) -> Self::Output {
                let mut x = self.clone();
                let rhs: Pt = rhs.into();
                x.iter_mut().for_each(|pt| *pt = pt.$fn(rhs));
                x
            }
        }
    };
}

/// impl<T> Trait<T> for &Name
#[macro_export]
macro_rules! ops_generic_ref {
    ($name:ident, $trait:ident, $fn:ident) => {
        impl<T> $trait<T> for &$name
        where
            T: Into<Pt>,
        {
            type Output = $name;
            fn $fn(self, rhs: T) -> Self::Output {
                let mut x = self.clone();
                let rhs: Pt = rhs.into();
                x.iter_mut().for_each(|pt| *pt = pt.$fn(rhs));
                x
            }
        }
    };
}

/// impl<T> TraitAssign<T> for Name
#[macro_export]
macro_rules! ops_assign_generic {
    ($name:ident, $trait:ident, $fn:ident) => {
        impl<T> $trait<T> for $name
        where
            T: Into<Pt>,
        {
            fn $fn(&mut self, rhs: T) {
                let rhs: Pt = rhs.into();
                self.iter_mut().for_each(|pt| pt.$fn(rhs));
            }
        }
    };
}

#[macro_export]
macro_rules! ops_defaults {
    ($name:ident) => {
        $crate::ops_generic!($name, Add, add);
        $crate::ops_generic!($name, Div, div);
        $crate::ops_generic!($name, Mul, mul);
        $crate::ops_generic!($name, Sub, sub);
        $crate::ops_generic_ref!($name, Add, add);
        $crate::ops_generic_ref!($name, Div, div);
        $crate::ops_generic_ref!($name, Mul, mul);
        $crate::ops_generic_ref!($name, Sub, sub);
        $crate::ops_assign_generic!($name, AddAssign, add_assign);
        $crate::ops_assign_generic!($name, DivAssign, div_assign);
        $crate::ops_assign_generic!($name, SubAssign, sub_assign);
        $crate::ops_assign_generic!($name, MulAssign, mul_assign);
        $crate::ops_assign_generic!($name, RemAssign, rem_assign);

        impl $crate::Translatable for $name {}
        impl $crate::Scalable<Pt> for $name {}
        impl $crate::Scalable<f64> for $name {}
    };
}
