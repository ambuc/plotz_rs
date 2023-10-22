//! Run this with:
//!     cargo run --release -- --output-directory "/tmp/whatever"
//!         --input-glob "/path/to/your/svg/files*.svg" --draw-frame
//!         --width 800 --height 1000 --center-lat 123.45 --center-lng 234.56
//!         --scale-factor 1.1

#![deny(missing_docs)]

use anyhow::{anyhow, Result};
use argh::FromArgs;
use console::style;
use dialoguer::Confirm;
use glob::glob;
use indicatif::ProgressBar;
use lazy_static::lazy_static;
use regex::Regex;
use std::{
    fs::canonicalize,
    path::PathBuf,
    process::{Command, Stdio},
    time::Duration,
};

#[derive(FromArgs)]
#[argh(description = "...")]
struct Args {
    #[argh(option, description = "all svgs")]
    glob: String,
    #[argh(option, description = "frame svg")]
    frame: String,
    #[argh(option, description = "all svg")]
    all: String,
}

fn print_ok(s: &str) {
    println!("{} {}", style("OK").green(), s);
}

fn print_err(s: &str) {
    println!("{} {}", style("ERR").red(), s);
}

fn hits_yes(s: &str) -> Result<bool> {
    Ok(Confirm::new()
        .with_prompt(format!("{} {}", style("ACTION").magenta(), s))
        .interact()?)
}

fn is_command_ok(c: &mut Command) -> Result<Option<std::process::Output>> {
    match c.spawn()?.wait_with_output() {
        Ok(output) => {
            if output.status.success() {
                if let Ok(stdout) = std::str::from_utf8(&output.stdout) {
                    if !stdout.is_empty() {
                        print_ok(&format!("{} {}", style("stdout").green(), stdout));
                    }
                }
                if let Ok(stderr) = std::str::from_utf8(&output.stderr) {
                    if !stderr.is_empty() {
                        print_ok(&format!("{} {}", style("stderr").yellow(), stderr));
                    }
                }
                Ok(Some(output))
            } else {
                print_err(std::str::from_utf8(&output.stderr)?);
                Ok(None)
            }
        }
        Err(e) => {
            print_err(&format!("{:?}", e));
            Ok(None)
        }
    }
}

fn make_default_axicli_args() -> Vec<&'static str> {
    vec![
        "--pen_pos_down", // https://axidraw.com/doc/cli_api/#pen_pos_down
        "10",
        "--pen_pos_up", // https://axidraw.com/doc/cli_api/#pen_pos_up
        "45",
        "--speed_pendown", // https://axidraw.com/doc/cli_api/#speed_pendown
        "70",
        "--speed_penup", // https://axidraw.com/doc/cli_api/#speed_penup
        "100",
        "--reordering", // https://axidraw.com/doc/cli_api/#reordering
        "2",
        "--model", // https://axidraw.com/doc/cli_api/#model
        "2",
        "--progress",    // https://axidraw.com/doc/cli_api/#progress
        "--report_time", // https://axidraw.com/doc/cli_api/#report_time
    ]
}

fn manual_cmd(s: &str) -> Result<()> {
    Command::new("axicli")
        .args(vec!["--mode", "manual", "--manual_cmd", s])
        .output()?;
    Ok(())
}

fn disable_motors() -> Result<()> {
    manual_cmd("disable_xy")?;
    Ok(())
}

fn raise_pen() -> Result<()> {
    manual_cmd("raise_pen")?;
    Ok(())
}

fn toggle() -> Result<()> {
    Command::new("axicli")
        .args(vec!["--mode", "toggle"])
        .output()?;
    Ok(())
}

fn parse_prediction(s: &str) -> Option<Duration> {
    lazy_static! {
        static ref RE: Regex = Regex::new(r"(\d?\d:)?\d\d:\d\d").unwrap();
    }
    let captures: Vec<_> = RE.captures_iter(s).collect::<_>();
    if captures.is_empty() {
        return None;
    }
    let mut duration: Duration = Duration::from_secs(0);
    for capture in captures {
        match capture.get(0) {
            Some(hms) => {
                for (value, multiplier) in hms.as_str().split(':').rev().zip(vec![1, 60, 60 * 60]) {
                    match value.parse::<u64>() {
                        Ok(v) => {
                            duration += Duration::from_secs(v * multiplier);
                        }
                        _ => {
                            return None;
                        }
                    }
                }
            }
            None => {
                return None;
            }
        }
    }
    Some(duration)
}

