//! A 2D multiline.
#![allow(missing_docs)]

use super::{point::Point, polygon::Polygon, segment::Segment};
use crate::{
    bounded::{Bounded, Bounds},
    crop::{CropType, Croppable},
    obj2::ObjType2d,
    Object,
};
use anyhow::{anyhow, Result};
use float_ord::FloatOrd;
use std::{fmt::Debug, ops::*};

#[derive(Clone)]
pub struct Multiline {
    // we promise, by construction,
    // (1) sgs will |never| be empty.
    // (2) for each (sg_{i}, sg_{i+1}), sg_{i}.f == sg_{i+1}.i
    // (3) sgs.last().f != sgs.first().i
    // panicable offence. Don't make me panic!
    pub pts: Vec<Point>,
}

impl Debug for Multiline {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Ml").field("pts", &self.pts).finish()
    }
}

impl PartialEq for Multiline {
    fn eq(&self, other: &Self) -> bool {
        // no cycle detection necessary! this ain't |Pg|!
        self.pts == other.pts
    }
}

impl TryFrom<Vec<Segment>> for Multiline {
    type Error = anyhow::Error;

    fn try_from(value: Vec<Segment>) -> Result<Self> {
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
        Ok(Multiline { pts })
    }
}

impl TryFrom<Vec<Point>> for Multiline {
    type Error = anyhow::Error;

    fn try_from(value: Vec<Point>) -> Result<Self> {
        if value.is_empty() {
            return Err(anyhow!("Prospective ML was empty!"));
        }
        if value.first().unwrap() == value.last().unwrap() {
            return Err(anyhow!("Hey, multilines can't be cycles!"));
        }
        Ok(Multiline { pts: value })
    }
}

#[allow(non_snake_case)]
pub fn Multiline(a: impl IntoIterator<Item = impl Into<Point>>) -> Multiline {
    a.into_iter()
        .map(|x| x.into())
        .collect::<Vec<_>>()
        .try_into()
        .unwrap()
}

impl Multiline {
    pub fn to_segments(&self) -> Vec<Segment> {
        self.pts
            .iter()
            .zip(self.pts.iter().skip(1))
            .map(|(i, j)| Segment(*i, *j))
            .collect()
    }

    pub fn rotate(&mut self, about: &Point, by_rad: f64) {
        self.pts
            .iter_mut()
            .for_each(|pt| pt.rotate_inplace(about, by_rad))
    }
}

impl Croppable for Multiline {
    type Output = Multiline;

    fn crop(&self, _other: &Polygon, _crop_type: CropType) -> Result<Vec<Self::Output>> {
        todo!("https://github.com/ambuc/plotz_rs/issues/7")
    }
}

impl IntoIterator for Multiline {
    type Item = Point;
    type IntoIter = std::vec::IntoIter<Point>;

    fn into_iter(self) -> Self::IntoIter {
        self.pts.into_iter()
    }
}

crate::ops_defaults_t!(Multiline, Point);

impl Bounded for Multiline {
    fn bounds(&self) -> Result<Bounds> {
        Ok(Bounds {
            y_max: self
                .pts
                .iter()
                .map(|p| FloatOrd(p.y))
                .max()
                .ok_or(anyhow!("not empty"))?
                .0,
            y_min: self
                .pts
                .iter()
                .map(|p| FloatOrd(p.y))
                .min()
                .ok_or(anyhow!("not empty"))?
                .0,
            x_min: self
                .pts
                .iter()
                .map(|p| FloatOrd(p.x))
                .min()
                .ok_or(anyhow!("not empty"))?
                .0,
            x_max: self
                .pts
                .iter()
                .map(|p| FloatOrd(p.x))
                .max()
                .ok_or(anyhow!("not empty"))?
                .0,
        })
    }
}

impl Object for Multiline {
    fn objtype(&self) -> ObjType2d {
        ObjType2d::Multiline2d
    }

    fn iter(&self) -> Box<dyn Iterator<Item = &Point> + '_> {
        Box::new(self.pts.iter())
    }

    fn iter_mut(&mut self) -> Box<dyn Iterator<Item = &mut Point> + '_> {
        Box::new(self.pts.iter_mut())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use test_case::test_case;

    #[test_case(vec![Segment((0,0),(0,1))], Multiline{pts: vec![Point(0,0),Point(0,1)]}; "one link")]
    #[test_case(vec![Segment((0,0),(0,1)), Segment((0,1),(1,1))], Multiline{pts: vec![Point(0,0),Point(0,1),Point(1,1)]}; "two links")]
    // useful test because this self-intersects -- but is not a cycle.
    #[test_case(vec![Segment((0,0),(0,1)), Segment((0,1),(0,0)), Segment((0,0),(0,1))], Multiline{pts: vec![Point(0,0),Point(0,1),Point(0,0),Point(0,1)]}; "scribble")]
    fn test_try_from_vec_sg_should_succeed(val: Vec<Segment>, expected: Multiline) -> Result<()> {
        let actual: Multiline = val.try_into()?;
        assert_eq!(actual, expected);
        Ok(())
    }

    #[test_case(vec![]; "empty")]
    #[test_case(vec![Segment((0,0),(0,0))]; "no distance")]
    #[test_case(vec![Segment((0,0),(1,1)), Segment((1,1),(0,0))]; "cycle")]
    fn test_try_from_vec_sg_should_fail(val: Vec<Segment>) -> Result<()> {
        assert!(<Vec<Segment> as TryInto<Multiline>>::try_into(val).is_err());
        Ok(())
    }

    #[test_case(vec![Point(0,0), Point(0,1)]; "one link")]
    #[test_case(vec![Point(0,0), Point(0,1), Point(0,0), Point(0,1)]; "scribble")]
    #[test_case(vec![Point(0,0), Point(0,1), Point(1,1)]; "two links")]
    fn test_try_from_vec_pts_should_succeed(val: Vec<Point>) -> Result<()> {
        let _: Multiline = val.try_into()?;
        Ok(())
    }

    #[test_case(vec![]; "empty")]
    #[test_case(vec![Point(0,0)]; "one")]
    #[test_case(vec![Point(0,0), Point(1,1), Point(0,0)]; "cycle")]
    fn test_try_from_vec_pts_should_fail(val: Vec<Point>) -> Result<()> {
        assert!(<Vec<Point> as TryInto<Multiline>>::try_into(val).is_err());
        Ok(())
    }

    #[test]
    fn test_multiline_to_segments() -> Result<()> {
        {
            let ml: Multiline = vec![Point(0, 0), Point(0, 1)].try_into()?;
            assert_eq!(ml.to_segments(), [Segment((0, 0), (0, 1))]);
        }

        {
            let ml: Multiline = vec![Point(0, 0), Point(0, 1), Point(0, 2)].try_into()?;
            assert_eq!(
                ml.to_segments(),
                [Segment((0, 0), (0, 1)), Segment((0, 1), (0, 2))]
            );
        }

        Ok(())
    }
}
