use clap::Parser;
use colored::Colorize;
use glob::glob;
use regex::Regex;
use std::{
    fs::File,
    io::{BufRead, BufReader},
    path::PathBuf,
};

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    /// Text or regex pattern to search for
    #[arg(value_name = "TEXT")]
    pattern: Option<String>,

    /// Files to search (supports glob patterns)
    #[arg(value_name = "FILE")]
    files: Option<String>,
}

fn main() {
    let cli = Cli::parse();
    if let Some(pattern) = &cli.pattern {
        println!("Searching for pattern: {}", pattern);
    }

    if let Some(files_pattern) = &cli.files {
        for path in glob(files_pattern).unwrap().filter_map(Result::ok) {
            if let Some(pattern) = &cli.pattern {
                process_file(&path, pattern);
            }
        }
    }
}

fn process_file(path: &PathBuf, pattern: &str) {
    let file = match File::open(path) {
        Ok(file) => file,
        Err(e) => {
            eprintln!("Failed to open {}: {}", path.display(), e);
            return;
        }
    };

    let regex = match Regex::new(pattern) {
        Ok(re) => re,
        Err(e) => {
            eprintln!("Invalid regex pattern '{}': {}", pattern, e);
            return;
        }
    };

    let mut has_matches = false;
    let reader = BufReader::new(file);

    for (line_num, line) in reader.lines().enumerate() {
        let line = match line {
            Ok(line) => line,
            Err(e) => {
                eprintln!(
                    "Error reading line {} in {}: {}",
                    line_num + 1,
                    path.display(),
                    e
                );
                continue;
            }
        };

        if let Some(highlighted) = highlight_match(&line, line_num, pattern, &regex) {
            if !has_matches {
                println!("{}:", path.display());
                has_matches = true;
            }
            println!("{}", highlighted);
        }
    }
}

fn highlight_match(line: &str, line_num: usize, pattern: &str, regex: &Regex) -> Option<String> {
    if line.contains(pattern) {
        // Simple string match
        let pos = line.find(pattern).unwrap();
        Some(format!(
            "{}:{} {}{}{}",
            (line_num + 1).to_string().blue(),
            (pos + 1).to_string().green(),
            &line[..pos],
            &line[pos..pos + pattern.len()].red().bold(),
            &line[pos + pattern.len()..]
        ))
    } else if let Some(m) = regex.find(line) {
        // Regex match
        Some(format!(
            "{}:{} {}{}{}",
            (line_num + 1).to_string().blue(),
            (m.start() + 1).to_string().green(),
            &line[..m.start()],
            &line[m.start()..m.end()].red().bold(),
            &line[m.end()..]
        ))
    } else {
        None
    }
}