fn do_layer(s: &str, special_name: Option<&str>) -> Result<()> {
    println!();
    let path: String = canonicalize(s)?.to_str().ok_or(anyhow!("?"))?.to_string();

    let predicted_duration: Option<Duration> = if hits_yes(&format!(
        "Preview {}{}",
        style(&s).blue(),
        special_name
            .map(|s| format!(" ({})", s))
            .unwrap_or_default()
    ))? {
        match is_command_ok(
            Command::new("axicli")
                .arg(&path)
                .arg("--preview")
                .arg("--report_time")
                .args(make_default_axicli_args())
                .stdout(Stdio::piped()),
        )? {
            Some(output) => parse_prediction(std::str::from_utf8(&output.stderr)?),
            _ => None,
        }
    } else {
        println!("{} Not previewing...", style("STOPPED").magenta(),);
        None
    };

    // done preview. quick toggle check.

    let mut n_toggles = 0;
    while hits_yes(&format!(
        "Toggle?{}",
        match n_toggles {
            0 => "".to_string(),
            _n => format!(" {}", style("(again)").red()),
        }
    ))? {
        n_toggles += 1;
        toggle()?;
    }

    print_ok("Raising pen.");
    raise_pen()?;

    let mut n_runs = 0;
    while hits_yes(&format!(
        "Print {}{}?{}{}",
        style(&s).blue(),
        special_name
            .map(|s| format!(" ({})", s))
            .unwrap_or_default(),
        predicted_duration.map_or_else(
            || "".to_string(),
            |s| format!("{}", style(format!(" Est. {:?}", s)).yellow())
        ),
        match n_runs {
            0 => "".to_string(),
            _n => format!(" {}", style("(again)").red()),
        }
    ))? {
        n_runs += 1;

        let (tx, rx) = std::sync::mpsc::channel::<()>();

        let progressbar_task = std::thread::spawn(move || {
            if let Some(duration) = predicted_duration {
                let n = duration.as_secs();
                let pb = ProgressBar::new(n).with_message("This layer");
                pb.set_style(
                    indicatif::ProgressStyle::default_bar()
                        .template("[{elapsed_precise}] {bar:40.yellow/red} {pos:>7}/{len:7} {msg}")
                        .unwrap()
                        .progress_chars("##-"),
                );

                let mut i = 0;
                while i < n {
                    pb.inc(1);
                    std::thread::sleep(std::time::Duration::from_secs(1));

                    match rx.try_recv() {
                        Ok(_) | Err(std::sync::mpsc::TryRecvError::Disconnected) => {
                            return;
                        }
                        Err(std::sync::mpsc::TryRecvError::Empty) => {}
                    }
                    i += 1;
                }
            }
        });

        let p2 = path.clone();
        let command_task = std::thread::spawn(move || {
            let _cmd = is_command_ok(
                Command::new("axicli")
                    .arg(&p2)
                    .args(make_default_axicli_args())
                    .stdout(Stdio::piped()),
            )
            .expect("?");
        });

        command_task.join().unwrap();

        // once command_task is done, cancel progressbar_task.
        let _ = tx.send(());
        progressbar_task.join().unwrap();

        disable_motors()?;
    }

    disable_motors()?;
    Ok(())
}

fn main() -> Result<()> {
    let args: Args = argh::from_env();

    let frame: PathBuf = glob(&args.frame)?.next().ok_or(anyhow!("?"))??;

    let all: PathBuf = glob(&args.all)?.next().ok_or(anyhow!("?"))??;

    // other files
    let files: Vec<PathBuf> = glob(&args.glob)?
        .filter(std::result::Result::is_ok)
        .map(std::result::Result::unwrap)
        .collect::<Vec<_>>();

    if files.is_empty() {
        panic!("{}: no matches for glob.", style("ERROR").red().bright());
    }

    let mut uniq = files
        .iter()
        .map(|f| f.display().to_string())
        .collect::<std::collections::HashSet<String>>();
    uniq.remove(frame.to_str().ok_or(anyhow!("?"))?);
    uniq.remove(all.to_str().ok_or(anyhow!("?"))?);

    println!();
    print_ok(&format!(
        "Found files: {:?}\n\t...will plot frame{:?}\n\t...and then other layers {:?}",
        style(files).blue(),
        style(&args.frame).blue().bright(),
        style(&uniq).blue(),
    ));
    println!();

    do_layer(&args.frame, Some("frame"))?;

    let mut layers_to_print: Vec<_> = uniq.iter().collect();
    layers_to_print.sort();

    let pb = ProgressBar::new(layers_to_print.len() as u64).with_message("All layers");
    pb.set_style(
        indicatif::ProgressStyle::default_bar()
            .template("[{elapsed_precise}] {bar:40.green/blue} {pos:>7}/{len:7} {msg}")?
            .progress_chars("##-"),
    );

    for layer in pb.wrap_iter(layers_to_print.iter()) {
        do_layer(layer, None)?;
    }
    Ok(())
}
