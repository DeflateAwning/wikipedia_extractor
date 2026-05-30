use clap::{CommandFactory, Parser, ValueEnum, error::ErrorKind};
use indicatif::{ProgressBar, ProgressStyle};
use itertools::Itertools;
use std::fs::OpenOptions;
use std::io::Result;
use std::path;
use std::{fs::File, io::Write};
use wiki_extractor::WikipediaIterator;

#[derive(Debug, Clone, ValueEnum)]
enum OutputFormat {
    Files,
    SingleFileWithIndex,
}

/// Extract articles from a Wikipedia dump
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Path to the xml or xml.bz2 file containing the dump
    #[arg(short, long)]
    path: String,

    /// Number of articles to extract. -1 will extract all articles.
    #[arg(short, long, default_value_t = -1)]
    number_of_articles: i64,

    /// Where the output should be written to
    ///     files: Writes all files to the output_path directory
    ///     single-file-with-index: Generates two files <output_path>.txt and <output_path>.csv
    #[arg(short, long, verbatim_doc_comment)]
    output_path: String,

    /// Specify the format of the output:
    ///     files: Generates one file per article
    ///     single-file-with-index: Generates a single files containing all article texts and a CSV file.
    ///                             Every value in the CSV file represents the offset to an article.
    #[arg(long, verbatim_doc_comment)]
    output_format: OutputFormat,
}

fn main() {
    let args = Args::parse();
    match args.output_format {
        OutputFormat::Files => generate_files(args),
        OutputFormat::SingleFileWithIndex => generate_single_file_with_index(args),
    }
    .unwrap_or_else(|err| {
        Args::command()
            .error(ErrorKind::Io, format!("{err}"))
            .exit()
    });
}

fn generate_files(args: Args) -> Result<()> {
    let file = File::open(&args.path).unwrap_or_else(|err| {
        Args::command()
            .error(
                ErrorKind::Io,
                format!("Failed to open {}. {err}", args.path),
            )
            .exit()
    });
    let path_out = path::absolute(&args.output_path).unwrap_or_else(|err| {
        Args::command()
            .error(
                ErrorKind::Io,
                format!("Failed to create path from {}. {err}", args.output_path),
            )
            .exit()
    });
    if !path_out.exists() {
        std::fs::create_dir_all(&path_out).unwrap_or_else(|err| {
            Args::command()
                .error(
                    ErrorKind::Io,
                    &format!(
                        "Failed to create directory at {}. {err}",
                        path_out.to_str().unwrap_or("")
                    ),
                )
                .exit()
        });
    } else if !path_out.is_dir() {
        println!("The output path should be a directory.");
        Args::command()
            .error(
                ErrorKind::Io,
                &format!("{} is not a directory.", path_out.to_str().unwrap_or("")),
            )
            .exit();
    }
    let file_length = file.metadata()?.len();
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
    let article_iter = WikipediaIterator::new(&args.path)?;
    for article in article_iter.take(if args.number_of_articles > 0 {
        args.number_of_articles as usize
    } else {
        usize::MAX
    }) {
        let path = format!(
            "{}/{}.txt",
            path_out.to_str().expect("Strange output path"),
            article.title
        );
        let mut current_file = File::create(&path).unwrap_or_else(|err| {
            Args::command()
                .error(
                    ErrorKind::Io,
                    format!("Failed to create file {path}. {err}"),
                )
                .exit()
        });
        current_file
            .write_all(article.content.as_bytes())
            .expect("Failed to write article content.");
        if args.number_of_articles != -1 {
            bar.inc(1);
        } else {
            bar.inc(article.content.len() as u64);
        }
    }
    bar.finish();
    Ok(())
}

fn generate_single_file_with_index(args: Args) -> Result<()> {
    let file = File::open(&args.path).unwrap_or_else(|err| {
        Args::command()
            .error(
                ErrorKind::Io,
                format!("Failed to open {}. {err}", args.path),
            )
            .exit()
    });
    let file_length = file.metadata()?.len();
    let path_content_file = format!("{}.txt", args.output_path);
    File::create(&path_content_file).unwrap_or_else(|err| {
        Args::command()
            .error(
                ErrorKind::Io,
                format!("Failed to create file {path_content_file}. {err}"),
            )
            .exit()
    });
    let path_index_file = format!("{}.csv", args.output_path);
    File::create(&path_index_file).unwrap_or_else(|err| {
        Args::command()
            .error(
                ErrorKind::Io,
                format!("Failed to create file {path_index_file}. {err}"),
            )
            .exit()
    });
    let mut content_file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(&path_content_file)
        .unwrap_or_else(|err| {
            Args::command()
                .error(
                    ErrorKind::Io,
                    format!("Failed to open file {path_content_file}. {err}"),
                )
                .exit()
        });
    let mut index_file = OpenOptions::new()
        .create(true)
        .write(true)
        .open(&path_index_file)
        .unwrap_or_else(|err| {
            Args::command()
                .error(
                    ErrorKind::Io,
                    format!("Failed to open file {path_index_file}. {err}"),
                )
                .exit()
        });
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
    let mut offsets: Vec<usize> = Vec::new();
    let article_iter = WikipediaIterator::new(&args.path)?;
    for article in article_iter.take(if args.number_of_articles > 0 {
        args.number_of_articles as usize
    } else {
        usize::MAX
    }) {
        let text = article.content.trim();
        content_file
            .write_all(text.as_bytes())
            .unwrap_or_else(|err| {
                Args::command()
                    .error(
                        ErrorKind::Io,
                        format!("Failed to write content file. {err}"),
                    )
                    .exit()
            });
        offsets.push(current_text_offset);
        current_text_offset += text.len();
        if args.number_of_articles != -1 {
            bar.inc(1);
        } else {
            bar.inc(text.len() as u64);
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
        .unwrap_or_else(|err| {
            Args::command()
                .error(ErrorKind::Io, format!("Failed to write index file. {err}"))
                .exit()
        });
    bar.finish();
    Ok(())
}
