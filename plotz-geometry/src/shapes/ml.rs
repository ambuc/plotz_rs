//! A 2D multiline.
#![allow(missing_docs)]

use super::{pg::Pg, pt::Pt, sg::Sg, txt::Txt};
use crate::{
    bounded::{Bounded, Bounds},
    crop::{CropType, Croppable},
    intersection::IntersectionResult,
    obj::Obj,
    style::Style,
    Annotatable, AnnotationSettings, Nullable, Roundable, Scalable, Translatable,
};
use anyhow::{anyhow, Result};
use float_ord::FloatOrd;
use itertools::iproduct;
use std::{fmt::Debug, ops::*};

#[derive(Clone)]
pub struct Ml {
    // we promise, by construction,
    // (1) sgs will |never| be empty.
    // (2) for each (sg_{i}, sg_{i+1}), sg_{i}.f == sg_{i+1}.i
    // (3) sgs.last().f != sgs.first().i
    // panicable offence. Don't make me panic!
    pub pts: Vec<Pt>,
}

impl Debug for Ml {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Ml").field("pts", &self.pts).finish()
    }
}

impl PartialEq for Ml {
    fn eq(&self, other: &Self) -> bool {
        // no cycle detection necessary! this ain't |Pg|!
        self.pts == other.pts
    }
}

impl TryFrom<Vec<Sg>> for Ml {
    type Error = anyhow::Error;

    fn try_from(value: Vec<Sg>) -> Result<Self> {
        let mut pts = vec![];
        if value.is_empty() {
            return Err(anyhow!("Prospective ML was empty!"));
        }
        if value.first().unwrap().i == value.last().unwrap().f {
            return Err(anyhow!("Hey, multilines can't be cycles!"));
        }
        for (i, j) in value.iter().zip(value.iter().skip(1)) {
            if i.f != j.i {
                return Err(anyhow!("Hey, multilines are supposed to be chains!"));
            }
        }
        for sg in &value {
            pts.push(sg.i);
        }
        pts.push(value.last().unwrap().f);
        Ok(Ml { pts })
    }
}

impl TryFrom<Vec<Pt>> for Ml {
    type Error = anyhow::Error;

    fn try_from(value: Vec<Pt>) -> Result<Self> {
        if value.is_empty() {
            return Err(anyhow!("Prospective ML was empty!"));
        }
        if value.first().unwrap() == value.last().unwrap() {
            return Err(anyhow!("Hey, multilines can't be cycles!"));
        }
        Ok(Ml { pts: value })
    }
}

#[allow(non_snake_case)]
pub fn Ml(a: impl IntoIterator<Item = impl Into<Pt>>) -> Ml {
    a.into_iter()
        .map(|x| x.into())
        .collect::<Vec<_>>()
        .try_into()
        .unwrap()
}

impl Ml {
    pub fn to_segments(&self) -> Vec<Sg> {
        self.pts
            .iter()
            .zip(self.pts.iter().skip(1))
            .map(|(i, j)| Sg(*i, *j))
            .collect()
    }

    pub fn rotate(&mut self, about: &Pt, by_rad: f64) {
        self.pts
            .iter_mut()
            .for_each(|pt| pt.rotate_inplace(about, by_rad))
    }

    pub fn intersects(&self, other: &Self) -> bool {
        self.intersects_detailed(other).count() != 0
    }

    pub fn intersects_detailed(&self, other: &Self) -> impl Iterator<Item = IntersectionResult> {
        iproduct!(self.to_segments(), other.to_segments()).flat_map(|(l1, l2)| l1.intersects(&l2))
    }

    pub fn intersects_segment(&self, other: &Sg) -> bool {
        self.to_segments()
            .iter()
            .any(|l: &Sg| l.intersects(other).is_some())
    }

    pub fn intersects_segment_detailed(&self, other: &Sg) -> Vec<IntersectionResult> {
        self.to_segments()
            .iter()
            .flat_map(|l: &Sg| l.intersects(other))
            .collect::<Vec<_>>()
    }

    pub fn iter(&self) -> impl Iterator<Item = &Pt> {
        self.pts.iter()
    }

    /// Mutable iterator.
    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut Pt> {
        self.pts.iter_mut()
    }
}

impl Croppable for Ml {
    type Output = Ml;

    fn crop(&self, _other: &Pg, _crop_type: CropType) -> Result<Vec<Self::Output>> {
        todo!("https://github.com/ambuc/plotz_rs/issues/7")
    }
}

impl IntoIterator for Ml {
    type Item = Pt;
    type IntoIter = std::vec::IntoIter<Pt>;

    fn into_iter(self) -> Self::IntoIter {
        self.pts.into_iter()
    }
}

crate::ops_defaults!(Ml);

