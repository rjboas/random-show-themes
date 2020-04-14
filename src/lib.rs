use std::error::Error;
use std::fs::File;
use std::io::BufReader;
use std::path::Path;
use std::str::FromStr;

use clap::{
    crate_authors, crate_description, crate_name, crate_version, App, Arg, ArgGroup, ArgMatches,
};
use serde::Deserialize;
use term_table::{row::Row, table_cell::TableCell, Table, TableStyle};

#[derive(Deserialize, Debug, Hash, Eq, PartialEq)]
pub struct Show {
    #[serde(alias = "mal_id")]
    pub id: usize,
    pub title: String,
    pub url: Option<String>,
    #[serde(default)]
    pub opening_themes: Vec<String>,
    #[serde(default)]
    pub ending_themes: Vec<String>,
    #[serde(default, alias = "soundtrack")]
    pub other_soundtrack: Vec<String>,
}

pub enum OutputMode {
    Table,
    Readable,
    CSV,
}

impl OutputMode {
    pub fn from_matches(matches: &ArgMatches) -> Self {
        if matches.is_present("table") {
            Self::Table
        } else if matches.is_present("readable") {
            Self::Readable
        } else if matches.is_present("csv") {
            Self::CSV
        } else {
            Self::Readable
        }
    }
}

pub fn create_clap_app<'a>() -> App<'a, 'a> {
    App::new(crate_name!())
        .about(crate_description!())
        .author(crate_authors!())
        .version(crate_version!())
        .args(&[
            Arg::with_name("dictionary")
                .help("The list of all known shows")
                .takes_value(true)
                .short("d")
                // .long("dictionary")
                .required(true),
            Arg::with_name("list")
                .help("The subset of shows to choose from the dictionary")
                .takes_value(true)
                .short("l")
                // .long("list")
                .required(true),
            Arg::with_name("number")
                .help("The number of results to output")
                .long_help(
"The number of results to output
Note: The program is not guarranteed to output the number of results specified if it is not possible with the provided inputs."
                )
                .takes_value(true)
                .short("n")
                .index(1)
                .required(true)
                .validator(pos_int_validate),
            Arg::with_name("hard-fail")
                .help("Exit with exit code 1 on any error")
                .long_help(
"Exit with exit code 1 on any error
Note: this will not necessarily prevent some output from reaching stdout before exiting."
                )
                .long("hard-fail"),
        ])
        // Logging arguments
        .args(&[
            Arg::with_name("verbosity")
                .short("v")
                .multiple(true)
                .help("Increase message verbosity"),
            Arg::with_name("quiet")
                .short("q")
                .long("quiet")
                .help("Silence all output"),
            Arg::with_name("timestamp")
                .long("timestamp")
                .help("Prepend log lines with a timestamp")
                .takes_value(true)
                .possible_values(&["none", "sec", "ms", "ns"]),
        ])
        // Output format arguments
        .args(&[
            Arg::with_name("table")
                .help("Sets output to a formatted table")
                .short("t")
                .long("table"),
            Arg::with_name("table width")
                .help("The number of results to show")
                .takes_value(true)
                .long("table-width")
                .requires("table")
                .validator(pos_int_validate),
            Arg::with_name("readable")
                .help("Sets output to human readable text")
                .long("readable"),
            Arg::with_name("csv").help("Sets output to csv").long("csv"),
        ])
        .group(ArgGroup::with_name("display").args(&["table", "readable", "csv"]))
}

pub fn set_up_logging(matches: &ArgMatches) {
    let verbose = matches.occurrences_of("verbosity") as usize;
    let quiet = matches.is_present("quiet");
    let ts = matches
        .value_of("timestamp")
        .map(|v| {
            stderrlog::Timestamp::from_str(v).unwrap_or_else(|_| {
                clap::Error {
                    message: "invalid value for 'timestamp'".into(),
                    kind: clap::ErrorKind::InvalidValue,
                    info: None,
                }
                .exit()
            })
        })
        .unwrap_or(stderrlog::Timestamp::Off);

    stderrlog::new()
        .module(module_path!())
        .quiet(quiet)
        .verbosity(verbose + 1) // change verbosity with no -v to warn
        .timestamp(ts)
        .init()
        .unwrap()
}

