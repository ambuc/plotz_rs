// https://tilings.math.uni-bielefeld.de/substitution/cromwell-kite-rhombus-trapezium/

use {
    plotz_color::*,
    plotz_geometry::{
        draw_obj::DrawObj,
        draw_obj_inner::DrawObjInner,
        group::Group,
        interpolate::extrapolate_2d as extrapolate,
        point::{PolarPt, Pt},
        polygon::Polygon,
        shading_02::{shade_polygon, ShadeConfig},
    },
    std::f64::consts::PI,
};

// kite, green
struct T1([Pt; 4]);
impl Tile for T1 {
    fn expand(&self) -> Vec<Box<dyn Tile>> {
        let T1([a, b, c, d]) = &self;

        let ell: f64 = (5.0_f64.sqrt() - 1.0) / 2.0;
        let x = 1.0_f64;

        let ab = extrapolate(*a, *b, ell / (ell + x));
        let ad = extrapolate(*a, *d, ell / (ell + x));
        let bc1 = extrapolate(*b, *c, x / (x + ell + x));
        let bc2 = extrapolate(*b, *c, (x + ell) / (x + ell + x));
        let cd1 = extrapolate(*c, *d, x / (x + ell + x));
        let cd2 = extrapolate(*c, *d, (x + ell) / (x + ell + x));
        let ac1 = extrapolate(*a, *c, ell / (ell + x + x));
        let ac2 = extrapolate(*a, *c, (ell + x) / (ell + x + x));
        let a_bc1 = extrapolate(*a, bc1, 1.0 / (1.0 + ell));
        let a_cd2 = extrapolate(*a, cd2, 1.0 / (1.0 + ell));

        vec![
            Box::new(T1([ac1, a_bc1, ac2, a_cd2])),
            {
                let mut t = T1([ac2, bc2, *c, cd1]);
                t.pts_iter_mut()
                    .for_each(|pt| pt.rotate_inplace(c, -1.0 * PI / 5.0));
                Box::new(t)
            },
            {
                let mut t = T1([a_bc1, ab, *b, bc1]);
                t.pts_iter_mut()
                    .for_each(|pt| pt.rotate_inplace(b, -1.0 * PI / 5.0));
                Box::new(t)
            },
            {
                let mut t = T1([a_cd2, cd2, *d, ad]);
                t.pts_iter_mut()
                    .for_each(|pt| pt.rotate_inplace(d, -1.0 * PI / 5.0));
                Box::new(t)
            },
            Box::new(T2([*a, ab, a_bc1, ac1])),
            Box::new(T2([*a, ac1, a_cd2, ad])),
            Box::new(T3([bc1, bc2, ac2, a_bc1])),
            Box::new(T4([cd2, cd1, ac2, a_cd2])),
        ]
    }
    fn color(&self) -> &'static ColorRGB {
        &GREEN
    }
    fn pts(&self) -> Vec<Pt> {
        self.0.to_vec()
    }
    fn pts_iter_mut<'a>(&mut self) -> Box<dyn Iterator<Item = &'_ mut Pt> + '_> {
        Box::new(self.0.iter_mut())
    }
}

