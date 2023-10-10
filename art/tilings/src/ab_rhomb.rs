// https://tilings.math.uni-bielefeld.de/substitution/ammann-beenker-rhomb-triangle/

use plotz_geometry::{obj2::Obj2, style::Style};

use plotz_color::*;
use plotz_geometry::{
    shading::{shade_config::ShadeConfig, shade_polygon},
    shapes::{pg2::Pg2, pt2::Pt2},
};

struct T1([Pt2; 3]);
impl Tile for T1 {
    fn expand(&self) -> Vec<Box<dyn Tile>> {
        let sq2: f64 = 2.0_f64.sqrt();

        let T1([a, b, c]) = &self;
        let ab = *a + (*b - *a) / (1.0 + sq2) * sq2;
        let bc = *b + (*c - *b) / (1.0 + sq2) * sq2;
        let ac1 = *a + (*c - *a) / (2.0 + sq2) * 1.0;
        let ac2 = *a + (*c - *a) / (2.0 + sq2) * (1.0 + sq2);
        let x = bc - (*c - *a) / (2.0 + sq2) * 1.0;
        vec![
            Box::new(T1([ab, ac1, *a])),
            Box::new(T1([bc, x, *b])),
            Box::new(T2([ac2, x, ac1])),
            Box::new(T3([ac1, ab, *b, x])),
            Box::new(T3([x, bc, *c, ac2])),
        ]
    }
    fn color(&self) -> &'static ColorRGB {
        &WHITE
    }
    fn pts(&self) -> Vec<Pt2> {
        self.0.to_vec()
    }
    fn slope(&self) -> f64 {
        let T1([a, _b, c]) = &self;
        ((c.y - a.y) / (c.x - a.x)).atan()
    }
}
struct T2([Pt2; 3]);
impl Tile for T2 {
    fn expand(&self) -> Vec<Box<dyn Tile>> {
        let sq2: f64 = 2.0_f64.sqrt();

        let T2([a, b, c]) = self;
        let ab = *a + (*b - *a) / (1.0 + sq2) * sq2;
        let bc = *b + (*c - *b) / (1.0 + sq2) * sq2;
        let ac1 = *a + (*c - *a) / (2.0 + sq2) * 1.0;
        let ac2 = *a + (*c - *a) / (2.0 + sq2) * (1.0 + sq2);
        let x = bc - (*c - *a) / (2.0 + sq2) * 1.0;
        vec![
            Box::new(T1([ac2, x, ac1])),
            Box::new(T2([ab, ac1, *a])),
            Box::new(T2([bc, x, *b])),
            Box::new(T3([ac1, x, *b, ab])),
            Box::new(T3([x, ac2, *c, bc])),
        ]
    }
    fn color(&self) -> &'static ColorRGB {
        &WHITE
    }
    fn pts(&self) -> Vec<Pt2> {
        self.0.to_vec()
    }
    fn slope(&self) -> f64 {
        let T2([a, _b, c]) = &self;
        ((c.y - a.y) / (c.x - a.x)).atan()
    }
}
struct T3([Pt2; 4]);
impl Tile for T3 {
    fn expand(&self) -> Vec<Box<dyn Tile>> {
        let T3([a, b, c, d]) = self;
        let sq2 = 2.0_f64.sqrt();
        let ab = *a + (*b - *a) / (1.0 + sq2) * 1.0;
        let bc = *b + (*c - *b) / (1.0 + sq2) * sq2;
        let cd = *d + (*c - *d) / (1.0 + sq2) * sq2;
        let ad = *a + (*d - *a) / (1.0 + sq2) * 1.0;
        let xa = ad + (ab - *a);
        let xc = bc - (*c - cd);
        vec![
            Box::new(T1([ad, xa, *d])),
            Box::new(T1([bc, xc, *b])),
            Box::new(T2([ab, xa, *b])),
            Box::new(T2([cd, xc, *d])),
            Box::new(T3([*a, ab, xa, ad])),
            Box::new(T3([xc, bc, *c, cd])),
            Box::new(T3([*b, xc, *d, xa])),
        ]
    }
    fn color(&self) -> &'static ColorRGB {
        &BLUE
    }
    fn pts(&self) -> Vec<Pt2> {
        self.0.to_vec()
    }
    fn slope(&self) -> f64 {
        let T3([a, _b, c, _d]) = &self;
        ((c.y - a.y) / (c.x - a.x)).atan()
    }
}

trait Tile {
    fn expand(&self) -> Vec<Box<dyn Tile>>;
    fn color(&self) -> &'static ColorRGB;
    fn pts(&self) -> Vec<Pt2>;
    fn slope(&self) -> f64;
}

pub fn make() -> Vec<(Obj2, Style)> {
    let origin = Pt2(0.1, 0.1);

    let sq2: f64 = 2.0_f64.sqrt();
    let ell = 1.0;
    let x: f64 = ell / sq2;

    let _t1 = T1([origin, origin + Pt2(ell, ell), origin + Pt2(2.0 * ell, 0.0)]);
    let _t2 = T2([
        origin,
        origin + Pt2(ell, -1.0 * ell),
        origin + Pt2(2.0 * ell, 0.0),
    ]);
    let t3 = T3([
        origin,
        origin + Pt2(ell, 0.0),
        origin + Pt2(ell + x, -x),
        origin + Pt2(x, -x),
    ]);

    let mut all_tiles: Vec<Box<dyn Tile>> = vec![Box::new(t3)];

    for _ in 0..4 {
        let next_layer = all_tiles
            .iter()
            .flat_map(|tile| tile.expand())
            .collect::<Vec<_>>();
        all_tiles = next_layer;
    }

    let dos: Vec<_> = all_tiles
        .into_iter()
        .flat_map(|tile| {
            let color = tile.color();
            let mut p = Pg2(tile.pts());
            p *= Pt2(1.0, -1.0); // flip
            p *= 530.0;
            p += Pt2(-17.0, 240.0); // translate

            let config = ShadeConfig::builder()
                .gap(1.5)
                .slope(tile.slope())
                .switchback(true)
                .build();
            let segments = shade_polygon(&config, &p).unwrap();

            std::iter::empty().chain(segments.into_iter().map(|s| {
                (
                    Obj2::Sg2(s),
                    Style {
                        color,
                        ..Default::default()
                    },
                )
            }))
        })
        .collect();

    dos
    //
}
