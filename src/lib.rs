extern crate csv;
extern crate glob;
extern crate xml;

#[macro_use]
extern crate serde_derive;

use glob::glob;

use std::convert::From;
use std::collections::HashSet;
use std::fs::File;
use std::io::BufReader;

use xml::reader::{EventReader, XmlEvent};
use xml::name::{Name, OwnedName};

#[derive(Debug)]
pub struct UrlReport {
    pub page_urls: Vec<String>,
    /// Not currently in use
    image_urls: Vec<String>,
    pub files: Vec<String>,
    pub prefixes: HashSet<String>,
}

#[derive(Debug, Serialize)]
pub enum UrlType {
    Page,
    Image,
}

#[derive(Debug, Serialize)]
pub struct IndexableEntry {
    pub url: String,
    url_type: UrlType,
}

pub fn fresh_report() -> UrlReport {
    UrlReport {
        page_urls: Vec::new(),
        image_urls: Vec::new(),
        files: Vec::new(),
        prefixes: HashSet::new(),
    }
}

/// Return a `Page` indexable entry, images not supported at this time
pub fn indexable_entry(url: String) -> IndexableEntry {
    IndexableEntry {
        url,
        url_type: UrlType::Page,
    }
}

pub fn process(glob_pattern: &str, report: &mut UrlReport) {
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
                                        if let "image" = pre.as_ref() {
                                            if let "loc" = current_name.local_name.as_ref() {
                                                report.image_urls.push(cs);
                                            }
                                        }
                                    }
                                    None => {
                                        if let "loc" = current_name.local_name.as_ref() {
                                            report.page_urls.push(cs);
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_example() {
        assert_eq!(42, example(None));
        assert_eq!(23, example(Some(23)));
    }
}
