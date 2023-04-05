//! Easily addressable web safe colors. (Not that this gets used with the web...
//! they're just well-named colors. Just normal colors.)
#![deny(missing_docs)]

use float_ord::FloatOrd;
use rand::prelude::SliceRandom;
use std::hash::{Hash, Hasher};

#[derive(Debug, Copy, Clone)]
/// A color, articulated in the [RGB color model](https://en.wikipedia.org/wiki/RGB_color_model).
pub struct ColorRGB {
    /// How much red (0.0 <= r <= 1.0).
    pub r: f64,
    /// How much green (0.0 <= g <= 1.0).
    pub g: f64,
    /// How much blue (0.0 <= b <= 1.0).
    pub b: f64,
}

#[allow(non_snake_case)]
const fn ColorRGB(r: f64, g: f64, b: f64) -> ColorRGB {
    ColorRGB { r, g, b }
}

macro_rules! color {
    ($name:ident, $r:expr, $g:expr, $b:expr) => {
        /// A color named $ident.
        pub const $name: ColorRGB = ColorRGB($r, $g, $b);
    };
}

impl PartialEq for ColorRGB {
    fn eq(&self, other: &Self) -> bool {
        FloatOrd(self.r).eq(&FloatOrd(other.r))
            && FloatOrd(self.g).eq(&FloatOrd(other.g))
            && FloatOrd(self.b).eq(&FloatOrd(other.b))
    }
}
impl Eq for ColorRGB {}

impl Hash for ColorRGB {
    fn hash<H: Hasher>(&self, state: &mut H) {
        FloatOrd(self.r).hash(state);
        FloatOrd(self.g).hash(state);
        FloatOrd(self.b).hash(state);
    }
}

impl PartialOrd for ColorRGB {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        FloatOrd(self.r * 100.0 + self.g * 10.0 + self.b * 1.0)
            .partial_cmp(&FloatOrd(other.r * 100.0 + other.g * 10.0 + other.b * 1.0))
    }
}

impl Ord for ColorRGB {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.partial_cmp(other).expect("color cmp")
    }
}

