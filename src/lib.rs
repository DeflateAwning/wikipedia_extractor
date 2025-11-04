use std::io::BufReader;
use std::{fs::File, io::Result};

use xml::reader::{EventReader, Events, XmlEvent};

/// An iterator over Wikipedia articles
pub struct WikipediaIterator {
    iter: Events<BufReader<File>>,
}

impl WikipediaIterator {
    /// create a new Iterator given the path to a Wikipedia XML dump
    pub fn new(path: &str) -> Result<WikipediaIterator> {
        let file = File::open(path)?;
        let file = BufReader::new(file);
        let parser = EventReader::new(file);
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
                    current_element_name = name
                        .to_string()
                        .split('}')
                        .next_back()
                        .unwrap()
                        .trim()
                        .to_string();
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
