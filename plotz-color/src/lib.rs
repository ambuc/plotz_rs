use float_ord::FloatOrd;
use std::hash::{Hash, Hasher};

#[derive(Debug, Copy, Clone)]
pub struct ColorRGB {
    pub r: f64,
    pub g: f64,
    pub b: f64,
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

pub const ALICEBLUE: ColorRGB = ColorRGB {
    r: 0.941_176_470_588_235,
    g: 0.972_549_019_607_843,
    b: 1.0,
};
pub const ANTIQUEWHITE: ColorRGB = ColorRGB {
    r: 0.980_392_156_862_745,
    g: 0.921_568_627_450_98,
    b: 0.843_137_254_901_961,
};
pub const AQUAMARINE: ColorRGB = ColorRGB {
    r: 0.498_039_215_686_275,
    g: 1.0,
    b: 0.831_372_549_019_608,
};
pub const AZURE: ColorRGB = ColorRGB {
    r: 0.941_176_470_588_235,
    g: 1.0,
    b: 1.0,
};
pub const BEIGE: ColorRGB = ColorRGB {
    r: 0.960_784_313_725_49,
    g: 0.960_784_313_725_49,
    b: 0.862_745_098_039_216,
};
pub const BISQUE: ColorRGB = ColorRGB {
    r: 1.0,
    g: 0.894_117_647_058_824,
    b: 0.768_627_450_980_392,
};
pub const BLACK: ColorRGB = ColorRGB {
    r: 0.0,
    g: 0.0,
    b: 0.0,
};
pub const BLANCHEDALMOND: ColorRGB = ColorRGB {
    r: 1.0,
    g: 0.921_568_627_450_98,
    b: 0.803_921_568_627_451,
};
pub const BLUE: ColorRGB = ColorRGB {
    r: 0.0,
    g: 0.0,
    b: 1.0,
};
pub const BLUEVIOLET: ColorRGB = ColorRGB {
    r: 0.541_176_470_588_235,
    g: 0.168_627_450_980_392,
    b: 0.886_274_509_803_922,
};
pub const BROWN: ColorRGB = ColorRGB {
    r: 0.647_058_823_529_412,
    g: 0.164_705_882_352_941,
    b: 0.164_705_882_352_941,
};
pub const BURLYWOOD: ColorRGB = ColorRGB {
    r: 0.870_588_235_294_118,
    g: 0.721_568_627_450_98,
    b: 0.529_411_764_705_882,
};
pub const CADETBLUE: ColorRGB = ColorRGB {
    r: 0.372_549_019_607_843,
    g: 0.619_607_843_137_255,
    b: 0.627_450_980_392_157,
};
pub const CHARTREUSE: ColorRGB = ColorRGB {
    r: 0.498_039_215_686_275,
    g: 1.0,
    b: 0.0,
};
pub const CHOCOLATE: ColorRGB = ColorRGB {
    r: 0.823_529_411_764_706,
    g: 0.411_764_705_882_353,
    b: 0.117_647_058_823_529,
};
pub const CORAL: ColorRGB = ColorRGB {
    r: 1.0,
    g: 0.498_039_215_686_275,
    b: 0.313_725_490_196_078,
};
pub const CORNFLOWERBLUE: ColorRGB = ColorRGB {
    r: 0.392_156_862_745_098,
    g: 0.584_313_725_490_196,
    b: 0.929_411_764_705_882,
};
pub const CORNSILK: ColorRGB = ColorRGB {
    r: 1.0,
    g: 0.972_549_019_607_843,
    b: 0.862_745_098_039_216,
};
pub const CYAN: ColorRGB = ColorRGB {
    r: 0.0,
    g: 1.0,
    b: 1.0,
};
pub const DARKBLUE: ColorRGB = ColorRGB {
    r: 0.0,
    g: 0.0,
    b: 0.545_098_039_215_686,
};
pub const DARKCYAN: ColorRGB = ColorRGB {
    r: 0.0,
    g: 0.545_098_039_215_686,
    b: 0.545_098_039_215_686,
};
pub const DARKGOLDENROD: ColorRGB = ColorRGB {
    r: 0.721_568_627_450_98,
    g: 0.525_490_196_078_431,
    b: 0.043_137_254_901_960_8,
};
pub const DARKGRAY: ColorRGB = ColorRGB {
    r: 0.662_745_098_039_216,
    g: 0.662_745_098_039_216,
    b: 0.662_745_098_039_216,
};
pub const DARKGREEN: ColorRGB = ColorRGB {
    r: 0.0,
    g: 0.392_156_862_745_098,
    b: 0.0,
};
pub const DARKGREY: ColorRGB = ColorRGB {
    r: 0.662_745_098_039_216,
    g: 0.662_745_098_039_216,
    b: 0.662_745_098_039_216,
};
pub const DARKKHAKI: ColorRGB = ColorRGB {
    r: 0.741_176_470_588_235,
    g: 0.717_647_058_823_529,
    b: 0.419_607_843_137_255,
};
pub const DARKMAGENTA: ColorRGB = ColorRGB {
    r: 0.545_098_039_215_686,
    g: 0.0,
    b: 0.545_098_039_215_686,
};
pub const DARKOLIVEGREEN: ColorRGB = ColorRGB {
    r: 0.333_333_333_333_333,
    g: 0.419_607_843_137_255,
    b: 0.184_313_725_490_196,
};
pub const DARKORANGE: ColorRGB = ColorRGB {
    r: 1.0,
    g: 0.549_019_607_843_137,
    b: 0.0,
};
pub const DARKORCHID: ColorRGB = ColorRGB {
    r: 0.6,
    g: 0.196_078_431_372_549,
    b: 0.8,
};
pub const DARKRED: ColorRGB = ColorRGB {
    r: 0.545_098_039_215_686,
    g: 0.0,
    b: 0.0,
};
pub const DARKSALMON: ColorRGB = ColorRGB {
    r: 0.913_725_490_196_078,
    g: 0.588_235_294_117_647,
    b: 0.478_431_372_549_02,
};
pub const DARKSEAGREEN: ColorRGB = ColorRGB {
    r: 0.560_784_313_725_49,
    g: 0.737_254_901_960_784,
    b: 0.560_784_313_725_49,
};
pub const DARKSLATEBLUE: ColorRGB = ColorRGB {
    r: 0.282_352_941_176_471,
    g: 0.239_215_686_274_51,
    b: 0.545_098_039_215_686,
};
pub const DARKSLATEGRAY: ColorRGB = ColorRGB {
    r: 0.184_313_725_490_196,
    g: 0.309_803_921_568_627,
    b: 0.309_803_921_568_627,
};
pub const DARKSLATEGREY: ColorRGB = ColorRGB {
    r: 0.184_313_725_490_196,
    g: 0.309_803_921_568_627,
    b: 0.309_803_921_568_627,
};
pub const DARKTURQUOISE: ColorRGB = ColorRGB {
    r: 0.0,
    g: 0.807_843_137_254_902,
    b: 0.819_607_843_137_255,
};
pub const DARKVIOLET: ColorRGB = ColorRGB {
    r: 0.580_392_156_862_745,
    g: 0.0,
    b: 0.827_450_980_392_157,
};
pub const DEBIANRED: ColorRGB = ColorRGB {
    r: 0.843_137_254_901_961,
    g: 0.027_450_980_392_156_9,
    b: 0.317_647_058_823_529,
};
pub const DEEPPINK: ColorRGB = ColorRGB {
    r: 1.0,
    g: 0.078_431_372_549_019_6,
    b: 0.576_470_588_235_294,
};
pub const DEEPSKYBLUE: ColorRGB = ColorRGB {
    r: 0.0,
    g: 0.749_019_607_843_137,
    b: 1.0,
};
pub const DIMGRAY: ColorRGB = ColorRGB {
    r: 0.411_764_705_882_353,
    g: 0.411_764_705_882_353,
    b: 0.411_764_705_882_353,
};
pub const DIMGREY: ColorRGB = ColorRGB {
    r: 0.411_764_705_882_353,
    g: 0.411_764_705_882_353,
    b: 0.411_764_705_882_353,
};
pub const DODGERBLUE: ColorRGB = ColorRGB {
    r: 0.117_647_058_823_529,
    g: 0.564_705_882_352_941,
    b: 1.0,
};
pub const FIREBRICK: ColorRGB = ColorRGB {
    r: 0.698_039_215_686_275,
    g: 0.133_333_333_333_333,
    b: 0.133_333_333_333_333,
};
pub const FLORALWHITE: ColorRGB = ColorRGB {
    r: 1.0,
    g: 0.980_392_156_862_745,
    b: 0.941_176_470_588_235,
};
pub const FORESTGREEN: ColorRGB = ColorRGB {
    r: 0.133_333_333_333_333,
    g: 0.545_098_039_215_686,
    b: 0.133_333_333_333_333,
};
pub const GAINSBORO: ColorRGB = ColorRGB {
    r: 0.862_745_098_039_216,
    g: 0.862_745_098_039_216,
    b: 0.862_745_098_039_216,
};
pub const GHOSTWHITE: ColorRGB = ColorRGB {
    r: 0.972_549_019_607_843,
    g: 0.972_549_019_607_843,
    b: 1.0,
};
pub const GOLD: ColorRGB = ColorRGB {
    r: 1.0,
    g: 0.843_137_254_901_961,
    b: 0.0,
};
pub const GOLDENROD: ColorRGB = ColorRGB {
    r: 0.854_901_960_784_314,
    g: 0.647_058_823_529_412,
    b: 0.125_490_196_078_431,
};
pub const GRAY: ColorRGB = ColorRGB {
    r: 0.745_098_039_215_686,
    g: 0.745_098_039_215_686,
    b: 0.745_098_039_215_686,
};
pub const GREEN: ColorRGB = ColorRGB {
    r: 0.0,
    g: 1.0,
    b: 0.0,
};
pub const GREENYELLOW: ColorRGB = ColorRGB {
    r: 0.678_431_372_549_02,
    g: 1.0,
    b: 0.184_313_725_490_196,
};
pub const HONEYDEW: ColorRGB = ColorRGB {
    r: 0.941_176_470_588_235,
    g: 1.0,
    b: 0.941_176_470_588_235,
};
pub const HOTPINK: ColorRGB = ColorRGB {
    r: 1.0,
    g: 0.411_764_705_882_353,
    b: 0.705_882_352_941_177,
};
pub const INDIANRED: ColorRGB = ColorRGB {
    r: 0.803_921_568_627_451,
    g: 0.360_784_313_725_49,
    b: 0.360_784_313_725_49,
};
pub const IVORY: ColorRGB = ColorRGB {
    r: 1.0,
    g: 1.0,
    b: 0.941_176_470_588_235,
};
pub const KHAKI: ColorRGB = ColorRGB {
    r: 0.941_176_470_588_235,
    g: 0.901_960_784_313_726,
    b: 0.549_019_607_843_137,
};
pub const LAVENDER: ColorRGB = ColorRGB {
    r: 0.901_960_784_313_726,
    g: 0.901_960_784_313_726,
    b: 0.980_392_156_862_745,
};
pub const LAVENDERBLUSH: ColorRGB = ColorRGB {
    r: 1.0,
    g: 0.941_176_470_588_235,
    b: 0.960_784_313_725_49,
};
pub const LAWNGREEN: ColorRGB = ColorRGB {
    r: 0.486_274_509_803_922,
    g: 0.988_235_294_117_647,
    b: 0.0,
};
pub const LEMONCHIFFON: ColorRGB = ColorRGB {
    r: 1.0,
    g: 0.980_392_156_862_745,
    b: 0.803_921_568_627_451,
};
pub const LIGHTBLUE: ColorRGB = ColorRGB {
    r: 0.678_431_372_549_02,
    g: 0.847_058_823_529_412,
    b: 0.901_960_784_313_726,
};
pub const LIGHTCORAL: ColorRGB = ColorRGB {
    r: 0.941_176_470_588_235,
    g: 0.501_960_784_313_726,
    b: 0.501_960_784_313_726,
};
pub const LIGHTCYAN: ColorRGB = ColorRGB {
    r: 0.878_431_372_549_02,
    g: 1.0,
    b: 1.0,
};
pub const LIGHTGOLDENROD: ColorRGB = ColorRGB {
    r: 0.933_333_333_333_333,
    g: 0.866_666_666_666_667,
    b: 0.509_803_921_568_627,
};
pub const LIGHTGRAY: ColorRGB = ColorRGB {
    r: 0.827_450_980_392_157,
    g: 0.827_450_980_392_157,
    b: 0.827_450_980_392_157,
};
pub const LIGHTGREEN: ColorRGB = ColorRGB {
    r: 0.564_705_882_352_941,
    g: 0.933_333_333_333_333,
    b: 0.564_705_882_352_941,
};
pub const LIGHTGREY: ColorRGB = ColorRGB {
    r: 0.827_450_980_392_157,
    g: 0.827_450_980_392_157,
    b: 0.827_450_980_392_157,
};
pub const LIGHTPINK: ColorRGB = ColorRGB {
    r: 1.0,
    g: 0.713_725_490_196_079,
    b: 0.756_862_745_098_039,
};
pub const LIGHTSALMON: ColorRGB = ColorRGB {
    r: 1.0,
    g: 0.627_450_980_392_157,
    b: 0.478_431_372_549_02,
};
pub const LIGHTSEAGREEN: ColorRGB = ColorRGB {
    r: 0.125_490_196_078_431,
    g: 0.698_039_215_686_275,
    b: 0.666_666_666_666_667,
};
pub const LIGHTSKYBLUE: ColorRGB = ColorRGB {
    r: 0.529_411_764_705_882,
    g: 0.807_843_137_254_902,
    b: 0.980_392_156_862_745,
};
pub const LIGHTSLATEBLUE: ColorRGB = ColorRGB {
    r: 0.517_647_058_823_529,
    g: 0.439_215_686_274_51,
    b: 1.0,
};
pub const LIGHTSLATEGRAY: ColorRGB = ColorRGB {
    r: 0.466_666_666_666_667,
    g: 0.533_333_333_333_333,
    b: 0.6,
};
pub const LIGHTSLATEGREY: ColorRGB = ColorRGB {
    r: 0.466_666_666_666_667,
    g: 0.533_333_333_333_333,
    b: 0.6,
};
pub const LIGHTSTEELBLUE: ColorRGB = ColorRGB {
    r: 0.690_196_078_431_373,
    g: 0.768_627_450_980_392,
    b: 0.870_588_235_294_118,
};
pub const LIGHTYELLOW: ColorRGB = ColorRGB {
    r: 1.0,
    g: 1.0,
    b: 0.878_431_372_549_02,
};
pub const LIMEGREEN: ColorRGB = ColorRGB {
    r: 0.196_078_431_372_549,
    g: 0.803_921_568_627_451,
    b: 0.196_078_431_372_549,
};
pub const LINEN: ColorRGB = ColorRGB {
    r: 0.980_392_156_862_745,
    g: 0.941_176_470_588_235,
    b: 0.901_960_784_313_726,
};
pub const MAGENTA: ColorRGB = ColorRGB {
    r: 1.0,
    g: 0.0,
    b: 1.0,
};
pub const MAROON: ColorRGB = ColorRGB {
    r: 0.690_196_078_431_373,
    g: 0.188_235_294_117_647,
    b: 0.376_470_588_235_294,
};
pub const MEDIUMAQUAMARINE: ColorRGB = ColorRGB {
    r: 0.4,
    g: 0.803_921_568_627_451,
    b: 0.666_666_666_666_667,
};
pub const MEDIUMBLUE: ColorRGB = ColorRGB {
    r: 0.0,
    g: 0.0,
    b: 0.803_921_568_627_451,
};
pub const MEDIUMORCHID: ColorRGB = ColorRGB {
    r: 0.729_411_764_705_882,
    g: 0.333_333_333_333_333,
    b: 0.827_450_980_392_157,
};
pub const MEDIUMPURPLE: ColorRGB = ColorRGB {
    r: 0.576_470_588_235_294,
    g: 0.439_215_686_274_51,
    b: 0.858_823_529_411_765,
};
pub const MEDIUMSEAGREEN: ColorRGB = ColorRGB {
    r: 0.235_294_117_647_059,
    g: 0.701_960_784_313_726,
    b: 0.443_137_254_901_961,
};
pub const MEDIUMSLATEBLUE: ColorRGB = ColorRGB {
    r: 0.482_352_941_176_471,
    g: 0.407_843_137_254_902,
    b: 0.933_333_333_333_333,
};
pub const MEDIUMSPRINGGREEN: ColorRGB = ColorRGB {
    r: 0.0,
    g: 0.980_392_156_862_745,
    b: 0.603_921_568_627_451,
};
pub const MEDIUMTURQUOISE: ColorRGB = ColorRGB {
    r: 0.282_352_941_176_471,
    g: 0.819_607_843_137_255,
    b: 0.8,
};
pub const MEDIUMVIOLETRED: ColorRGB = ColorRGB {
    r: 0.780_392_156_862_745,
    g: 0.082_352_941_176_470_6,
    b: 0.521_568_627_450_98,
};
pub const MIDNIGHTBLUE: ColorRGB = ColorRGB {
    r: 0.098_039_215_686_274_5,
    g: 0.098_039_215_686_274_5,
    b: 0.439_215_686_274_51,
};
pub const MINTCREAM: ColorRGB = ColorRGB {
    r: 0.960_784_313_725_49,
    g: 1.0,
    b: 0.980_392_156_862_745,
};
pub const MISTYROSE: ColorRGB = ColorRGB {
    r: 1.0,
    g: 0.894_117_647_058_824,
    b: 0.882_352_941_176_471,
};
pub const MOCCASIN: ColorRGB = ColorRGB {
    r: 1.0,
    g: 0.894_117_647_058_824,
    b: 0.709_803_921_568_628,
};
pub const NAVAJOWHITE: ColorRGB = ColorRGB {
    r: 1.0,
    g: 0.870_588_235_294_118,
    b: 0.678_431_372_549_02,
};
pub const NAVY: ColorRGB = ColorRGB {
    r: 0.0,
    g: 0.0,
    b: 0.501_960_784_313_726,
};
pub const NAVYBLUE: ColorRGB = ColorRGB {
    r: 0.0,
    g: 0.0,
    b: 0.501_960_784_313_726,
};
pub const OLDLACE: ColorRGB = ColorRGB {
    r: 0.992_156_862_745_098,
    g: 0.960_784_313_725_49,
    b: 0.901_960_784_313_726,
};
pub const OLIVEDRAB: ColorRGB = ColorRGB {
    r: 0.419_607_843_137_255,
    g: 0.556_862_745_098_039,
    b: 0.137_254_901_960_784,
};
pub const ORANGE: ColorRGB = ColorRGB {
    r: 1.0,
    g: 0.647_058_823_529_412,
    b: 0.0,
};
pub const ORANGERED: ColorRGB = ColorRGB {
    r: 1.0,
    g: 0.270_588_235_294_118,
    b: 0.0,
};
pub const ORCHID: ColorRGB = ColorRGB {
    r: 0.854_901_960_784_314,
    g: 0.439_215_686_274_51,
    b: 0.839_215_686_274_51,
};
pub const PALEGOLDENROD: ColorRGB = ColorRGB {
    r: 0.933_333_333_333_333,
    g: 0.909_803_921_568_628,
    b: 0.666_666_666_666_667,
};
pub const PALEGREEN: ColorRGB = ColorRGB {
    r: 0.596_078_431_372_549,
    g: 0.984_313_725_490_196,
    b: 0.596_078_431_372_549,
};
pub const PALETURQUOISE: ColorRGB = ColorRGB {
    r: 0.686_274_509_803_922,
    g: 0.933_333_333_333_333,
    b: 0.933_333_333_333_333,
};
pub const PALEVIOLETRED: ColorRGB = ColorRGB {
    r: 0.858_823_529_411_765,
    g: 0.439_215_686_274_51,
    b: 0.576_470_588_235_294,
};
pub const PAPAYAWHIP: ColorRGB = ColorRGB {
    r: 1.0,
    g: 0.937_254_901_960_784,
    b: 0.835_294_117_647_059,
};
pub const PEACHPUFF: ColorRGB = ColorRGB {
    r: 1.0,
    g: 0.854_901_960_784_314,
    b: 0.725_490_196_078_431,
};
pub const PERU: ColorRGB = ColorRGB {
    r: 0.803_921_568_627_451,
    g: 0.521_568_627_450_98,
    b: 0.247_058_823_529_412,
};
pub const PINK: ColorRGB = ColorRGB {
    r: 1.0,
    g: 0.752_941_176_470_588,
    b: 0.796_078_431_372_549,
};
pub const PLUM: ColorRGB = ColorRGB {
    r: 0.866_666_666_666_667,
    g: 0.627_450_980_392_157,
    b: 0.866_666_666_666_667,
};
pub const POWDERBLUE: ColorRGB = ColorRGB {
    r: 0.690_196_078_431_373,
    g: 0.878_431_372_549_02,
    b: 0.901_960_784_313_726,
};
pub const RED: ColorRGB = ColorRGB {
    r: 1.0,
    g: 0.0,
    b: 0.0,
};
pub const ROSYBROWN: ColorRGB = ColorRGB {
    r: 0.737_254_901_960_784,
    g: 0.560_784_313_725_49,
    b: 0.560_784_313_725_49,
};
pub const ROYALBLUE: ColorRGB = ColorRGB {
    r: 0.254_901_960_784_314,
    g: 0.411_764_705_882_353,
    b: 0.882_352_941_176_471,
};
pub const SADDLEBROWN: ColorRGB = ColorRGB {
    r: 0.545_098_039_215_686,
    g: 0.270_588_235_294_118,
    b: 0.074_509_803_921_568_6,
};
pub const SALMON: ColorRGB = ColorRGB {
    r: 0.980_392_156_862_745,
    g: 0.501_960_784_313_726,
    b: 0.447_058_823_529_412,
};
pub const SANDYBROWN: ColorRGB = ColorRGB {
    r: 0.956_862_745_098_039,
    g: 0.643_137_254_901_961,
    b: 0.376_470_588_235_294,
};
pub const SEAGREEN: ColorRGB = ColorRGB {
    r: 0.180_392_156_862_745,
    g: 0.545_098_039_215_686,
    b: 0.341_176_470_588_235,
};
pub const SEASHELL: ColorRGB = ColorRGB {
    r: 1.0,
    g: 0.960_784_313_725_49,
    b: 0.933_333_333_333_333,
};
pub const SIENNA: ColorRGB = ColorRGB {
    r: 0.627_450_980_392_157,
    g: 0.321_568_627_450_98,
    b: 0.176_470_588_235_294,
};
pub const SKYBLUE: ColorRGB = ColorRGB {
    r: 0.529_411_764_705_882,
    g: 0.807_843_137_254_902,
    b: 0.921_568_627_450_98,
};
pub const SLATEBLUE: ColorRGB = ColorRGB {
    r: 0.415_686_274_509_804,
    g: 0.352_941_176_470_588,
    b: 0.803_921_568_627_451,
};
pub const SLATEGRAY: ColorRGB = ColorRGB {
    r: 0.439_215_686_274_51,
    g: 0.501_960_784_313_726,
    b: 0.564_705_882_352_941,
};
pub const SLATEGREY: ColorRGB = ColorRGB {
    r: 0.439_215_686_274_51,
    g: 0.501_960_784_313_726,
    b: 0.564_705_882_352_941,
};
pub const SNOW: ColorRGB = ColorRGB {
    r: 1.0,
    g: 0.980_392_156_862_745,
    b: 0.980_392_156_862_745,
};
pub const SPRINGGREEN: ColorRGB = ColorRGB {
    r: 0.0,
    g: 1.0,
    b: 0.498_039_215_686_275,
};
pub const STEELBLUE: ColorRGB = ColorRGB {
    r: 0.274_509_803_921_569,
    g: 0.509_803_921_568_627,
    b: 0.705_882_352_941_177,
};
pub const TAN: ColorRGB = ColorRGB {
    r: 0.823_529_411_764_706,
    g: 0.705_882_352_941_177,
    b: 0.549_019_607_843_137,
};
pub const THISTLE: ColorRGB = ColorRGB {
    r: 0.847_058_823_529_412,
    g: 0.749_019_607_843_137,
    b: 0.847_058_823_529_412,
};
pub const TOMATO: ColorRGB = ColorRGB {
    r: 1.0,
    g: 0.388_235_294_117_647,
    b: 0.278_431_372_549_02,
};
pub const TURQUOISE: ColorRGB = ColorRGB {
    r: 0.250_980_392_156_863,
    g: 0.878_431_372_549_02,
    b: 0.815_686_274_509_804,
};
pub const VIOLET: ColorRGB = ColorRGB {
    r: 0.933_333_333_333_333,
    g: 0.509_803_921_568_627,
    b: 0.933_333_333_333_333,
};
pub const VIOLETRED: ColorRGB = ColorRGB {
    r: 0.815_686_274_509_804,
    g: 0.125_490_196_078_431,
    b: 0.564_705_882_352_941,
};
pub const WHEAT: ColorRGB = ColorRGB {
    r: 0.960_784_313_725_49,
    g: 0.870_588_235_294_118,
    b: 0.701_960_784_313_726,
};
pub const WHITE: ColorRGB = ColorRGB {
    r: 1.0,
    g: 1.0,
    b: 1.0,
};
pub const WHITESMOKE: ColorRGB = ColorRGB {
    r: 0.960_784_313_725_49,
    g: 0.960_784_313_725_49,
    b: 0.960_784_313_725_49,
};
pub const YELLOW: ColorRGB = ColorRGB {
    r: 1.0,
    g: 1.0,
    b: 0.0,
};
pub const YELLOWGREEN: ColorRGB = ColorRGB {
    r: 0.603_921_568_627_451,
    g: 0.803_921_568_627_451,
    b: 0.196_078_431_372_549,
};