// diamond, blue
struct T2([Pt; 4]);
impl Tile for T2 {
    fn expand(&self) -> Vec<Box<dyn Tile>> {
        let T2([a, b, c, d]) = &self;

        let ell: f64 = (5.0_f64.sqrt() - 1.0) / 2.0;
        let x = 1.0_f64;

        let ab = extrapolate(*a, *b, x / (x + ell));
        let bc = extrapolate(*b, *c, ell / (ell + x));
        let cd = extrapolate(*c, *d, x / (x + ell));
        let ad = extrapolate(*d, *a, ell / (ell + x));

        let ac1 = extrapolate(*a, *c, ell / (ell + x + x));
        let ac2 = extrapolate(*a, *c, (ell + x) / (ell + x + x));

        vec![
            Box::new(T1([ac1, ab, ac2, ad])),
            {
                let mut t = T1([ac2, bc, *c, cd]);
                t.pts_iter_mut()
                    .for_each(|pt| pt.rotate_inplace(c, -1.0 * PI / 5.0));
                Box::new(t)
            },
            {
                let mut td = *a + (ad - *a) / x * ell;
                td.rotate(a, -1.0 * 2.0 * PI / 10.0);
                Box::new(T2([*a, ac1, ad, td]))
            },
            Box::new(T3([*b, bc, ac2, ab])),
            Box::new(T4([*d, cd, ac2, ad])),
        ]
    }
    fn color(&self) -> &'static ColorRGB {
        &ORANGE
    }
    fn pts(&self) -> Vec<Pt> {
        self.0.to_vec()
    }
    fn pts_iter_mut<'a>(&mut self) -> Box<dyn Iterator<Item = &'_ mut Pt> + '_> {
        Box::new(self.0.iter_mut())
    }
}
// trapezoid, red
struct T3([Pt; 4]);
impl Tile for T3 {
    fn expand(&self) -> Vec<Box<dyn Tile>> {
        let T3([a, b, c, d]) = &self;

        let ell: f64 = (5.0_f64.sqrt() - 1.0) / 2.0;
        let x = 1.0_f64;

        let ab = extrapolate(*a, *b, x / (x + ell));
        let bc = extrapolate(*b, *c, ell / (ell + x));
        let cd1 = extrapolate(*c, *d, x / (x + ell + x));
        let cd2 = extrapolate(*c, *d, (x + ell) / (x + ell + x));
        let ad = extrapolate(*d, *a, x / (x + ell));

        let ac1 = extrapolate(*a, *c, ell / (ell + x + x));
        let ac2 = extrapolate(*a, *c, (ell + x) / (ell + x + x));

        let a_cd2 = extrapolate(*a, cd2, 1.0 / (1.0 + ell));

        vec![
            Box::new(T1([ac1, ab, ac2, a_cd2])),
            {
                let mut t = T1([ac2, bc, *c, cd1]);
                t.pts_iter_mut()
                    .for_each(|pt| pt.rotate_inplace(c, -1.0 * PI / 5.0));
                Box::new(t)
            },
            {
                let mut t = T1([a_cd2, cd2, *d, ad]);
                t.pts_iter_mut()
                    .for_each(|pt| pt.rotate_inplace(d, -1.0 * PI / 5.0));
                Box::new(t)
            },
            Box::new(T2([*a, ac1, a_cd2, ad])),
            // {
            //     let mut t = T2([*a, ac1, a_cd2, ad]);
            //     t.pts_iter_mut().for_each(|pt| pt.rotate(a, 2.0 * PI / 5.0));
            //     Box::new(t)
            // },
            Box::new(T3([*b, bc, ac2, ab])),
            Box::new(T4([cd2, cd1, ac2, a_cd2])),
        ]
    }
    fn color(&self) -> &'static ColorRGB {
        &RED
    }
    fn pts(&self) -> Vec<Pt> {
        self.0.to_vec()
    }
    fn pts_iter_mut<'a>(&mut self) -> Box<dyn Iterator<Item = &'_ mut Pt> + '_> {
        Box::new(self.0.iter_mut())
    }
}
// trapezoid, blue
struct T4([Pt; 4]);
impl Tile for T4 {
    fn expand(&self) -> Vec<Box<dyn Tile>> {
        let T4([a, b, c, d]) = &self;

        let ell: f64 = (5.0_f64.sqrt() - 1.0) / 2.0;
        let x = 1.0_f64;

        let ab = extrapolate(*a, *b, x / (x + ell));
        let bc = extrapolate(*b, *c, ell / (ell + x));
        let cd1 = extrapolate(*c, *d, x / (x + ell + x));
        let cd2 = extrapolate(*c, *d, (x + ell) / (x + ell + x));
        let ad = extrapolate(*d, *a, x / (x + ell));

        let ac1 = extrapolate(*a, *c, ell / (ell + x + x));
        let ac2 = extrapolate(*a, *c, (ell + x) / (ell + x + x));

        let a_cd2 = extrapolate(*a, cd2, 1.0 / (1.0 + ell));

        vec![
            Box::new(T1([ac1, a_cd2, ac2, ab])),
            {
                let mut t = T1([ac2, cd1, *c, bc]);
                t.pts_iter_mut()
                    .for_each(|pt| pt.rotate_inplace(c, -1.0 * PI / 5.0));
                Box::new(t)
            },
            {
                let mut t = T1([a_cd2, ad, *d, cd2]);
                t.pts_iter_mut()
                    .for_each(|pt| pt.rotate_inplace(d, -1.0 * PI / 5.0));
                Box::new(t)
            },
            Box::new(T2([*a, ad, a_cd2, ac1])),
            {
                let mut t = T2([*a, ad, a_cd2, ac1]);
                t.pts_iter_mut()
                    .for_each(|pt| pt.rotate_inplace(a, -2.0 * PI / 5.0));
                Box::new(t)
            },
            Box::new(T4([*b, bc, ac2, ab])),
            Box::new(T3([cd2, cd1, ac2, a_cd2])),
        ]
    }
    fn color(&self) -> &'static ColorRGB {
        &BLUE
    }
    fn pts(&self) -> Vec<Pt> {
        self.0.to_vec()
    }
    fn pts_iter_mut<'a>(&mut self) -> Box<dyn Iterator<Item = &'_ mut Pt> + '_> {
        Box::new(self.0.iter_mut())
    }
}

