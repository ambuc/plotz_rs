use float_cmp::approx_eq;

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
    pub fn new(f: f64) -> Option<Percent> {
        match f {
            f if approx_eq!(f64, f, 0.0) => Some(Percent::Zero),
            f if approx_eq!(f64, f, 1.0) => Some(Percent::One),
            f if (0.0..=1.0).contains(&f) => Some(Percent::Val(f)),
            _ => None,
        }
    }

    pub fn as_f64(&self) -> f64 {
        match self {
            Percent::Zero => 0.0,
            Percent::Val(f) => *f,
            Percent::One => 1.0,
        }
    }
}
