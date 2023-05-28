//! Grid (for debugging, mostly)

use {
    crate::{
        p2,
        shapes::{pt2::Pt2, sg2::Sg2},
        styled_obj2::StyledObj2,
    },
    num::range_step,
    plotz_color::*,
    typed_builder::TypedBuilder,
};

#[derive(Debug, TypedBuilder)]
/// A grid.
pub struct Grid {
    /// top-left coordinate of grid.
    #[builder(default = 0)]
    x_init: u64,
    #[builder(default = 0)]
    y_init: u64,
    /// width of grid.
    width: u64,
    /// height of grid.
    height: u64,
    /// minor grid marker.
    #[builder(default = 5)]
    minor_every: u64,
    /// major grid marker.
    #[builder(default = 25)]
    major_every: u64,
    /// minor thickness.
    #[builder(default = 0.75)]
    minor_thickness: f64,
    /// major thickness.
    #[builder(default = 1.50)]
    major_thickness: f64,
    #[builder(default = &GRAY)]
    minor_color: &'static ColorRGB,
    #[builder(default = &BLACK)]
    major_color: &'static ColorRGB,
}

impl Grid {
    /// Renders the grid to a set of object2ds for plotting.
    pub fn to_segments(&self) -> Vec<StyledObj2> {
        let h = self.height as f64;
        let w = self.width as f64;

        let mut v = vec![];
        for x in range_step(self.x_init, self.x_init + self.width, self.minor_every) {
            let i = p2!((self.x_init + x) as f64, (self.y_init) as f64);
            let f = i + p2!(0.0, h);
            v.push(
                StyledObj2::new(Sg2(i, f))
                    .with_color(self.minor_color)
                    .with_thickness(self.minor_thickness),
            );
        }
        for x in range_step(self.x_init, self.x_init + self.width, self.major_every) {
            let i = p2!((self.x_init + x) as f64, (self.y_init) as f64);
            let f = i + p2!(0.0, h);
            v.push(
                StyledObj2::new(Sg2(i, f))
                    .with_color(self.major_color)
                    .with_thickness(self.major_thickness),
            );
        }
        for y in range_step(self.y_init, self.y_init + self.height, self.minor_every) {
            let i = p2!((self.x_init) as f64, (self.y_init + y) as f64);
            let f = i + p2!(w, 0.0);
            v.push(
                StyledObj2::new(Sg2(i, f))
                    .with_color(self.minor_color)
                    .with_thickness(self.minor_thickness),
            )
        }
        for y in range_step(self.y_init, self.y_init + self.height, self.major_every) {
            let i = p2!((self.x_init) as f64, (self.y_init + y) as f64);
            let f = i + p2!(w, 0.0);
            v.push(
                StyledObj2::new(Sg2(i, f))
                    .with_color(self.major_color)
                    .with_thickness(self.major_thickness),
            )
        }
        v
    }
}