color!(ALICEBLUE, 0.941_176_470_588_235, 0.972_549_019_607_843, 1.0);
color!(
    ANTIQUEWHITE,
    0.980_392_156_862_745,
    0.921_568_627_450_98,
    0.843_137_254_901_961
);
color!(
    AQUAMARINE,
    0.498_039_215_686_275,
    1.0,
    0.831_372_549_019_608
);
color!(AZURE, 0.941_176_470_588_235, 1.0, 1.0);
color!(
    BEIGE,
    0.960_784_313_725_49,
    0.960_784_313_725_49,
    0.862_745_098_039_216
);
color!(BISQUE, 1.0, 0.894_117_647_058_824, 0.768_627_450_980_392);
color!(BLACK, 0.0, 0.0, 0.0);
color!(
    BLANCHEDALMOND,
    1.0,
    0.921_568_627_450_98,
    0.803_921_568_627_451
);
color!(BLUE, 0.0, 0.0, 1.0);
color!(
    BLUEVIOLET,
    0.541_176_470_588_235,
    0.168_627_450_980_392,
    0.886_274_509_803_922
);
color!(
    BROWN,
    0.647_058_823_529_412,
    0.164_705_882_352_941,
    0.164_705_882_352_941
);
color!(
    BURLYWOOD,
    0.870_588_235_294_118,
    0.721_568_627_450_98,
    0.529_411_764_705_882
);
color!(
    CADETBLUE,
    0.372_549_019_607_843,
    0.619_607_843_137_255,
    0.627_450_980_392_157
);
color!(CHARTREUSE, 0.498_039_215_686_275, 1.0, 0.0);
color!(
    CHOCOLATE,
    0.823_529_411_764_706,
    0.411_764_705_882_353,
    0.117_647_058_823_529
);
color!(CORAL, 1.0, 0.498_039_215_686_275, 0.313_725_490_196_078);
color!(
    CORNFLOWERBLUE,
    0.392_156_862_745_098,
    0.584_313_725_490_196,
    0.929_411_764_705_882
);
color!(CORNSILK, 1.0, 0.972_549_019_607_843, 0.862_745_098_039_216);
color!(CYAN, 0.0, 1.0, 1.0);
color!(DARKBLUE, 0.0, 0.0, 0.545_098_039_215_686);
color!(DARKCYAN, 0.0, 0.545_098_039_215_686, 0.545_098_039_215_686);
color!(
    DARKGOLDENROD,
    0.721_568_627_450_98,
    0.525_490_196_078_431,
    0.043_137_254_901_960_8
);
color!(
    DARKGRAY,
    0.662_745_098_039_216,
    0.662_745_098_039_216,
    0.662_745_098_039_216
);
color!(DARKGREEN, 0.0, 0.392_156_862_745_098, 0.0);
color!(
    DARKGREY,
    0.662_745_098_039_216,
    0.662_745_098_039_216,
    0.662_745_098_039_216
);
color!(
    DARKKHAKI,
    0.741_176_470_588_235,
    0.717_647_058_823_529,
    0.419_607_843_137_255
);
color!(
    DARKMAGENTA,
    0.545_098_039_215_686,
    0.0,
    0.545_098_039_215_686
);
color!(
    DARKOLIVEGREEN,
    0.333_333_333_333_333,
    0.419_607_843_137_255,
    0.184_313_725_490_196
);
color!(DARKORANGE, 1.0, 0.549_019_607_843_137, 0.0);
color!(DARKORCHID, 0.6, 0.196_078_431_372_549, 0.8);
color!(DARKRED, 0.545_098_039_215_686, 0.0, 0.0);
color!(
    DARKSALMON,
    0.913_725_490_196_078,
    0.588_235_294_117_647,
    0.478_431_372_549_02
);
color!(
    DARKSEAGREEN,
    0.560_784_313_725_49,
    0.737_254_901_960_784,
    0.560_784_313_725_49
);
color!(
    DARKSLATEBLUE,
    0.282_352_941_176_471,
    0.239_215_686_274_51,
    0.545_098_039_215_686
);
color!(
    DARKSLATEGRAY,
    0.184_313_725_490_196,
    0.309_803_921_568_627,
    0.309_803_921_568_627
);
color!(
    DARKSLATEGREY,
    0.184_313_725_490_196,
    0.309_803_921_568_627,
    0.309_803_921_568_627
);
color!(
    DARKTURQUOISE,
    0.0,
    0.807_843_137_254_902,
    0.819_607_843_137_255
);
color!(
    DARKVIOLET,
    0.580_392_156_862_745,
    0.0,
    0.827_450_980_392_157
);
color!(
    DEBIANRED,
    0.843_137_254_901_961,
    0.027_450_980_392_156_9,
    0.317_647_058_823_529
);
color!(
    DEEPPINK,
    1.0,
    0.078_431_372_549_019_6,
    0.576_470_588_235_294
);
color!(DEEPSKYBLUE, 0.0, 0.749_019_607_843_137, 1.0);
color!(
    DIMGRAY,
    0.411_764_705_882_353,
    0.411_764_705_882_353,
    0.411_764_705_882_353
);
color!(
    DIMGREY,
    0.411_764_705_882_353,
    0.411_764_705_882_353,
    0.411_764_705_882_353
);
color!(
    DODGERBLUE,
    0.117_647_058_823_529,
    0.564_705_882_352_941,
    1.0
);
color!(
    FIREBRICK,
    0.698_039_215_686_275,
    0.133_333_333_333_333,
    0.133_333_333_333_333
);
color!(
    FLORALWHITE,
    1.0,
    0.980_392_156_862_745,
    0.941_176_470_588_235
);
color!(
    FORESTGREEN,
    0.133_333_333_333_333,
    0.545_098_039_215_686,
    0.133_333_333_333_333
);
color!(
    GAINSBORO,
    0.862_745_098_039_216,
    0.862_745_098_039_216,
    0.862_745_098_039_216
);
color!(
    GHOSTWHITE,
    0.972_549_019_607_843,
    0.972_549_019_607_843,
    1.0
);
color!(GOLD, 1.0, 0.843_137_254_901_961, 0.0);
color!(
    GOLDENROD,
    0.854_901_960_784_314,
    0.647_058_823_529_412,
    0.125_490_196_078_431
);
color!(
    GRAY,
    0.745_098_039_215_686,
    0.745_098_039_215_686,
    0.745_098_039_215_686
);
color!(GREEN, 0.0, 1.0, 0.0);
color!(
    GREENYELLOW,
    0.678_431_372_549_02,
    1.0,
    0.184_313_725_490_196
);
color!(HONEYDEW, 0.941_176_470_588_235, 1.0, 0.941_176_470_588_235);
color!(HOTPINK, 1.0, 0.411_764_705_882_353, 0.705_882_352_941_177);
color!(
    INDIANRED,
    0.803_921_568_627_451,
    0.360_784_313_725_49,
    0.360_784_313_725_49
);
color!(IVORY, 1.0, 1.0, 0.941_176_470_588_235);
color!(
    KHAKI,
    0.941_176_470_588_235,
    0.901_960_784_313_726,
    0.549_019_607_843_137
);
color!(
    LAVENDER,
    0.901_960_784_313_726,
    0.901_960_784_313_726,
    0.980_392_156_862_745
);
color!(
    LAVENDERBLUSH,
    1.0,
    0.941_176_470_588_235,
    0.960_784_313_725_49
);
color!(LAWNGREEN, 0.486_274_509_803_922, 0.988_235_294_117_647, 0.0);
color!(
    LEMONCHIFFON,
    1.0,
    0.980_392_156_862_745,
    0.803_921_568_627_451
);
color!(
    LIGHTBLUE,
    0.678_431_372_549_02,
    0.847_058_823_529_412,
    0.901_960_784_313_726
);
color!(
    LIGHTCORAL,
    0.941_176_470_588_235,
    0.501_960_784_313_726,
    0.501_960_784_313_726
);
color!(LIGHTCYAN, 0.878_431_372_549_02, 1.0, 1.0);
color!(
    LIGHTGOLDENROD,
    0.933_333_333_333_333,
    0.866_666_666_666_667,
    0.509_803_921_568_627
);
color!(
    LIGHTGRAY,
    0.827_450_980_392_157,
    0.827_450_980_392_157,
    0.827_450_980_392_157
);
color!(
    LIGHTGREEN,
    0.564_705_882_352_941,
    0.933_333_333_333_333,
    0.564_705_882_352_941
);
color!(
    LIGHTGREY,
    0.827_450_980_392_157,
    0.827_450_980_392_157,
    0.827_450_980_392_157
);
color!(LIGHTPINK, 1.0, 0.713_725_490_196_079, 0.756_862_745_098_039);
color!(
    LIGHTSALMON,
    1.0,
    0.627_450_980_392_157,
    0.478_431_372_549_02
);
color!(
    LIGHTSEAGREEN,
    0.125_490_196_078_431,
    0.698_039_215_686_275,
    0.666_666_666_666_667
);
color!(
    LIGHTSKYBLUE,
    0.529_411_764_705_882,
    0.807_843_137_254_902,
    0.980_392_156_862_745
);
color!(
    LIGHTSLATEBLUE,
    0.517_647_058_823_529,
    0.439_215_686_274_51,
    1.0
);
color!(
    LIGHTSLATEGRAY,
    0.466_666_666_666_667,
    0.533_333_333_333_333,
    0.6
);
color!(
    LIGHTSLATEGREY,
    0.466_666_666_666_667,
    0.533_333_333_333_333,
    0.6
);
color!(
    LIGHTSTEELBLUE,
    0.690_196_078_431_373,
    0.768_627_450_980_392,
    0.870_588_235_294_118
);
color!(LIGHTYELLOW, 1.0, 1.0, 0.878_431_372_549_02);
color!(
    LIMEGREEN,
    0.196_078_431_372_549,
    0.803_921_568_627_451,
    0.196_078_431_372_549
);
color!(
    LINEN,
    0.980_392_156_862_745,
    0.941_176_470_588_235,
    0.901_960_784_313_726
);
color!(MAGENTA, 1.0, 0.0, 1.0);
color!(
    MAROON,
    0.690_196_078_431_373,
    0.188_235_294_117_647,
    0.376_470_588_235_294
);
color!(
    MEDIUMAQUAMARINE,
    0.4,
    0.803_921_568_627_451,
    0.666_666_666_666_667
);
color!(MEDIUMBLUE, 0.0, 0.0, 0.803_921_568_627_451);
color!(
    MEDIUMORCHID,
    0.729_411_764_705_882,
    0.333_333_333_333_333,
    0.827_450_980_392_157
);
color!(
    MEDIUMPURPLE,
    0.576_470_588_235_294,
    0.439_215_686_274_51,
    0.858_823_529_411_765
);
color!(
    MEDIUMSEAGREEN,
    0.235_294_117_647_059,
    0.701_960_784_313_726,
    0.443_137_254_901_961
);
color!(
    MEDIUMSLATEBLUE,
    0.482_352_941_176_471,
    0.407_843_137_254_902,
    0.933_333_333_333_333
);
color!(
    MEDIUMSPRINGGREEN,
    0.0,
    0.980_392_156_862_745,
    0.603_921_568_627_451
);
color!(
    MEDIUMTURQUOISE,
    0.282_352_941_176_471,
    0.819_607_843_137_255,
    0.8
);
color!(
    MEDIUMVIOLETRED,
    0.780_392_156_862_745,
    0.082_352_941_176_470_6,
    0.521_568_627_450_98
);
color!(
    MIDNIGHTBLUE,
    0.098_039_215_686_274_5,
    0.098_039_215_686_274_5,
    0.439_215_686_274_51
);
color!(MINTCREAM, 0.960_784_313_725_49, 1.0, 0.980_392_156_862_745);
color!(MISTYROSE, 1.0, 0.894_117_647_058_824, 0.882_352_941_176_471);
color!(MOCCASIN, 1.0, 0.894_117_647_058_824, 0.709_803_921_568_628);
color!(
    NAVAJOWHITE,
    1.0,
    0.870_588_235_294_118,
    0.678_431_372_549_02
);
color!(NAVY, 0.0, 0.0, 0.501_960_784_313_726);
color!(NAVYBLUE, 0.0, 0.0, 0.501_960_784_313_726);
color!(
    OLDLACE,
    0.992_156_862_745_098,
    0.960_784_313_725_49,
    0.901_960_784_313_726
);
color!(
    OLIVEDRAB,
    0.419_607_843_137_255,
    0.556_862_745_098_039,
    0.137_254_901_960_784
);
color!(ORANGE, 1.0, 0.647_058_823_529_412, 0.0);
color!(ORANGERED, 1.0, 0.270_588_235_294_118, 0.0);
color!(
    ORCHID,
    0.854_901_960_784_314,
    0.439_215_686_274_51,
    0.839_215_686_274_51
);
color!(
    PALEGOLDENROD,
    0.933_333_333_333_333,
    0.909_803_921_568_628,
    0.666_666_666_666_667
);
color!(
    PALEGREEN,
    0.596_078_431_372_549,
    0.984_313_725_490_196,
    0.596_078_431_372_549
);
color!(
    PALETURQUOISE,
    0.686_274_509_803_922,
    0.933_333_333_333_333,
    0.933_333_333_333_333
);
color!(
    PALEVIOLETRED,
    0.858_823_529_411_765,
    0.439_215_686_274_51,
    0.576_470_588_235_294
);
color!(
    PAPAYAWHIP,
    1.0,
    0.937_254_901_960_784,
    0.835_294_117_647_059
);
color!(PEACHPUFF, 1.0, 0.854_901_960_784_314, 0.725_490_196_078_431);
color!(
    PERU,
    0.803_921_568_627_451,
    0.521_568_627_450_98,
    0.247_058_823_529_412
);
color!(PINK, 1.0, 0.752_941_176_470_588, 0.796_078_431_372_549);
color!(
    PLUM,
    0.866_666_666_666_667,
    0.627_450_980_392_157,
    0.866_666_666_666_667
);
color!(
    POWDERBLUE,
    0.690_196_078_431_373,
    0.878_431_372_549_02,
    0.901_960_784_313_726
);
color!(RED, 1.0, 0.0, 0.0);
color!(
    ROSYBROWN,
    0.737_254_901_960_784,
    0.560_784_313_725_49,
    0.560_784_313_725_49
);
color!(
    ROYALBLUE,
    0.254_901_960_784_314,
    0.411_764_705_882_353,
    0.882_352_941_176_471
);
color!(
    SADDLEBROWN,
    0.545_098_039_215_686,
    0.270_588_235_294_118,
    0.074_509_803_921_568_6
);
color!(
    SALMON,
    0.980_392_156_862_745,
    0.501_960_784_313_726,
    0.447_058_823_529_412
);
color!(
    SANDYBROWN,
    0.956_862_745_098_039,
    0.643_137_254_901_961,
    0.376_470_588_235_294
);
color!(
    SEAGREEN,
    0.180_392_156_862_745,
    0.545_098_039_215_686,
    0.341_176_470_588_235
);
color!(SEASHELL, 1.0, 0.960_784_313_725_49, 0.933_333_333_333_333);
color!(
    SIENNA,
    0.627_450_980_392_157,
    0.321_568_627_450_98,
    0.176_470_588_235_294
);
color!(
    SKYBLUE,
    0.529_411_764_705_882,
    0.807_843_137_254_902,
    0.921_568_627_450_98
);
color!(
    SLATEBLUE,
    0.415_686_274_509_804,
    0.352_941_176_470_588,
    0.803_921_568_627_451
);
color!(
    SLATEGRAY,
    0.439_215_686_274_51,
    0.501_960_784_313_726,
    0.564_705_882_352_941
);
color!(
    SLATEGREY,
    0.439_215_686_274_51,
    0.501_960_784_313_726,
    0.564_705_882_352_941
);
color!(SNOW, 1.0, 0.980_392_156_862_745, 0.980_392_156_862_745);
color!(SPRINGGREEN, 0.0, 1.0, 0.498_039_215_686_275);
color!(
    STEELBLUE,
    0.274_509_803_921_569,
    0.509_803_921_568_627,
    0.705_882_352_941_177
);
color!(
    TAN,
    0.823_529_411_764_706,
    0.705_882_352_941_177,
    0.549_019_607_843_137
);
color!(
    THISTLE,
    0.847_058_823_529_412,
    0.749_019_607_843_137,
    0.847_058_823_529_412
);
color!(TOMATO, 1.0, 0.388_235_294_117_647, 0.278_431_372_549_02);
color!(
    TURQUOISE,
    0.250_980_392_156_863,
    0.878_431_372_549_02,
    0.815_686_274_509_804
);
color!(
    VIOLET,
    0.933_333_333_333_333,
    0.509_803_921_568_627,
    0.933_333_333_333_333
);
color!(
    VIOLETRED,
    0.815_686_274_509_804,
    0.125_490_196_078_431,
    0.564_705_882_352_941
);
color!(
    WHEAT,
    0.960_784_313_725_49,
    0.870_588_235_294_118,
    0.701_960_784_313_726
);
color!(WHITE, 1.0, 1.0, 1.0);
color!(
    WHITESMOKE,
    0.960_784_313_725_49,
    0.960_784_313_725_49,
    0.960_784_313_725_49
);
color!(YELLOW, 1.0, 1.0, 0.0);
color!(
    YELLOWGREEN,
    0.603_921_568_627_451,
    0.803_921_568_627_451,
    0.196_078_431_372_549
);


