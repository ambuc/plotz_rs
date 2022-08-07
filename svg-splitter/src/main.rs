use anyhow::Result;
use argh::FromArgs;
use lazy_static::lazy_static;
use regex::Regex;
use std::fs::File;
use std::io::{self, BufRead, BufReader, BufWriter, Lines, Write};

#[derive(FromArgs)]
#[argh(description = "...")]
struct Args {
    #[argh(option, description = "input SVG (full path)")]
    input: String,
    #[argh(option, description = "output SVG (prefix)")]
    output: String,
    #[argh(option, description = "if present, max # of lines per svg")]
    split: Option<u64>,
}

fn main_inner(args: Args) -> Result<()> {
    // read lines from input and copy to output
    let file = File::open(args.input).expect("open input");
    let mut lines = io::BufReader::new(file).lines();

    // assert first line
    let xml_header: String = lines.next().expect("first line").expect("first line");
    lazy_static! {
        static ref RE_1: Regex = Regex::new(r#"<\?xml.*>"#).unwrap();
    }
    assert!(RE_1.is_match(&xml_header));

    // assert second line
    let svg_header: String = lines.next().expect("second line").expect("second line");
    lazy_static! {
        static ref RE_2: Regex = Regex::new(r#"<svg.*>"#).unwrap();
    }
    assert!(RE_2.is_match(&svg_header));

    loop {
        if !consume_group(
            &args.output,
            &xml_header,
            &svg_header,
            &mut lines,
            &args.split,
            /*index=*/ if args.split.is_some() { Some(0) } else { None },
        )? {
            return Ok(());
        }
    }
}

fn main() {
    let args: Args = argh::from_env();
    main_inner(args).expect("ok");
}

fn consume_lines(
    output_prefix: &str,
    xml_header: &str,
    svg_header: &str,
    name: &str,
    group_open: &str,
    group_close: &str,
    group_subopen: &str,
    group_subclose: &str,
    lines: &mut Lines<BufReader<File>>,
    split: &Option<u64>,
    index: Option<u64>,
) -> Result<bool> {
    let f = File::create(
        [
            output_prefix.to_string(),
            "_".to_string(),
            name.to_string(),
            "_".to_string(),
            index.unwrap_or(0).to_string(),
            ".svg".to_string(),
        ]
        .concat(),
    )?;
    let mut f = BufWriter::new(f);
    f.write_all(xml_header.as_bytes())?;
    f.write_all("\n".as_bytes())?;

    f.write_all(svg_header.as_bytes())?;
    f.write_all("\n".as_bytes())?;

    f.write_all(group_open.as_bytes())?;
    f.write_all("\n".as_bytes())?;

    f.write_all(group_subopen.as_bytes())?;
    f.write_all("\n".as_bytes())?;

    let mut num_written = 0;

    loop {
        let line = lines.next().expect("a").expect("b");
        if line == group_subclose {
            // skip group_close matching line too.
            lines.next().expect("a").expect("b");
            break;
        }
        f.write_all(line.as_bytes())?;
        f.write_all("\n".as_bytes())?;
        num_written += 1;

        if let Some(sa) = split {
            if num_written > *sa {
                consume_lines(
                    output_prefix,
                    xml_header,
                    svg_header,
                    name,
                    group_open,
                    group_close,
                    group_subopen,
                    group_subclose,
                    lines,
                    split,
                    index.map(|x| x + 1),
                )?;
                break;
            }
        }
    }
    f.write_all(group_subclose.as_bytes())?;
    f.write_all("\n".as_bytes())?;

    f.write_all(group_close.as_bytes())?;
    f.write_all("\n".as_bytes())?;

    f.write_all("</svg>".to_string().as_bytes())?;
    f.write_all("\n".as_bytes())?;
    f.flush()?;
    return Ok(true);
}

// returns true if successful match; false if otherwise (file is done)
fn consume_group(
    output_prefix: &str,
    xml_header: &str,
    svg_header: &str,
    lines: &mut Lines<BufReader<File>>,
    split: &Option<u64>,
    index: Option<u64>,
) -> Result<bool> {
    lazy_static! {
        static ref RE_GROUP_OPEN: Regex =
            Regex::new(r#"(\s*)<g id="([A-Za-z_]*)\s([A-Za-z_]*)".*>"#).unwrap();
    }

    let group_open: String = lines.next().expect("next line").expect("next line");
    let caps = RE_GROUP_OPEN.captures_iter(&group_open).collect::<Vec<_>>();
    if caps.is_empty() {
        return Ok(false);
    }
    let indent = &caps[0].get(1).expect("indent");
    let name = &caps[0].get(3).expect("name");
    let group_close: String = [indent.as_str(), "</g>"].concat();

    lazy_static! {
        static ref RE_GROUP_SUBOPEN: Regex = Regex::new(r#"(\s*)<g.*>"#).unwrap();
    }
    let group_subopen: String = lines.next().expect("next line").expect("next line");
    let subcaps = RE_GROUP_SUBOPEN
        .captures_iter(&group_subopen)
        .collect::<Vec<_>>();
    if subcaps.is_empty() {
        return Ok(false);
    }
    let subindent = &subcaps[0].get(1).expect("indent");
    let group_subclose: String = [subindent.as_str(), "</g>"].concat();

    consume_lines(
        output_prefix,
        xml_header,
        svg_header,
        name.as_str(),
        group_open.as_str(),
        group_close.as_str(),
        group_subopen.as_str(),
        group_subclose.as_str(),
        lines,
        split,
        index,
    )?;

    Ok(true)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Read;
    use tempdir::TempDir;

    #[test]
    fn test_no_split() -> Result<()> {
        let tmp_dir = TempDir::new("example").expect("tmpdir");
        let tmp_dir_str: String = tmp_dir.path().to_str().unwrap().to_string();
        main_inner(Args {
            input: "testdata/in.svg".to_string(),
            output: tmp_dir_str.clone(),
            split: None,
        })?;

        for (actual_path, expected_path) in vec![
            (
                [tmp_dir_str.clone(), "_blue_0.svg".to_string()].concat(),
                "testdata/expected_out_blue.svg",
            ),
            (
                [tmp_dir_str.clone(), "_buildings_0.svg".to_string()].concat(),
                "testdata/expected_out_buildings.svg",
            ),
            (
                [tmp_dir_str.clone(), "_roads_0.svg".to_string()].concat(),
                "testdata/expected_out_roads.svg",
            ),
            (
                [tmp_dir_str.clone(), "_green_0.svg".to_string()].concat(),
                "testdata/expected_out_green.svg",
            ),
        ] {
            let mut actual = vec![];
            File::open(actual_path)?.read_to_end(&mut actual)?;

            let mut expected = vec![];
            File::open(expected_path)?.read_to_end(&mut expected)?;

            assert_eq!(actual, expected);
        }
        Ok(())
    }
}
