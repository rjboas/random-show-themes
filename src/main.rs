use std::collections::HashMap;
use std::path::PathBuf;

use clap::ArgMatches;
use log::{error, info};
use rand::seq::SliceRandom;
use term_table::{row::Row, table_cell::TableCell, Table};

use random_show_themes::{
    create_clap_app, create_table, output_theme, read_json_file, set_up_logging, smart_append,
    OutputMode, Show,
};

fn main() {
    let matches = create_clap_app().get_matches();

    // Set up all logging stuff
    set_up_logging(&matches);

    match run(&matches) {
        Err(()) => std::process::exit(1),
        Ok(()) => {}
    }
}

fn run(matches: &ArgMatches) -> Result<(), ()> {
    // Get inital argument values
    let dictionary: PathBuf = matches.value_of("dictionary").unwrap().into();
    let list: PathBuf = matches.value_of("list").unwrap().into();
    let number_of_results: usize = matches.value_of("number").unwrap().parse().unwrap();

    let output_mode: OutputMode = OutputMode::from_matches(&matches);
    let hard_fail = matches.is_present("hard-fail");

    // Re-assign variables to parsed data
    let dictionary: HashMap<usize, Show> =
        read_json_file(dictionary).expect("couldn't parse dictionary into HashMap<usize, Show>");

    let list: Vec<usize> = read_json_file(list).expect("couldn't parse list into Vec<usize>");

    if dictionary.is_empty() {
        error!("dictionary cannot be empty");
        return Err(());
    } else if list.is_empty() {
        error!("list cannot be empty");
        return Err(());
    }

    let list_len = list.len();
    let number_of_results = if list_len < number_of_results {
        error!(
            "{} results were requested, however the list only contained {} entries",
            number_of_results, list_len
        );
        if hard_fail {
            return Err(());
        }
        info!("requesting {} results instead", { list_len });
        list_len
    } else {
        number_of_results
    };

    let mut rng = &mut rand::thread_rng();

    // Before result loop output
    let mut table = match output_mode {
        OutputMode::Table => {
            let mut table = create_table(&matches);

            table.add_row(Row::new(vec![
                TableCell::new("Song"),
                TableCell::new("Show"),
                TableCell::new("Type"),
            ]));

            Some(table)
        }
        OutputMode::Readable => None,
        OutputMode::CSV => {
            let mut wtr = csv::Writer::from_writer(std::io::stdout());
            if let Err(e) = wtr.write_record(&["Song", "Show", "Type"]) {
                error!("{}", e);
                return Err(());
            }
            if let Err(e) = wtr.flush() {
                error!("{}", e);
                return Err(());
            }
            None
        }
    };

    match result_loop(
        number_of_results,
        &list,
        &dictionary,
        &mut rng,
        &output_mode,
        &mut table,
    ) {
        Err(x) => {
            if hard_fail {
                return Err(x);
            }
        }
        Ok(_) => {}
    }

    // After result loop output
    match output_mode {
        OutputMode::Table => {
            // The table has to exist if the output mode is set to table
            println!("{}", table.as_mut().unwrap().render());
        }
        // No cleanup required for readable
        OutputMode::Readable => {}
        // We don't own and pass around the writer, we create a new one and flush it each time, so we don't flush it here
        OutputMode::CSV => {}
    }

    Ok(())
}

fn result_loop(
    number_of_results: usize,
    list: &Vec<usize>,
    dictionary: &HashMap<usize, Show>,
    rng: &mut impl rand::Rng,
    output_mode: &OutputMode,
    table: &mut Option<Table>,
) -> Result<(), ()> {
    let mut prev_res = Vec::with_capacity(number_of_results as _);
    let mut loop_res = Vec::with_capacity(number_of_results as _);
    for _ in 0..number_of_results {
        let a = loop {
            let res = list.choose(rng);

            // if the list is not empty `res` will be Some
            if let Some(res) = res {
                if !prev_res.contains(res) {
                    prev_res.push(res.clone());

                    if let Some(show) = dictionary.get(res) {
                        let mut themes = show.opening_themes.clone();
                        // Avoid clone + appending if we know there's nothing there
                        smart_append(&mut themes, &show.ending_themes);
                        smart_append(&mut themes, &show.other_soundtrack);

                        // if the list [of all themes] is not empty `choice` will be Some
                        if let Some(choice) = themes.choose(rng) {
                            if let Err(e) = output_theme(choice, show, output_mode, table) {
                                error!("{}", e);
                                // We don't have access to hard_fail, so we leave it up to the caller's error handling
                                break Err(());
                            };
                            break Ok(());
                        } else {
                            // try again for a different show
                            continue;
                        }
                    }
                } else if prev_res.len() == list.len() {
                    // If we've gone through everything we can and still don't have enough, there's nothing we can do
                    error!("not enough results were found");
                    break Err(());
                }
            }
        };
        loop_res.push(a);
    }
    match loop_res.contains(&Err(())) {
        true => Err(()),
        false => Ok(()),
    }
}
