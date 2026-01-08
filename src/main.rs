use clap::Parser;
use colored::Colorize;
use log::{debug, info, warn};
use regex::Regex;
use std::fmt::Debug;
use std::fs;

/// Regex404 is a tool to debug regular expressions on some content in a file.
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Path to file to check the pattern against
    #[arg(short, long)]
    file: String,

    /// Regex to run on {file}
    #[arg(short, long)]
    regex: Regex,
}

enum ProgError {
    IO(String),
    NoMatch,
}

/// Capture group contents
struct Cap {
    name: String,
    value: String,
}

fn main() -> Result<(), ProgError> {
    env_logger::builder()
        .filter_level(log::LevelFilter::Info)
        .parse_env("RUST_LOG")
        .init();
    let args = Args::parse();

    let file = &args.file;
    let haystack = fs::read_to_string(file)
        .map_err(|err| ProgError::IO(format!("failed to read file '{file}': {err}")))?;
    let re = args.regex;

    let coloring = colored::control::ShouldColorize::from_env().should_colorize();
    if !coloring {
        debug!("Disabling coloring as the environment doesn't seem to handle it.");
    }

    debug!("Parsed regex: {re}");

    if re.capture_names().len() <= 1 {
        warn!("no cap group");
        return Ok(());
    }

    for name in re.capture_names() {
        match name {
            Some(name) => {
                debug!("Looking for capture group '{name}'");
            }
            None => continue,
        }
    }

    let captures = match re.captures(&haystack) {
        None => return Err(ProgError::NoMatch),
        Some(cap) => cap,
    };

    let matcha = captures.get_match().as_str();
    debug!("Found match: {matcha}");

    let mut caps: Vec<Cap> = Vec::new();

    for name in re.capture_names() {
        match name {
            Some(name) => match captures.name(name) {
                Some(val) => {
                    let valstr = val.as_str();
                    let cap = Cap {
                        name: name.to_owned(),
                        value: valstr.to_owned(),
                    };
                    debug!("Found match: <{name}>={valstr}");
                    caps.push(cap);
                }
                None => warn!("Capture group <{name}> missing value."),
            },
            None => continue,
        }
    }

    let colors: Vec<colored::Color> = vec![
        colored::Color::Blue,
        colored::Color::Green,
        colored::Color::Red,
        colored::Color::Black,
    ];

    let mut regexstring = re.to_string();
    let mut regexstringprint = regexstring.to_owned();
    let mut matchstring = matcha.to_owned();
    let mut matches: Vec<String> = Vec::new();

    for (i, cap) in caps.into_iter().enumerate() {
        let color = colors[i % colors.len()];
        let Cap { name, value: val } = cap;
        let namecolor = name.color(color);
        let valcolor = val.color(color);

        let found = format!("<{namecolor}>: {valcolor}");
        debug!("{found}");

        if coloring {
            // Find the capture group name and expand coloring to the wrapping parentheses
            let mut regexstring_copy = regexstring.to_owned();
            let capgroup_name = format!("<{name}>");
            let capgroup_start = regexstring_copy.find(&capgroup_name);
            let mut capgroupstringfind = regexstring_copy.to_owned();
            capgroupstringfind
                .split_off(capgroup_start.expect("capgroup should have start"))
                .truncate(0);
            let capgroup_start2 = capgroupstringfind.rfind("(");
            let mut end: Option<usize> = None;
            let mut opened_parens = 1;
            for i in capgroup_start2.expect("capture group to have a start match") + 1
                ..regexstring_copy.len()
            {
                let c = regexstring_copy.chars().nth(i).expect("char should exist");

                // If we find other groups within this group, or the match includes parentheses,
                // make sure we keep searching for the end.
                if c == '(' {
                    opened_parens += 1;
                }
                if c == ')' {
                    opened_parens -= 1;
                    end = Some(i + 1); // include the wrapping )
                }
                if opened_parens == 0 {
                    break;
                }
            }
            let mut capg = regexstring_copy.split_off(capgroup_start2.unwrap());
            let capg_end = capg.split_off(end.unwrap() - regexstring_copy.len());
            regexstringprint = regexstring_copy + &capg.color(color).to_string() + &capg_end;
            regexstring = regexstringprint.to_string();
            matchstring = matchstring.replace(&val, &valcolor.to_string());
        }
        matches.push(found);
    }

    info!("Regex:");
    println!("{regexstringprint}");
    info!("Match:");
    println!("{matchstring}");
    info!("Capture groups:");
    matches.iter().for_each(|m| println!("{m}"));

    Ok(())
}

impl Debug for ProgError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let msg = match self {
            ProgError::IO(m) => m.to_owned(),
            ProgError::NoMatch => "found no matches".to_owned(),
        };
        f.write_str(&msg)
    }
}
