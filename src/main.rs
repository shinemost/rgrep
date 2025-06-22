use clap::Parser;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::PathBuf;

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    /// Optional text to operate on
    text: Option<String>,

    /// a custom config file
    #[arg(value_name = "FILE")]
    file: Option<PathBuf>,
}

fn main() {
    let cli = Cli::parse();

    let file = File::open(cli.file.unwrap()).unwrap();
    let reader = BufReader::new(file);
    
    for (line_num, line) in reader.lines().enumerate() {
        let line = line.unwrap();

        if let Some(text) = cli.text.as_deref() {
            if line.contains(text) {
                println!("{} : {}", line_num + 1, line);
            }
        }
    }
}
