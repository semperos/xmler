// extern crate curl;
extern crate glob;
extern crate xml;

use glob::glob;

use std::env;
use std::fs::File;
// use std::io::{stdout, Write, BufReader};
use std::io::BufReader;

// use curl::easy::Easy;
use xml::reader::{EventReader, XmlEvent};

// fn fetch() {
//     let mut file = File::create("/tmp/sitemaps.zip")?;
//     let mut easy = Easy::new();
//     easy.url("https://www.rust-lang.org/").unwrap();
//     easy.write_function(|data| {
//         Ok(stdout().write(data).unwrap())
//     }).unwrap();
//     easy.perform().unwrap();

//     println!("{}", easy.response_code().unwrap());
// }

fn main() {
    let glob_pattern = env::args().last().unwrap();
    for entry in glob(&glob_pattern).expect("Failed to read glob pattern for XML files.") {
        match entry {
            Ok(path) => {
                let file = File::open(path).unwrap();
                let file = BufReader::new(file);

                let parser = EventReader::new(file);
                let mut current_tag = "sitemapindex".to_string();
                // let mut urls = Vec::new();

                for e in parser {
                    match e {
                        Ok(XmlEvent::StartElement { name, .. }) => {
                            current_tag = name.local_name;
                        }
                        Ok(XmlEvent::Characters(cs)) => {
                            if current_tag == "loc" {
                                println!("URL: {}", cs);
                            }
                        }
                        Err(e) => {
                            println!("Error: {}", e);
                            break;
                        }
                        _ => {}
                    }
                }
            }
            Err(e) => println!("Error: {:?}", e)
        }
    }
}
