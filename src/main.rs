extern crate clap;
extern crate csv;
extern crate xmler;

use clap::{App, Arg};
use std::collections::HashSet;
use std::process;

fn main() {
    let matches = App::new("xmler")
        .version("0.1.0")
        .author("Daniel Gregoire <daniel.l.gregoire@gmail.com>")
        .about("TBD")
        .arg(
            Arg::with_name("glob_pattern")
                .value_name("GLOB_PATTERN")
                .required(true)
                .index(1)
                .help("Glob pattern which limits which XML files to analyze"),
        )
        .arg(
            Arg::with_name("ouptut_file")
                .value_name("OUTPUT_FILE")
                .required(false)
                .help(
                    "Output file to which a CSV version of the report will be written",
                ),
        )
        .get_matches();

    // Required by options ^^^, so by here it will have a value.
    let glob_pattern = matches.value_of("glob_pattern").unwrap();
    let mut report = xmler::fresh_report();

    println!("Processing all files that match your glob pattern...");

    xmler::process(glob_pattern, &mut report);

    println!(
        "
    Report:

        * {} Page URLs",
        report.page_urls.len()
    );
    let page_urls_set: HashSet<String> = report.page_urls.clone().into_iter().collect();
    println!(
        "
        * {} Unique Page URLs",
        page_urls_set.len()
    );
    println!(
        "
        * {} files were consulted",
        report.files.len()
    );
    println!(
        "
        * XML prefixes encountered: {:?}",
        report.prefixes
    );

    // Persist report to CSV if output file-name specified
    if let Some(file_path) = matches.value_of("output_file") {
        println!("Writing output to CSV {}", file_path);
        match csv::Writer::from_path(file_path) {
            Ok(mut writer) => {
                for url in page_urls_set {
                    if let Err(e) = writer.serialize(xmler::indexable_entry(url)) {
                        eprintln!("Error writing entry to output file {}: {:?}", file_path, e);
                        process::exit(1);
                    }
                }
                if let Err(e) = writer.flush() {
                    eprintln!(
                        "Error flushing writer while writing to {}: {:?}",
                        file_path,
                        e
                    );
                    process::exit(1);
                }
            }
            Err(e) => {
                eprintln!("Error creating writer for {}: {:?}", file_path, e);
                process::exit(1);
            }
        }
    }
}