pub mod subway {
    //! Colors used only in plotting the NYC subway.
    use super::*;

    color!(BLUE_ACE, 0.0, 0.22, 0.6);
    color!(ORANGE_BDFM, 1.0, 0.39, 0.1);
    color!(LIME_G, 0.42, 0.75, 0.27);
    color!(GREY_L, 0.65, 0.66, 0.67);
    color!(BROWN_JZ, 0.6, 0.4, 0.2);
    color!(YELLOW_NQRW, 0.99, 0.8, 0.04);
    color!(RED_123, 0.93, 0.21, 0.18);
    color!(GREEN_456, 0.0, 0.58, 0.24);
    color!(PURPLE_7, 0.73, 0.20, 0.68);
    color!(TEAL_T, 0.0, 0.68, 0.82);
    color!(GRAY_S, 0.5, 0.51, 0.51);
}

/// All known colors.
pub static COLORS: [&ColorRGB; 141] = [
    &ALICEBLUE,
    &ANTIQUEWHITE,
    &AQUAMARINE,
    &AZURE,
    &BEIGE,
    &BISQUE,
    &BLACK,
    &BLANCHEDALMOND,
    &BLUE,
    &BLUEVIOLET,
    &BROWN,
    &BURLYWOOD,
    &CADETBLUE,
    &CHARTREUSE,
    &CHOCOLATE,
    &CORAL,
    &CORNFLOWERBLUE,
    &CORNSILK,
    &CYAN,
    &DARKBLUE,
    &DARKCYAN,
    &DARKGOLDENROD,
    &DARKGRAY,
    &DARKGREEN,
    &DARKGREY,
    &DARKKHAKI,
    &DARKMAGENTA,
    &DARKOLIVEGREEN,
    &DARKORANGE,
    &DARKORCHID,
    &DARKRED,
    &DARKSALMON,
    &DARKSEAGREEN,
    &DARKSLATEBLUE,
    &DARKSLATEGRAY,
    &DARKSLATEGREY,
    &DARKTURQUOISE,
    &DARKVIOLET,
    &DEBIANRED,
    &DEEPPINK,
    &DEEPSKYBLUE,
    &DIMGRAY,
    &DIMGREY,
    &DODGERBLUE,
    &FIREBRICK,
    &FLORALWHITE,
    &FORESTGREEN,
    &GAINSBORO,
    &GHOSTWHITE,
    &GOLD,
    &GOLDENROD,
    &GRAY,
    &GREEN,
    &GREENYELLOW,
    &HONEYDEW,
    &HOTPINK,
    &INDIANRED,
    &IVORY,
    &KHAKI,
    &LAVENDER,
    &LAVENDERBLUSH,
    &LAWNGREEN,
    &LEMONCHIFFON,
    &LIGHTBLUE,
    &LIGHTCORAL,
    &LIGHTCYAN,
    &LIGHTGOLDENROD,
    &LIGHTGRAY,
    &LIGHTGREEN,
    &LIGHTGREY,
    &LIGHTPINK,
    &LIGHTSALMON,
    &LIGHTSEAGREEN,
    &LIGHTSKYBLUE,
    &LIGHTSLATEBLUE,
    &LIGHTSLATEGRAY,
    &LIGHTSLATEGREY,
    &LIGHTSTEELBLUE,
    &LIGHTYELLOW,
    &LIMEGREEN,
    &LINEN,
    &MAGENTA,
    &MAROON,
    &MEDIUMAQUAMARINE,
    &MEDIUMBLUE,
    &MEDIUMORCHID,
    &MEDIUMPURPLE,
    &MEDIUMSEAGREEN,
    &MEDIUMSLATEBLUE,
    &MEDIUMSPRINGGREEN,
    &MEDIUMTURQUOISE,
    &MEDIUMVIOLETRED,
    &MIDNIGHTBLUE,
    &MINTCREAM,
    &MISTYROSE,
    &MOCCASIN,
    &NAVAJOWHITE,
    &NAVY,
    &NAVYBLUE,
    &OLDLACE,
    &OLIVEDRAB,
    &ORANGE,
    &ORANGERED,
    &ORCHID,
    &PALEGOLDENROD,
    &PALEGREEN,
    &PALETURQUOISE,
    &PALEVIOLETRED,
    &PAPAYAWHIP,
    &PEACHPUFF,
    &PERU,
    &PINK,
    &PLUM,
    &POWDERBLUE,
    &RED,
    &ROSYBROWN,
    &ROYALBLUE,
    &SADDLEBROWN,
    &SALMON,
    &SANDYBROWN,
    &SEAGREEN,
    &SEASHELL,
    &SIENNA,
    &SKYBLUE,
    &SLATEBLUE,
    &SLATEGRAY,
    &SLATEGREY,
    &SNOW,
    &SPRINGGREEN,
    &STEELBLUE,
    &TAN,
    &THISTLE,
    &TOMATO,
    &TURQUOISE,
    &VIOLET,
    &VIOLETRED,
    &WHEAT,
    &WHITE,
    &WHITESMOKE,
    &YELLOW,
    &YELLOWGREEN,
];

/// Returns a vector of |limit| random colors.
pub fn take_random_colors(limit: usize) -> Vec<&'static ColorRGB> {
    let mut colors = COLORS;

    let mut rng = rand::thread_rng();
    colors.shuffle(&mut rng);

    colors.into_iter().take(limit).collect()
}