impl Bounded for Ml {
    fn bounds(&self) -> Result<Bounds> {
        Ok(Bounds {
            top_bound: self
                .pts
                .iter()
                .map(|p| FloatOrd(p.y))
                .max()
                .ok_or(anyhow!("not empty"))?
                .0,
            bottom_bound: self
                .pts
                .iter()
                .map(|p| FloatOrd(p.y))
                .min()
                .ok_or(anyhow!("not empty"))?
                .0,
            left_bound: self
                .pts
                .iter()
                .map(|p| FloatOrd(p.x))
                .min()
                .ok_or(anyhow!("not empty"))?
                .0,
            right_bound: self
                .pts
                .iter()
                .map(|p| FloatOrd(p.x))
                .max()
                .ok_or(anyhow!("not empty"))?
                .0,
        })
    }
}

impl Translatable for Ml {}
impl Scalable<Pt> for Ml {}
impl Scalable<f64> for Ml {}

impl Roundable for Ml {
    fn round_to_nearest(&mut self, f: f64) {
        self.pts.iter_mut().for_each(|pt| pt.round_to_nearest(f));
    }
}

impl Nullable for Ml {
    fn is_empty(&self) -> bool {
        self.pts.is_empty()
    }
}

impl Annotatable for Ml {
    fn annotate(&self, settings: &AnnotationSettings) -> Vec<(Obj, Style)> {
        let mut a = vec![];

        let AnnotationSettings {
            font_size,
            precision,
        } = settings;
        for (_idx, pt) in self.pts.iter().enumerate() {
            let x = format!("{:.1$}", pt.x, precision);
            let y = format!("{:.1$}", pt.y, precision);
            a.push((
                Txt {
                    pt: *pt,
                    inner: format!("({}, {})", x, y),
                    font_size: *font_size,
                }
                .into(),
                Style::default(),
            ));
        }

        a
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use test_case::test_case;

    #[test_case(vec![Sg((0,0),(0,1))], Ml{pts: vec![Pt(0,0),Pt(0,1)]}; "one link")]
    #[test_case(vec![Sg((0,0),(0,1)), Sg((0,1),(1,1))], Ml{pts: vec![Pt(0,0),Pt(0,1),Pt(1,1)]}; "two links")]
    // useful test because this self-intersects -- but is not a cycle.
    #[test_case(vec![Sg((0,0),(0,1)), Sg((0,1),(0,0)), Sg((0,0),(0,1))], Ml{pts: vec![Pt(0,0),Pt(0,1),Pt(0,0),Pt(0,1)]}; "scribble")]
    fn test_try_from_vec_sg_should_succeed(val: Vec<Sg>, expected: Ml) -> Result<()> {
        let actual: Ml = val.try_into()?;
        assert_eq!(actual, expected);
        Ok(())
    }

    #[test_case(vec![]; "empty")]
    #[test_case(vec![Sg((0,0),(0,0))]; "no distance")]
    #[test_case(vec![Sg((0,0),(1,1)), Sg((1,1),(0,0))]; "cycle")]
    fn test_try_from_vec_sg_should_fail(val: Vec<Sg>) -> Result<()> {
        assert!(<Vec<Sg> as TryInto<Ml>>::try_into(val).is_err());
        Ok(())
    }

    #[test_case(vec![Pt(0,0), Pt(0,1)]; "one link")]
    #[test_case(vec![Pt(0,0), Pt(0,1), Pt(0,0), Pt(0,1)]; "scribble")]
    #[test_case(vec![Pt(0,0), Pt(0,1), Pt(1,1)]; "two links")]
    fn test_try_from_vec_pts_should_succeed(val: Vec<Pt>) -> Result<()> {
        let _: Ml = val.try_into()?;
        Ok(())
    }

    #[test_case(vec![]; "empty")]
    #[test_case(vec![Pt(0,0)]; "one")]
    #[test_case(vec![Pt(0,0), Pt(1,1), Pt(0,0)]; "cycle")]
    fn test_try_from_vec_pts_should_fail(val: Vec<Pt>) -> Result<()> {
        assert!(<Vec<Pt> as TryInto<Ml>>::try_into(val).is_err());
        Ok(())
    }

    #[test]
    fn test_multiline_to_segments() -> Result<()> {
        {
            let ml: Ml = vec![Pt(0, 0), Pt(0, 1)].try_into()?;
            assert_eq!(ml.to_segments(), [Sg((0, 0), (0, 1))]);
        }

        {
            let ml: Ml = vec![Pt(0, 0), Pt(0, 1), Pt(0, 2)].try_into()?;
            assert_eq!(ml.to_segments(), [Sg((0, 0), (0, 1)), Sg((0, 1), (0, 2))]);
        }

        Ok(())
    }
}
