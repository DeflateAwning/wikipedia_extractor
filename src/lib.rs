#![doc = include_str!("../README.md")]
use bzip2::read::BzDecoder;
use std::io::{BufReader, Read};
use std::{fs::File, io::Result};

use xml::reader::{EventReader, Events, XmlEvent};

/// An iterator over Wikipedia articles
pub struct WikipediaIterator {
    iter: Events<BufReader<Box<dyn Read>>>,
}

impl WikipediaIterator {
    /// Create a new iterator given the path to a Wikipedia XML dump.
    /// Accepts both plain `.xml` files and bzip2-compressed `.xml.bz2` files.
    pub fn new(path: &str) -> Result<WikipediaIterator> {
        let file = File::open(path)?;
        let reader: Box<dyn Read> = if path.ends_with(".bz2") {
            Box::new(BzDecoder::new(file))
        } else {
            Box::new(file)
        };
        let parser = EventReader::new(BufReader::new(reader));
        let iter = parser.into_iter();

        Ok(WikipediaIterator { iter })
    }
}

/// A Wikipedia article
pub struct WikipediaArticle {
    /// the title of the article
    pub title: String,
    /// the content of the article
    pub content: String,
}

impl Iterator for WikipediaIterator {
    type Item = WikipediaArticle;

    /// get the next article
    fn next(&mut self) -> Option<Self::Item> {
        let mut current_element_name = String::new();
        let mut article = None;
        for e in &mut self.iter {
            match e {
                Ok(XmlEvent::StartElement { name, .. }) => {
                    current_element_name =
                        name.to_string().split('}').next_back()?.trim().to_string();
                }
                Ok(XmlEvent::Characters(text)) => {
                    if current_element_name == "title" {
                        let title = text.trim().replace(" ", "_").replace("/", "__");
                        article = Some(WikipediaArticle {
                            title,
                            content: "".to_string(),
                        });
                    } else if current_element_name == "text" {
                        let text = text.trim();
                        if let Some(article) = &mut article {
                            article.content = text.to_string();
                        }
                        return article.take();
                    }
                }
                Err(e) => {
                    eprintln!("Error: {e}");
                    break;
                }
                _ => {}
            }
        }
        None
    }
}