pub fn read_json_file<P, T>(path: P) -> Result<T, Box<dyn Error>>
where
    P: AsRef<Path>,
    for<'de> T: Deserialize<'de>,
{
    // Open the file in read-only mode with buffer.
    let file = File::open(path)?;
    let reader = BufReader::new(file);

    // Read the JSON contents of the file as an instance of T
    let result = serde_json::from_reader(reader)?;

    // Return the `User`.
    Ok(result)
}

/// Checks if the value can be parsed as a positive, non-zero integer
fn pos_int_validate(value: String) -> Result<(), String> {
    let error_msg = "must be a positive, non-zero integer";
    let value = value.parse::<usize>().map_err(|_| error_msg.to_owned())?;
    if value == 0 {
        Err(error_msg.to_owned())
    } else {
        Ok(())
    }
}

pub fn create_table<'a>(matches: &'a ArgMatches) -> Table<'a> {
    let mut table = Table::new();

    use terminal_size::{terminal_size, Height, Width};
    let width = matches
        .value_of("table width")
        .map(|s| (Width(s.parse().unwrap()), Height(20)))
        .unwrap_or(terminal_size().unwrap_or((Width(60), Height(20))));
    let (Width(width), _) = width;
    table.max_column_width = width as _;

    // Set table style (hardcoded)
    // Note: should this option be exposed to users?
    table.style = TableStyle::rounded();

    table
}

pub fn output_theme(
    choice: &String,
    show: &Show,
    output_mode: &OutputMode,
    table: &mut Option<Table>,
) -> Result<(), Box<dyn Error>> {
    let song_type = if show.opening_themes.contains(choice) {
        "OP"
    } else if show.ending_themes.contains(choice) {
        "ED"
    } else {
        "ST"
    };

    match output_mode {
        OutputMode::Table => {
            // Unwrap is ok if we know it definetly exists
            table.as_mut().unwrap().add_row(Row::new(vec![
                TableCell::new(choice),
                TableCell::new(&show.title),
                TableCell::new(song_type),
            ]));
        }
        OutputMode::Readable => {
            println!("{} [{}] from {}", choice, song_type, show.title);
        }
        OutputMode::CSV => {
            let mut wtr = csv::Writer::from_writer(std::io::stdout());
            wtr.write_record(&[choice, song_type, &show.title])?;
            wtr.flush()?;
        }
    }

    Ok(())
}

/// Appends `other` to `first` if `other` is not empty
pub fn smart_append<T: Clone>(first: &mut Vec<T>, other: &Vec<T>) {
    if !other.is_empty() {
        first.append(&mut other.clone());
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn pos_int_validation() {
        assert!(pos_int_validate("1".to_owned()).is_ok());
        assert!(pos_int_validate("99".to_owned()).is_ok());
        assert!(pos_int_validate("-2".to_owned()).is_err());
        assert!(pos_int_validate("0".to_owned()).is_err());
    }

    fn smart_appending_template<T: Clone>(
        a: T,
        b: T,
        c: T,
        d: T,
        e: T,
        f: T,
    ) -> ((Vec<T>, Vec<T>), (Vec<T>, Vec<T>)) {
        let mut first = vec![a.clone(), b.clone(), c.clone()];
        let other = vec![d.clone(), e.clone(), f.clone()];
        let other_bckp = other.clone();
        let expected = vec![a, b, c, d, e, f];
        smart_append(&mut first, &other);
        ((first, expected), (other, other_bckp))
    }

    #[test]
    fn smart_appending() {
        let (first, second) = smart_appending_template(1, 2, 3, 4, 5, 6);
        assert_eq!(first.0, first.1);
        assert_eq!(second.0, second.1);
        let (first, second) = smart_appending_template(
            "a".to_owned(),
            "b".to_owned(),
            "c".to_owned(),
            "d".to_owned(),
            "e".to_owned(),
            "f".to_owned(),
        );
        assert_eq!(first.0, first.1);
        assert_eq!(second.0, second.1);
    }
}
