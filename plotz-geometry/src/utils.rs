use std::cmp::Ordering;

use anyhow::{anyhow, Result};
use float_cmp::approx_eq;
use float_ord::FloatOrd;

#[derive(Debug, Copy, Clone)]

pub enum Which {
    A,
    B,
}
impl Which {
    pub fn flip(&self) -> Which {
        match self {
            Which::A => Which::B,
            Which::B => Which::A,
        }
    }
}

pub struct Pair<'a, T> {
    pub a: &'a T,
    pub b: &'a T,
}

impl<'a, T> Pair<'a, T> {
    pub fn get(&'a self, which: Which) -> &'a T {
        match which {
            Which::A => self.a,
            Which::B => self.b,
        }
    }
}

#[derive(Debug, PartialEq, Copy, Clone)]
/// Guaranteed to be 0.0 <= f <= 1.0. Witness type.
pub enum Percent {
    Zero,
    Val(f64),
    One,
}
impl Percent {
    pub fn new(f: f64) -> Result<Percent> {
        match f {
            f if approx_eq!(f64, f, 0.0) => Ok(Percent::Zero),
            f if approx_eq!(f64, f, 1.0) => Ok(Percent::One),
            f if (0.0..=1.0).contains(&f) => Ok(Percent::Val(f)),
            _ => Err(anyhow!("f not in 0.0..=1.0")),
        }
    }

    pub fn as_f64(&self) -> f64 {
        match self {
            Percent::Zero => 0.0,
            Percent::Val(f) => *f,
            Percent::One => 1.0,
        }
    }

    pub fn is_at_boundary(&self) -> bool {
        match self {
            Percent::Zero | Percent::One => true,
            Percent::Val(_) => false,
        }
    }
}

impl Eq for Percent {}

impl PartialOrd for Percent {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(match (self, other) {
            (Percent::Zero, Percent::Zero) => Ordering::Equal,
            (Percent::Zero, Percent::Val(_)) => Ordering::Less,
            (Percent::Zero, Percent::One) => Ordering::Less,
            (Percent::Val(_), Percent::Zero) => Ordering::Greater,
            (Percent::Val(v1), Percent::Val(v2)) => FloatOrd(*v1).cmp(&FloatOrd(*v2)),
            (Percent::Val(_), Percent::One) => Ordering::Less,
            (Percent::One, Percent::Zero) => Ordering::Greater,
            (Percent::One, Percent::Val(_)) => Ordering::Greater,
            (Percent::One, Percent::One) => Ordering::Equal,
        })
    }
}

impl Ord for Percent {
    fn cmp(&self, other: &Self) -> Ordering {
        self.partial_cmp(other).unwrap()
    }
    //
}
