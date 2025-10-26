use clap::{Parser, ValueEnum};
use indicatif::{ProgressBar, ProgressStyle};
use itertools::Itertools;
use std::fs::OpenOptions;
use std::io::BufReader;
use std::{fs::File, io::Write};

use xml::reader::{EventReader, XmlEvent};

#[derive(Debug, Clone, ValueEnum)]
enum OutputFormat {
    Files,
    SingleFileWithIndex,
}

/// Extract articles from a Wikipedia dump
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// path to the xml file containing the dump
    #[arg(short, long)]
    path: String,

    /// number of articles to extract. -1 will extract all articles.
    #[arg(short, long, default_value_t = -1)]
    number_of_articles: i64,

    /// where the output should be written to
    #[arg(short, long)]
    output_path: String,

    /// the format of the output:
    ///     files: generates one file per article
    ///     single-file-with-index: generates a single files containing all article texts and a CSV
    ///     file. Every value in the CSV file represents the offset to an article.
    #[arg(long)]
    output_format: OutputFormat,
}

//TODO: move logic to lib
//TODO: handle slashes e.g. add missing / at path end for files parameter

fn main() {
    let args = Args::parse();
    match args.output_format {
        OutputFormat::Files => generate_files(args),
        OutputFormat::SingleFileWithIndex => generate_single_file_with_index(args),
    }
}

fn generate_files(args: Args) {
    let file = File::open(args.path).unwrap();
    let file_length = file.metadata().unwrap().len();
    let file = BufReader::new(file);
    let parser = EventReader::new(file);
    let bar = if args.number_of_articles != -1 {
        let bar = ProgressBar::new(args.number_of_articles as u64);
        bar.set_style(
            ProgressStyle::default_bar()
                .template("{bar:40.white/gray} {pos}/{len} ({eta})")
                .unwrap(),
        );
        bar
    } else {
        let bar = ProgressBar::new(file_length);
        bar.set_style(
            ProgressStyle::default_bar()
                .template("{bar:40.white/gray} {bytes}/{total_bytes} ({eta})")
                .unwrap(),
        );
        bar
    };
    let mut current_element_name = String::new();
    let mut current_file: Option<File> = None;
    let mut number_of_files_written = 0;
    for e in parser.into_iter() {
        if args.number_of_articles > 0 && args.number_of_articles <= number_of_files_written {
            break;
        }
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
                    current_file =
                        Some(File::create(format!("{}{}.txt", args.output_path, title)).unwrap());
                } else if current_element_name == "text" {
                    let text = text.trim();
                    current_file
                        .take()
                        .unwrap()
                        .write_all(text.as_bytes())
                        .unwrap();
                    number_of_files_written += 1;
                    if args.number_of_articles != -1 {
                        bar.inc(1);
                    } else {
                        bar.inc(text.len() as u64);
                    }
                }
            }
            Err(e) => {
                eprintln!("Error: {e}");
                break;
            }
            _ => {}
        }
    }
    bar.finish();
}

fn generate_single_file_with_index(args: Args) {
    let file = File::open(args.path).unwrap();
    let file_length = file.metadata().unwrap().len();
    let file = BufReader::new(file);
    let parser = EventReader::new(file);
    let mut current_element_name = String::new();
    File::create(format!("{}.txt", args.output_path)).unwrap();
    File::create(format!("{}.csv", args.output_path)).unwrap();
    let mut content_file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(format!("{}.txt", args.output_path))
        .unwrap();
    let mut index_file = OpenOptions::new()
        .create(true)
        .write(true)
        .open(format!("{}.csv", args.output_path))
        .unwrap();
    let bar = if args.number_of_articles != -1 {
        let bar = ProgressBar::new(args.number_of_articles as u64);
        bar.set_style(
            ProgressStyle::default_bar()
                .template("{bar:40.white/gray} {pos}/{len} ({eta})")
                .unwrap(),
        );
        bar
    } else {
        let bar = ProgressBar::new(file_length);
        bar.set_style(
            ProgressStyle::default_bar()
                .template("{bar:40.white/gray} {bytes}/{total_bytes} ({eta})")
                .unwrap(),
        );
        bar
    };
    let mut current_text_offset = 0;
    let mut number_of_articles_extracted = 0;
    let mut offsets: Vec<usize> = Vec::new();
    for e in parser.into_iter() {
        if args.number_of_articles > 0 && args.number_of_articles <= number_of_articles_extracted {
            break;
        }
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
                if current_element_name == "text" {
                    let text = text.trim();
                    content_file.write_all(text.as_bytes()).unwrap();
                    offsets.push(current_text_offset);
                    current_text_offset += text.len();
                    number_of_articles_extracted += 1;
                    if args.number_of_articles != -1 {
                        bar.inc(1);
                    } else {
                        bar.inc(text.len() as u64);
                    }
                }
            }
            Err(e) => {
                eprintln!("Error: {e}");
                break;
            }
            _ => {}
        }
    }
    index_file
        .write_all(
            offsets
                .iter()
                .map(|n| n.to_string())
                .collect_vec()
                .join(",")
                .as_bytes(),
        )
        .unwrap();
    bar.finish();
}
