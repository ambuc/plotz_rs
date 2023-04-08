//! GridLayout for grid...layouts... what do you want from me.

use crate::{bounded::Bounded, point::Pt};

use {
    crate::{bounded::Bounds, crop::Croppable, draw_obj::DrawObj},
    typed_builder::TypedBuilder,
};

#[derive(Debug, TypedBuilder, Copy, Clone)]
/// Settings struct.
pub struct GridLayoutSettings {
    /// x coordinate of top-left of grid.
    #[builder(default = 0)]
    x_init: u64,
    /// y coordinate of top-left of grid.
    #[builder(default = 0)]
    y_init: u64,
    /// total width of grid.
    total_width: u64,
    /// total height of grid.
    total_height: u64,
    /// the number of width divisions.
    x_divisions: usize,
    /// the number of height divisions.
    y_divisions: usize,
    /// the x margin around each object.
    object_margin_x: u64,
    /// the y margin around each object.
    object_margin_y: u64,
}

#[derive(Debug)]
/// A grid layout of aligned boxes.
pub struct GridLayout {
    /// the settings. See above.
    settings: GridLayoutSettings,
    /// A vector of objects. By default these will be empty vectors.
    objs: Vec<Vec<Vec<DrawObj>>>,
}

impl GridLayout {
    /// Creates a new GridLayout.
    pub fn new(settings: GridLayoutSettings) -> GridLayout {
        GridLayout {
            settings,
            objs: vec![
                // row
                vec![vec![]]
                    .into_iter()
                    .cycle()
                    .take(settings.y_divisions)
                    .collect::<_>(),
            ]
            .into_iter()
            .cycle()
            .take(settings.x_divisions)
            .collect::<_>(),
        }
    }

    /// Number of horizontal cubby divisions.
    pub fn num_cubbys_x(&self) -> usize {
        self.settings.x_divisions
    }
    /// Number of vertical cubby divisions.
    pub fn num_cubbys_y(&self) -> usize {
        self.settings.y_divisions
    }

    /// Get the bounds of a cubby at (i,j).
    pub fn get_cubby_bounds(&self, (i, j): (usize, usize)) -> Bounds {
        let cubby_width: f64 =
            (self.settings.total_width / (self.settings.x_divisions as u64)) as f64;
        let cubby_height: f64 =
            (self.settings.total_height / (self.settings.y_divisions as u64)) as f64;
        Bounds {
            top_bound: self.settings.y_init as f64 + (j + 1) as f64 * cubby_height
                - self.settings.object_margin_y as f64,
            bottom_bound: self.settings.y_init as f64
                + j as f64 * cubby_height
                + self.settings.object_margin_y as f64,
            left_bound: self.settings.x_init as f64
                + i as f64 * cubby_width
                + self.settings.object_margin_x as f64,
            right_bound: self.settings.x_init as f64 + (i + 1) as f64 * cubby_width
                - self.settings.object_margin_y as f64,
        }
    }

    /// Returns the center of the cubby.
    pub fn cubby_ctr(&self, (i, j): (usize, usize)) -> Pt {
        let bounds = self.get_cubby_bounds((i, j));
        bounds.bbox_center()
    }

    /// Returns a list of all inner objects.
    pub fn to_draw_obj(&self) -> Vec<DrawObj> {
        self.objs
            .clone()
            .into_iter()
            .flatten()
            .flatten()
            .collect::<Vec<_>>()
    }

    /// Given a DrawObj, crops it to the cubby at objs[i][j] and inserts that
    /// into the grid.
    pub fn insert_and_crop_to_cubby(&mut self, (i, j): (usize, usize), d_o: DrawObj) {
        let cropped = d_o
            .crop_to_bounds(self.get_cubby_bounds((i, j)))
            .expect("crop failed");

        self.objs[i][j].extend(cropped);
    }
}
