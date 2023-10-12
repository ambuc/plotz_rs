#![allow(missing_docs)]

use indicatif::{ProgressBar, ProgressStyle};

fn make_bar_style() -> ProgressStyle {
    ProgressStyle::with_template("[{elapsed_precise}] {bar:20.cyan/blue} {pos:>7}/{len:7} {msg}")
        .unwrap()
}

pub fn make_bar<T>(len: T, msg: &'static str) -> ProgressBar
where
    <T as TryInto<u64>>::Error: std::fmt::Debug,
    u64: TryFrom<T>,
{
    let bar = ProgressBar::new(len.try_into().unwrap());
    bar.set_message(msg);
    bar.set_style(make_bar_style());
    bar
}
