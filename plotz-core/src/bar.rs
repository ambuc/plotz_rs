#![allow(missing_docs)]

use indicatif::{ProgressBar, ProgressStyle};
use std::fmt::Debug;

pub fn make_bar<T>(len: T, msg: &'static str) -> ProgressBar
where
    <T as TryInto<u64>>::Error: Debug,
    u64: TryFrom<T>,
{
    let template = "[{elapsed_precise}] {bar:20.cyan/blue} {pos:>7}/{len:7} {msg}";
    let style = ProgressStyle::with_template(template).unwrap();

    let bar = ProgressBar::new(len.try_into().unwrap());
    bar.set_message(msg);
    bar.set_style(style);
    bar
}
