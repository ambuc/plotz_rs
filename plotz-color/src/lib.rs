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
    r: 0.941176470588235,
    g: 0.972549019607843,
    b: 1.0,
};
pub const ANTIQUEWHITE: ColorRGB = ColorRGB {
    r: 0.980392156862745,
    g: 0.92156862745098,
    b: 0.843137254901961,
};
pub const AQUAMARINE: ColorRGB = ColorRGB {
    r: 0.498039215686275,
    g: 1.0,
    b: 0.831372549019608,
};
pub const AZURE: ColorRGB = ColorRGB {
    r: 0.941176470588235,
    g: 1.0,
    b: 1.0,
};
pub const BEIGE: ColorRGB = ColorRGB {
    r: 0.96078431372549,
    g: 0.96078431372549,
    b: 0.862745098039216,
};
pub const BISQUE: ColorRGB = ColorRGB {
    r: 1.0,
    g: 0.894117647058824,
    b: 0.768627450980392,
};
pub const BLACK: ColorRGB = ColorRGB {
    r: 0.0,
    g: 0.0,
    b: 0.0,
};
pub const BLANCHEDALMOND: ColorRGB = ColorRGB {
    r: 1.0,
    g: 0.92156862745098,
    b: 0.803921568627451,
};
pub const BLUE: ColorRGB = ColorRGB {
    r: 0.0,
    g: 0.0,
    b: 1.0,
};
pub const BLUEVIOLET: ColorRGB = ColorRGB {
    r: 0.541176470588235,
    g: 0.168627450980392,
    b: 0.886274509803922,
};
pub const BROWN: ColorRGB = ColorRGB {
    r: 0.647058823529412,
    g: 0.164705882352941,
    b: 0.164705882352941,
};
pub const BURLYWOOD: ColorRGB = ColorRGB {
    r: 0.870588235294118,
    g: 0.72156862745098,
    b: 0.529411764705882,
};
pub const CADETBLUE: ColorRGB = ColorRGB {
    r: 0.372549019607843,
    g: 0.619607843137255,
    b: 0.627450980392157,
};
pub const CHARTREUSE: ColorRGB = ColorRGB {
    r: 0.498039215686275,
    g: 1.0,
    b: 0.0,
};
pub const CHOCOLATE: ColorRGB = ColorRGB {
    r: 0.823529411764706,
    g: 0.411764705882353,
    b: 0.117647058823529,
};
pub const CORAL: ColorRGB = ColorRGB {
    r: 1.0,
    g: 0.498039215686275,
    b: 0.313725490196078,
};
pub const CORNFLOWERBLUE: ColorRGB = ColorRGB {
    r: 0.392156862745098,
    g: 0.584313725490196,
    b: 0.929411764705882,
};
pub const CORNSILK: ColorRGB = ColorRGB {
    r: 1.0,
    g: 0.972549019607843,
    b: 0.862745098039216,
};
pub const CYAN: ColorRGB = ColorRGB {
    r: 0.0,
    g: 1.0,
    b: 1.0,
};
pub const DARKBLUE: ColorRGB = ColorRGB {
    r: 0.0,
    g: 0.0,
    b: 0.545098039215686,
};
pub const DARKCYAN: ColorRGB = ColorRGB {
    r: 0.0,
    g: 0.545098039215686,
    b: 0.545098039215686,
};
pub const DARKGOLDENROD: ColorRGB = ColorRGB {
    r: 0.72156862745098,
    g: 0.525490196078431,
    b: 0.0431372549019608,
};
pub const DARKGRAY: ColorRGB = ColorRGB {
    r: 0.662745098039216,
    g: 0.662745098039216,
    b: 0.662745098039216,
};
pub const DARKGREEN: ColorRGB = ColorRGB {
    r: 0.0,
    g: 0.392156862745098,
    b: 0.0,
};
pub const DARKGREY: ColorRGB = ColorRGB {
    r: 0.662745098039216,
    g: 0.662745098039216,
    b: 0.662745098039216,
};
pub const DARKKHAKI: ColorRGB = ColorRGB {
    r: 0.741176470588235,
    g: 0.717647058823529,
    b: 0.419607843137255,
};
pub const DARKMAGENTA: ColorRGB = ColorRGB {
    r: 0.545098039215686,
    g: 0.0,
    b: 0.545098039215686,
};
pub const DARKOLIVEGREEN: ColorRGB = ColorRGB {
    r: 0.333333333333333,
    g: 0.419607843137255,
    b: 0.184313725490196,
};
pub const DARKORANGE: ColorRGB = ColorRGB {
    r: 1.0,
    g: 0.549019607843137,
    b: 0.0,
};
pub const DARKORCHID: ColorRGB = ColorRGB {
    r: 0.6,
    g: 0.196078431372549,
    b: 0.8,
};
pub const DARKRED: ColorRGB = ColorRGB {
    r: 0.545098039215686,
    g: 0.0,
    b: 0.0,
};
pub const DARKSALMON: ColorRGB = ColorRGB {
    r: 0.913725490196078,
    g: 0.588235294117647,
    b: 0.47843137254902,
};
pub const DARKSEAGREEN: ColorRGB = ColorRGB {
    r: 0.56078431372549,
    g: 0.737254901960784,
    b: 0.56078431372549,
};
pub const DARKSLATEBLUE: ColorRGB = ColorRGB {
    r: 0.282352941176471,
    g: 0.23921568627451,
    b: 0.545098039215686,
};
pub const DARKSLATEGRAY: ColorRGB = ColorRGB {
    r: 0.184313725490196,
    g: 0.309803921568627,
    b: 0.309803921568627,
};
pub const DARKSLATEGREY: ColorRGB = ColorRGB {
    r: 0.184313725490196,
    g: 0.309803921568627,
    b: 0.309803921568627,
};
pub const DARKTURQUOISE: ColorRGB = ColorRGB {
    r: 0.0,
    g: 0.807843137254902,
    b: 0.819607843137255,
};
pub const DARKVIOLET: ColorRGB = ColorRGB {
    r: 0.580392156862745,
    g: 0.0,
    b: 0.827450980392157,
};
pub const DEBIANRED: ColorRGB = ColorRGB {
    r: 0.843137254901961,
    g: 0.0274509803921569,
    b: 0.317647058823529,
};
pub const DEEPPINK: ColorRGB = ColorRGB {
    r: 1.0,
    g: 0.0784313725490196,
    b: 0.576470588235294,
};
pub const DEEPSKYBLUE: ColorRGB = ColorRGB {
    r: 0.0,
    g: 0.749019607843137,
    b: 1.0,
};
pub const DIMGRAY: ColorRGB = ColorRGB {
    r: 0.411764705882353,
    g: 0.411764705882353,
    b: 0.411764705882353,
};
pub const DIMGREY: ColorRGB = ColorRGB {
    r: 0.411764705882353,
    g: 0.411764705882353,
    b: 0.411764705882353,
};
pub const DODGERBLUE: ColorRGB = ColorRGB {
    r: 0.117647058823529,
    g: 0.564705882352941,
    b: 1.0,
};
pub const FIREBRICK: ColorRGB = ColorRGB {
    r: 0.698039215686275,
    g: 0.133333333333333,
    b: 0.133333333333333,
};
pub const FLORALWHITE: ColorRGB = ColorRGB {
    r: 1.0,
    g: 0.980392156862745,
    b: 0.941176470588235,
};
pub const FORESTGREEN: ColorRGB = ColorRGB {
    r: 0.133333333333333,
    g: 0.545098039215686,
    b: 0.133333333333333,
};
pub const GAINSBORO: ColorRGB = ColorRGB {
    r: 0.862745098039216,
    g: 0.862745098039216,
    b: 0.862745098039216,
};
pub const GHOSTWHITE: ColorRGB = ColorRGB {
    r: 0.972549019607843,
    g: 0.972549019607843,
    b: 1.0,
};
pub const GOLD: ColorRGB = ColorRGB {
    r: 1.0,
    g: 0.843137254901961,
    b: 0.0,
};
pub const GOLDENROD: ColorRGB = ColorRGB {
    r: 0.854901960784314,
    g: 0.647058823529412,
    b: 0.125490196078431,
};
pub const GRAY: ColorRGB = ColorRGB {
    r: 0.745098039215686,
    g: 0.745098039215686,
    b: 0.745098039215686,
};
pub const GREEN: ColorRGB = ColorRGB {
    r: 0.0,
    g: 1.0,
    b: 0.0,
};
pub const GREENYELLOW: ColorRGB = ColorRGB {
    r: 0.67843137254902,
    g: 1.0,
    b: 0.184313725490196,
};
pub const HONEYDEW: ColorRGB = ColorRGB {
    r: 0.941176470588235,
    g: 1.0,
    b: 0.941176470588235,
};
pub const HOTPINK: ColorRGB = ColorRGB {
    r: 1.0,
    g: 0.411764705882353,
    b: 0.705882352941177,
};
pub const INDIANRED: ColorRGB = ColorRGB {
    r: 0.803921568627451,
    g: 0.36078431372549,
    b: 0.36078431372549,
};
pub const IVORY: ColorRGB = ColorRGB {
    r: 1.0,
    g: 1.0,
    b: 0.941176470588235,
};
pub const KHAKI: ColorRGB = ColorRGB {
    r: 0.941176470588235,
    g: 0.901960784313726,
    b: 0.549019607843137,
};
pub const LAVENDER: ColorRGB = ColorRGB {
    r: 0.901960784313726,
    g: 0.901960784313726,
    b: 0.980392156862745,
};
pub const LAVENDERBLUSH: ColorRGB = ColorRGB {
    r: 1.0,
    g: 0.941176470588235,
    b: 0.96078431372549,
};
pub const LAWNGREEN: ColorRGB = ColorRGB {
    r: 0.486274509803922,
    g: 0.988235294117647,
    b: 0.0,
};
pub const LEMONCHIFFON: ColorRGB = ColorRGB {
    r: 1.0,
    g: 0.980392156862745,
    b: 0.803921568627451,
};
pub const LIGHTBLUE: ColorRGB = ColorRGB {
    r: 0.67843137254902,
    g: 0.847058823529412,
    b: 0.901960784313726,
};
pub const LIGHTCORAL: ColorRGB = ColorRGB {
    r: 0.941176470588235,
    g: 0.501960784313726,
    b: 0.501960784313726,
};
pub const LIGHTCYAN: ColorRGB = ColorRGB {
    r: 0.87843137254902,
    g: 1.0,
    b: 1.0,
};
pub const LIGHTGOLDENROD: ColorRGB = ColorRGB {
    r: 0.933333333333333,
    g: 0.866666666666667,
    b: 0.509803921568627,
};
pub const LIGHTGRAY: ColorRGB = ColorRGB {
    r: 0.827450980392157,
    g: 0.827450980392157,
    b: 0.827450980392157,
};
pub const LIGHTGREEN: ColorRGB = ColorRGB {
    r: 0.564705882352941,
    g: 0.933333333333333,
    b: 0.564705882352941,
};
pub const LIGHTGREY: ColorRGB = ColorRGB {
    r: 0.827450980392157,
    g: 0.827450980392157,
    b: 0.827450980392157,
};
pub const LIGHTPINK: ColorRGB = ColorRGB {
    r: 1.0,
    g: 0.713725490196079,
    b: 0.756862745098039,
};
pub const LIGHTSALMON: ColorRGB = ColorRGB {
    r: 1.0,
    g: 0.627450980392157,
    b: 0.47843137254902,
};
pub const LIGHTSEAGREEN: ColorRGB = ColorRGB {
    r: 0.125490196078431,
    g: 0.698039215686275,
    b: 0.666666666666667,
};
pub const LIGHTSKYBLUE: ColorRGB = ColorRGB {
    r: 0.529411764705882,
    g: 0.807843137254902,
    b: 0.980392156862745,
};
pub const LIGHTSLATEBLUE: ColorRGB = ColorRGB {
    r: 0.517647058823529,
    g: 0.43921568627451,
    b: 1.0,
};
pub const LIGHTSLATEGRAY: ColorRGB = ColorRGB {
    r: 0.466666666666667,
    g: 0.533333333333333,
    b: 0.6,
};
pub const LIGHTSLATEGREY: ColorRGB = ColorRGB {
    r: 0.466666666666667,
    g: 0.533333333333333,
    b: 0.6,
};
pub const LIGHTSTEELBLUE: ColorRGB = ColorRGB {
    r: 0.690196078431373,
    g: 0.768627450980392,
    b: 0.870588235294118,
};
pub const LIGHTYELLOW: ColorRGB = ColorRGB {
    r: 1.0,
    g: 1.0,
    b: 0.87843137254902,
};
pub const LIMEGREEN: ColorRGB = ColorRGB {
    r: 0.196078431372549,
    g: 0.803921568627451,
    b: 0.196078431372549,
};
pub const LINEN: ColorRGB = ColorRGB {
    r: 0.980392156862745,
    g: 0.941176470588235,
    b: 0.901960784313726,
};
pub const MAGENTA: ColorRGB = ColorRGB {
    r: 1.0,
    g: 0.0,
    b: 1.0,
};
pub const MAROON: ColorRGB = ColorRGB {
    r: 0.690196078431373,
    g: 0.188235294117647,
    b: 0.376470588235294,
};
pub const MEDIUMAQUAMARINE: ColorRGB = ColorRGB {
    r: 0.4,
    g: 0.803921568627451,
    b: 0.666666666666667,
};
pub const MEDIUMBLUE: ColorRGB = ColorRGB {
    r: 0.0,
    g: 0.0,
    b: 0.803921568627451,
};
pub const MEDIUMORCHID: ColorRGB = ColorRGB {
    r: 0.729411764705882,
    g: 0.333333333333333,
    b: 0.827450980392157,
};
pub const MEDIUMPURPLE: ColorRGB = ColorRGB {
    r: 0.576470588235294,
    g: 0.43921568627451,
    b: 0.858823529411765,
};
pub const MEDIUMSEAGREEN: ColorRGB = ColorRGB {
    r: 0.235294117647059,
    g: 0.701960784313726,
    b: 0.443137254901961,
};
pub const MEDIUMSLATEBLUE: ColorRGB = ColorRGB {
    r: 0.482352941176471,
    g: 0.407843137254902,
    b: 0.933333333333333,
};
pub const MEDIUMSPRINGGREEN: ColorRGB = ColorRGB {
    r: 0.0,
    g: 0.980392156862745,
    b: 0.603921568627451,
};
pub const MEDIUMTURQUOISE: ColorRGB = ColorRGB {
    r: 0.282352941176471,
    g: 0.819607843137255,
    b: 0.8,
};
pub const MEDIUMVIOLETRED: ColorRGB = ColorRGB {
    r: 0.780392156862745,
    g: 0.0823529411764706,
    b: 0.52156862745098,
};
pub const MIDNIGHTBLUE: ColorRGB = ColorRGB {
    r: 0.0980392156862745,
    g: 0.0980392156862745,
    b: 0.43921568627451,
};
pub const MINTCREAM: ColorRGB = ColorRGB {
    r: 0.96078431372549,
    g: 1.0,
    b: 0.980392156862745,
};
pub const MISTYROSE: ColorRGB = ColorRGB {
    r: 1.0,
    g: 0.894117647058824,
    b: 0.882352941176471,
};
pub const MOCCASIN: ColorRGB = ColorRGB {
    r: 1.0,
    g: 0.894117647058824,
    b: 0.709803921568628,
};
pub const NAVAJOWHITE: ColorRGB = ColorRGB {
    r: 1.0,
    g: 0.870588235294118,
    b: 0.67843137254902,
};
pub const NAVY: ColorRGB = ColorRGB {
    r: 0.0,
    g: 0.0,
    b: 0.501960784313726,
};
pub const NAVYBLUE: ColorRGB = ColorRGB {
    r: 0.0,
    g: 0.0,
    b: 0.501960784313726,
};
pub const OLDLACE: ColorRGB = ColorRGB {
    r: 0.992156862745098,
    g: 0.96078431372549,
    b: 0.901960784313726,
};
pub const OLIVEDRAB: ColorRGB = ColorRGB {
    r: 0.419607843137255,
    g: 0.556862745098039,
    b: 0.137254901960784,
};
pub const ORANGE: ColorRGB = ColorRGB {
    r: 1.0,
    g: 0.647058823529412,
    b: 0.0,
};
pub const ORANGERED: ColorRGB = ColorRGB {
    r: 1.0,
    g: 0.270588235294118,
    b: 0.0,
};
pub const ORCHID: ColorRGB = ColorRGB {
    r: 0.854901960784314,
    g: 0.43921568627451,
    b: 0.83921568627451,
};
pub const PALEGOLDENROD: ColorRGB = ColorRGB {
    r: 0.933333333333333,
    g: 0.909803921568628,
    b: 0.666666666666667,
};
pub const PALEGREEN: ColorRGB = ColorRGB {
    r: 0.596078431372549,
    g: 0.984313725490196,
    b: 0.596078431372549,
};
pub const PALETURQUOISE: ColorRGB = ColorRGB {
    r: 0.686274509803922,
    g: 0.933333333333333,
    b: 0.933333333333333,
};
pub const PALEVIOLETRED: ColorRGB = ColorRGB {
    r: 0.858823529411765,
    g: 0.43921568627451,
    b: 0.576470588235294,
};
pub const PAPAYAWHIP: ColorRGB = ColorRGB {
    r: 1.0,
    g: 0.937254901960784,
    b: 0.835294117647059,
};
pub const PEACHPUFF: ColorRGB = ColorRGB {
    r: 1.0,
    g: 0.854901960784314,
    b: 0.725490196078431,
};
pub const PERU: ColorRGB = ColorRGB {
    r: 0.803921568627451,
    g: 0.52156862745098,
    b: 0.247058823529412,
};
pub const PINK: ColorRGB = ColorRGB {
    r: 1.0,
    g: 0.752941176470588,
    b: 0.796078431372549,
};
pub const PLUM: ColorRGB = ColorRGB {
    r: 0.866666666666667,
    g: 0.627450980392157,
    b: 0.866666666666667,
};
pub const POWDERBLUE: ColorRGB = ColorRGB {
    r: 0.690196078431373,
    g: 0.87843137254902,
    b: 0.901960784313726,
};
pub const RED: ColorRGB = ColorRGB {
    r: 1.0,
    g: 0.0,
    b: 0.0,
};
pub const ROSYBROWN: ColorRGB = ColorRGB {
    r: 0.737254901960784,
    g: 0.56078431372549,
    b: 0.56078431372549,
};
pub const ROYALBLUE: ColorRGB = ColorRGB {
    r: 0.254901960784314,
    g: 0.411764705882353,
    b: 0.882352941176471,
};
pub const SADDLEBROWN: ColorRGB = ColorRGB {
    r: 0.545098039215686,
    g: 0.270588235294118,
    b: 0.0745098039215686,
};
pub const SALMON: ColorRGB = ColorRGB {
    r: 0.980392156862745,
    g: 0.501960784313726,
    b: 0.447058823529412,
};
pub const SANDYBROWN: ColorRGB = ColorRGB {
    r: 0.956862745098039,
    g: 0.643137254901961,
    b: 0.376470588235294,
};
pub const SEAGREEN: ColorRGB = ColorRGB {
    r: 0.180392156862745,
    g: 0.545098039215686,
    b: 0.341176470588235,
};
pub const SEASHELL: ColorRGB = ColorRGB {
    r: 1.0,
    g: 0.96078431372549,
    b: 0.933333333333333,
};
pub const SIENNA: ColorRGB = ColorRGB {
    r: 0.627450980392157,
    g: 0.32156862745098,
    b: 0.176470588235294,
};
pub const SKYBLUE: ColorRGB = ColorRGB {
    r: 0.529411764705882,
    g: 0.807843137254902,
    b: 0.92156862745098,
};
pub const SLATEBLUE: ColorRGB = ColorRGB {
    r: 0.415686274509804,
    g: 0.352941176470588,
    b: 0.803921568627451,
};
pub const SLATEGRAY: ColorRGB = ColorRGB {
    r: 0.43921568627451,
    g: 0.501960784313726,
    b: 0.564705882352941,
};
pub const SLATEGREY: ColorRGB = ColorRGB {
    r: 0.43921568627451,
    g: 0.501960784313726,
    b: 0.564705882352941,
};
pub const SNOW: ColorRGB = ColorRGB {
    r: 1.0,
    g: 0.980392156862745,
    b: 0.980392156862745,
};
pub const SPRINGGREEN: ColorRGB = ColorRGB {
    r: 0.0,
    g: 1.0,
    b: 0.498039215686275,
};
pub const STEELBLUE: ColorRGB = ColorRGB {
    r: 0.274509803921569,
    g: 0.509803921568627,
    b: 0.705882352941177,
};
pub const TAN: ColorRGB = ColorRGB {
    r: 0.823529411764706,
    g: 0.705882352941177,
    b: 0.549019607843137,
};
pub const THISTLE: ColorRGB = ColorRGB {
    r: 0.847058823529412,
    g: 0.749019607843137,
    b: 0.847058823529412,
};
pub const TOMATO: ColorRGB = ColorRGB {
    r: 1.0,
    g: 0.388235294117647,
    b: 0.27843137254902,
};
pub const TURQUOISE: ColorRGB = ColorRGB {
    r: 0.250980392156863,
    g: 0.87843137254902,
    b: 0.815686274509804,
};
pub const VIOLET: ColorRGB = ColorRGB {
    r: 0.933333333333333,
    g: 0.509803921568627,
    b: 0.933333333333333,
};
pub const VIOLETRED: ColorRGB = ColorRGB {
    r: 0.815686274509804,
    g: 0.125490196078431,
    b: 0.564705882352941,
};
pub const WHEAT: ColorRGB = ColorRGB {
    r: 0.96078431372549,
    g: 0.870588235294118,
    b: 0.701960784313726,
};
pub const WHITE: ColorRGB = ColorRGB {
    r: 1.0,
    g: 1.0,
    b: 1.0,
};
pub const WHITESMOKE: ColorRGB = ColorRGB {
    r: 0.96078431372549,
    g: 0.96078431372549,
    b: 0.96078431372549,
};
pub const YELLOW: ColorRGB = ColorRGB {
    r: 1.0,
    g: 1.0,
    b: 0.0,
};
pub const YELLOWGREEN: ColorRGB = ColorRGB {
    r: 0.603921568627451,
    g: 0.803921568627451,
    b: 0.196078431372549,
};
