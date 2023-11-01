//! Macros.
#![allow(missing_docs)]

#[macro_export]
macro_rules! ops_generic {
    ($name:ident, $trait:ident, $fn:ident) => {
        impl<T> $trait<T> for $name
        where
            T: Into<Pt> + Copy,
        {
            type Output = Self;
            fn $fn(self, rhs: T) -> Self::Output {
                let mut x = self.clone();
                x.iter_mut().for_each(|pt| *pt = pt.$fn(rhs.into()));
                x
            }
        }

        impl<T> $trait<T> for &$name
        where
            T: Into<Pt> + Copy,
        {
            type Output = $name;
            fn $fn(self, rhs: T) -> Self::Output {
                let mut x = self.clone();
                x.iter_mut().for_each(|pt| *pt = pt.$fn(rhs.into()));
                x
            }
        }
    };
}
#[macro_export]
macro_rules! ops_assign_generic {
    ($name:ident, $trait:ident, $fn:ident) => {
        impl<T> $trait<T> for $name
        where
            T: Into<Pt> + Copy,
        {
            fn $fn(&mut self, rhs: T) {
                self.iter_mut().for_each(|pt| pt.$fn(rhs.into()));
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
        $crate::ops_assign_generic!($name, AddAssign, add_assign);
        $crate::ops_assign_generic!($name, DivAssign, div_assign);
        $crate::ops_assign_generic!($name, SubAssign, sub_assign);
        $crate::ops_assign_generic!($name, MulAssign, mul_assign);
        $crate::ops_assign_generic!($name, RemAssign, rem_assign);
    };
}
