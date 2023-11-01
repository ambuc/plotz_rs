//! Macros.
#![allow(missing_docs)]

/// impl<T> Trait<T> for Name
#[macro_export]
macro_rules! ops_generic {
    ($name:ident, $rhs: ident, $trait:ident, $fn:ident) => {
        impl<T> $trait<T> for $name
        where
            T: Into<$rhs>,
        {
            type Output = Self;
            fn $fn(self, rhs: T) -> Self::Output {
                let mut x = self.clone();
                let rhs: $rhs = rhs.into();
                x.iter_mut().for_each(|pt| *pt = pt.$fn(rhs));
                x
            }
        }
    };
}

/// impl<T> Trait<T> for &Name
#[macro_export]
macro_rules! ops_generic_ref {
    ($name:ident, $rhs:ident, $trait:ident, $fn:ident) => {
        impl<T> $trait<T> for &$name
        where
            T: Into<$rhs>,
        {
            type Output = $name;
            fn $fn(self, rhs: T) -> Self::Output {
                let mut x = self.clone();
                let rhs: $rhs = rhs.into();
                x.iter_mut().for_each(|pt| *pt = pt.$fn(rhs));
                x
            }
        }
    };
}

/// impl<T> TraitAssign<T> for Name
#[macro_export]
macro_rules! ops_assign_generic {
    ($name:ident, $rhs:ident, $trait:ident, $fn:ident) => {
        impl<T> $trait<T> for $name
        where
            T: Into<$rhs>,
        {
            fn $fn(&mut self, rhs: T) {
                let rhs: $rhs = rhs.into();
                self.iter_mut().for_each(|pt| pt.$fn(rhs));
            }
        }
    };
}

#[macro_export]
macro_rules! ops_defaults_t {
    ($name:ident, $rhs:ident) => {
        $crate::ops_generic!($name, $rhs, Add, add);
        $crate::ops_generic!($name, $rhs, Div, div);
        $crate::ops_generic!($name, $rhs, Mul, mul);
        $crate::ops_generic!($name, $rhs, Sub, sub);
        $crate::ops_generic_ref!($name, $rhs, Add, add);
        $crate::ops_generic_ref!($name, $rhs, Div, div);
        $crate::ops_generic_ref!($name, $rhs, Mul, mul);
        $crate::ops_generic_ref!($name, $rhs, Sub, sub);
        $crate::ops_assign_generic!($name, $rhs, AddAssign, add_assign);
        $crate::ops_assign_generic!($name, $rhs, DivAssign, div_assign);
        $crate::ops_assign_generic!($name, $rhs, SubAssign, sub_assign);
        $crate::ops_assign_generic!($name, $rhs, MulAssign, mul_assign);
        $crate::ops_assign_generic!($name, $rhs, RemAssign, rem_assign);
    };
}
