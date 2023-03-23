#![allow(unused)]
#![allow(missing_docs)]
//! Things which can be plotted with Cairo.

use crate::point::Pt;

pub trait PlotContext {
    fn line_to(&mut self, pt: &Pt);
    fn move_to(&mut self, pt: &Pt);
    fn select_font_face(&mut self, family: &str);
    fn set_font_size(&mut self, size: f64);
    fn show_text(&mut self, text: &str);
    fn arc(&mut self, xc: f64, yc: f64, radius: f64, angle1: f64, angle2: f64);
}

pub trait Plottable {
    fn plot<P: PlotContext>(&self,context: &mut P);
}