trait Tile {
    fn expand(&self) -> Vec<Box<dyn Tile>>;
    fn color(&self) -> &'static ColorRGB;
    fn pts(&self) -> Vec<Pt>;
    fn pts_iter_mut(&mut self) -> Box<dyn Iterator<Item = &'_ mut Pt> + '_>;
}

pub fn make() -> Vec<DrawObj> {
    let ell: f64 = (5.0_f64.sqrt() - 1.0) / 2.0;

    let t1 = {
        let a = Pt(0.0, 0.0);
        let b = a + PolarPt(ell, -1.0 * PI / 10.0);
        let c = Pt(0.0, -1.0);
        let d = a + PolarPt(ell, 11.0 * PI / 10.0);
        T1([a, b, c, d])
    };

    let _t2 = {
        let a = Pt(0.0, 0.0);
        let b = a + PolarPt(ell, -3.0 * PI / 10.0);
        let c = a + PolarPt(1.0, 15.0 * PI / 10.0);
        let d = a + PolarPt(ell, 13.0 * PI / 10.0);
        T2([a, b, c, d])
    };

    let _t3 = {
        let a = Pt(0.0, 0.0);
        let b = a + Pt(ell, 0.0);
        let c = b + PolarPt(ell, 16.0 * PI / 10.0);
        let d = c - Pt(1.0, 0.0);
        let mut t = T3([a, b, c, d]);
        t.pts_iter_mut()
            .for_each(|pt| pt.rotate_inplace(&a, -1.0 * 3.0 * PI / 10.0));
        t
    };

    let _t4 = {
        let a = Pt(0.0, 0.0);
        let b = a - Pt(ell, 0.0);
        let c = b + PolarPt(ell, 14.0 * PI / 10.0);
        let d = c + Pt(1.0, 0.0);
        let mut t = T4([a, b, c, d]);
        t.pts_iter_mut()
            .for_each(|pt| pt.rotate_inplace(&a, 1.0 * 3.0 * PI / 10.0));
        t
    };

    let mut all_tiles: Vec<Box<dyn Tile>> = vec![Box::new(t1)];

    for _ in 0..5 {
        let next_layer = all_tiles
            .iter()
            .flat_map(|tile: &Box<dyn Tile>| tile.expand())
            .collect::<Vec<_>>();
        all_tiles = next_layer;
    }

    all_tiles
        .into_iter()
        .flat_map(|tile| {
            let color = tile.color();
            let mut p = Polygon(tile.pts()).unwrap();
            p *= Pt(1.0, -1.0); // flip
            p *= 3500.0;
            p += Pt(95.0, -300.0); // translate

            let config = ShadeConfig::builder()
                .gap(1.5)
                .slope(0.0)
                .switchback(true)
                .build();
            let segments = shade_polygon(&config, &p).unwrap();

            // std::iter::empty() //
            std::iter::once(DrawObj::new(p).with_color(color)).chain([DrawObj::new(Group::new(
                segments.into_iter().map(DrawObjInner::Segment),
            ))
            .with_color(color)])
        })
        .collect()
}
