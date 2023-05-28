//! GridLayout for grid...layouts... what do you want from me.

use {
    crate::{
        bounded::{Bounded, Bounds},
        crop::Croppable,
        shapes::pt2::Pt,
        styled_obj2::StyledObj2,
    },
    float_ord::FloatOrd,
    typed_builder::TypedBuilder,
};

#[derive(Debug, TypedBuilder, Copy, Clone)]
/// Settings struct.
pub struct GridLayoutSettings {
    /// coordinates of top-left of grid.
    #[builder(default = (0,0))]
    init: (u64, u64),
    /// total (width, height) of grid.
    dims: (u64, u64),
    /// the number of (x,y) divisions.
    divisions: (usize, usize),
    /// the (x,y) margin around each object.
    object_margin: (u64, u64),
}

#[derive(Debug)]
/// A grid layout of aligned boxes.
pub struct GridLayout {
    /// the settings. See above.
    settings: GridLayoutSettings,
    /// A vector of objects. By default these will be empty vectors.
    objs: Vec<Vec<Vec<StyledObj2>>>,
}

impl GridLayout {
    /// Creates a new GridLayout.
    pub fn new(settings: GridLayoutSettings) -> GridLayout {
        let (x_divisions, y_divisions) = settings.divisions;
        GridLayout {
            settings,
            objs: vec![
                // row
                vec![
                    // inner
                    vec![],
                ]
                .into_iter()
                .cycle()
                .take(y_divisions)
                .collect::<_>(),
            ]
            .into_iter()
            .cycle()
            .take(x_divisions)
            .collect::<_>(),
        }
    }

    /// Number of horizontal cubby divisions.
    pub fn num_cubbys_x(&self) -> usize {
        self.settings.divisions.0
    }
    /// Number of vertical cubby divisions.
    pub fn num_cubbys_y(&self) -> usize {
        self.settings.divisions.1
    }

    /// Get the bounds of a cubby at (i,j).
    pub fn get_cubby_bounds(&self, (i, j): (usize, usize)) -> Bounds {
        let (x_divisions, y_divisions) = self.settings.divisions;
        let (total_width, total_height) = self.settings.dims;
        let cubby_width: f64 = (total_width / (x_divisions as u64)) as f64;
        let cubby_height: f64 = (total_height / (y_divisions as u64)) as f64;
        let (object_margin_x, object_margin_y) = self.settings.object_margin;
        let (x_init, y_init) = self.settings.init;

        Bounds {
            top_bound: y_init as f64 + (j + 1) as f64 * cubby_height - object_margin_y as f64,
            bottom_bound: y_init as f64 + j as f64 * cubby_height + object_margin_y as f64,
            left_bound: x_init as f64 + i as f64 * cubby_width + object_margin_x as f64,
            right_bound: x_init as f64 + (i + 1) as f64 * cubby_width - object_margin_x as f64,
        }
    }

    /// Returns the center of the cubby.
    pub fn cubby_ctr(&self, (i, j): (usize, usize)) -> Pt {
        self.get_cubby_bounds((i, j)).bbox_center()
    }

    /// Returns a list of all inner objects.
    pub fn to_object2ds(&self) -> Vec<StyledObj2> {
        self.objs
            .clone()
            .into_iter()
            .flatten()
            .flatten()
            .collect::<Vec<_>>()
    }

    /// Given an Object2d, crops it to the cubby at objs[i][j] and inserts that
    /// into the grid.
    pub fn insert_and_crop_to_cubby(&mut self, (i, j): (usize, usize), d_o: StyledObj2) {
        let cropped = d_o.crop_to_bounds(self.get_cubby_bounds((i, j)));

        self.objs[i][j].extend(cropped);
    }

    /// Given an Object2d, recales it to the cubby at objs[i][j] and inserts that into the grid.
    pub fn insert_and_rescale_to_cubby(
        &mut self,
        (i, j): (usize, usize),
        d_o: StyledObj2,
        buffer: f64,
    ) {
        let mut modified_obj = d_o;
        {
            let frame_bounds = self.get_cubby_bounds((i, j));
            let inner_bounds = modified_obj.bounds();

            let w_scale = frame_bounds.width() / inner_bounds.width();
            let s_scale = frame_bounds.height() / inner_bounds.height();
            let scale = std::cmp::min(FloatOrd(w_scale), FloatOrd(s_scale)).0 * buffer;

            modified_obj *= scale;
        }

        {
            let frame_bounds = self.get_cubby_bounds((i, j));
            let inner_bounds = modified_obj.bounds();

            let translate_diff = frame_bounds.bbox_center() - inner_bounds.bbox_center();

            modified_obj += translate_diff;
        }

        self.objs[i][j].push(modified_obj);
    }
}
