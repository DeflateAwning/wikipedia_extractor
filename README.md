# wikipedia_extractor
A tool for parsing Wikipedia XML dumps.

## Building
```
cargo build --release
```

## Usage Bin
```
Extract articles from a Wikipedia dump

Usage: wiki_extractor [OPTIONS] --path <PATH> --output-path <OUTPUT_PATH> --output-format <OUTPUT_FORMAT>

Options:
  -p, --path <PATH>
          Path to the xml file containing the dump
  -n, --number-of-articles <NUMBER_OF_ARTICLES>
          Number of articles to extract. -1 will extract all articles [default: -1]
  -o, --output-path <OUTPUT_PATH>
          Where the output should be written to
              files: Writes all files to the output_path directory
              single-file-with-index: Generates two files <output_path>.txt and <output_path>.csv
      --output-format <OUTPUT_FORMAT>
          Specify the format of the output:
              files: Generates one file per article
              single-file-with-index: Generates a single files containing all article texts and a CSV file.
                                      Every value in the CSV file represents the offset to an article. [possible values: files, single-file-with-index]
  -h, --help
          Print help
  -V, --version
          Print version
```

### Example
Generate `out.txt` and `out.csv` for the first 1000 articles
```
cargo run --release -- -p ./wiki.xml -n 1000 -o ./out --output-format=single-file-with-index
```

Generate one file per article in `out/` for the first 1000 articles
```
cargo run --release -- -p ./wiki.xml -n 1000 -o ./out --output-format=files
```

## Usage Lib
```rust
use wiki_extractor::WikipediaIterator;
let article_iter = WikipediaIterator::new("./wiki.xml")?;
for article in article_iter {
  println!("title: {}\n content: {}", article.title, article.content);
}
```
