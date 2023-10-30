//! A 2D multiline.
#![allow(missing_docs)]

use super::{pt::Pt, sg::Sg};
use anyhow::{anyhow, Result};
use std::fmt::Debug;

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
}
