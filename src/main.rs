extern crate csv;
extern crate glob;
extern crate xml;

#[macro_use]
extern crate serde_derive;

use glob::glob;

use std::convert::From;
use std::collections::HashSet;
use std::env;
use std::fs::File;
use std::io::BufReader;

use xml::reader::{EventReader, XmlEvent};
use xml::name::{Name, OwnedName};

#[derive(Debug)]
struct UrlReport {
    page_urls: Vec<String>,
    image_urls: Vec<String>,
    files: Vec<String>,
    prefixes: HashSet<String>,
}

#[derive(Debug, Serialize)]
enum UrlType {
    Page,
    Image,
}

#[derive(Debug, Serialize)]
struct IndexableEntry {
    url: String,
    url_type: UrlType,
}

fn process(glob_pattern: &str, report: &mut UrlReport) {
    for entry in glob(glob_pattern).expect("Failed to read glob pattern for XML files.") {
        match entry {
            Ok(path) => {
                report.files.push(String::from(path.to_str().unwrap()));
                let file = File::open(path).unwrap();
                let file = BufReader::new(file);

                let parser = EventReader::new(file);
                let mut current_name: OwnedName = Name {
                    local_name: "sitemapindex",
                    namespace: Some("http://www.sitemaps.org/schemas/sitemap/0.9"),
                    prefix: None,
                }.to_owned();
                let mut in_url_set = false;

                for e in parser {
                    match e {
                        Ok(XmlEvent::StartElement { name, .. }) => {
                            if current_name.local_name == "urlset" {
                                in_url_set = true;
                            }
                            current_name = name;
                        }
                        Ok(XmlEvent::Characters(cs)) => {
                            if in_url_set {
                                // TODO Understand why clone() is needed here, what's causing a move
                                match current_name.prefix.clone() {
                                    Some(pre) => {
                                        report.prefixes.insert(pre.clone());
                                        match pre.as_ref() {
                                            "image" => {
                                                match current_name.local_name.as_ref() {
                                                    "loc" => {
                                                        report.image_urls.push(cs);
                                                    }
                                                    _ => {}
                                                }
                                            }
                                            _ => {}
                                        }
                                    }
                                    None => {
                                        match current_name.local_name.as_ref() {
                                            "loc" => {
                                                report.page_urls.push(cs);
                                            }
                                            _ => {}
                                        }
                                    }
                                }
                            }
                        }
                        Err(e) => {
                            println!("Error: {:?}", e);
                            break;
                        }
                        _ => {}
                    }
                }
            }
            Err(e) => println!("Error: {:?}", e),
        }
    }
}

fn main() {
    let args = env::args();
    if args.len() < 2 || args.len() > 3 {
        println!(
            "
    USAGE: xmler <glob-pattern> [output-csv-file]

    You must supply a glob pattern representing the file patterns for xmler to search.

       Example: ./xmler 'abc/**/*.xml'

    If you then specify a file name, all URLs found will be written to it in CSV format.
    "
        );
        ::std::process::exit(1);
    }
    let glob_pattern = env::args().nth(1).unwrap();
    let report = &mut UrlReport {
        page_urls: Vec::new(),
        image_urls: Vec::new(),
        files: Vec::new(),
        prefixes: HashSet::new(),
    };
    println!("Processing all files that match your glob pattern...");
    process(&glob_pattern, report);
    println!(
        "
    Report:

        * {} Page URLs
        * {} Image URLs",
        report.page_urls.len(),
        report.image_urls.len()
    );
    let page_urls_set: HashSet<String> = report.page_urls.clone().into_iter().collect();
    let image_urls_set: HashSet<String> = report.image_urls.clone().into_iter().collect();
    println!(
        "
        * {} Unique Page URLs
        * {} Unique Image URLs",
        page_urls_set.len(),
        image_urls_set.len()
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
    match env::args().nth(2) {
        Some(file_path) => {
            println!("Writing output to CSV {}", file_path);
            match csv::Writer::from_path(file_path) {
                Ok(mut writer) => {
                    for url in page_urls_set {
                        match writer.serialize(IndexableEntry {
                            url: url,
                            url_type: UrlType::Page,
                        }) {
                            Ok(_) => {}
                            Err(e) => println!("Error: {:?}", e),
                        }
                    }
                    for url in image_urls_set {
                        match writer.serialize(IndexableEntry {
                            url: url,
                            url_type: UrlType::Image,
                        }) {
                            Ok(_) => {}
                            Err(e) => println!("Error: {:?}", e),
                        }
                    }
                    match writer.flush() {
                        Ok(_) => {}
                        Err(e) => println!("Error: {:?}", e),
                    }
                }
                Err(e) => println!("Error: {:?}", e),
            }
        }
        None => {}
    }
}
