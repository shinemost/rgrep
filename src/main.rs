use clap::Parser;
use colored::Colorize;
use glob::glob;
use regex::Regex;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::PathBuf;

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    /// 需要识别的文本或者正则表达式
    #[arg(value_name = "TEXT")]
    text: Option<String>,

    /// 需要做文本匹配的文件，支持通配符
    #[arg(value_name = "FILE")]
    file: Option<String>,
}

fn main() {
    let cli = Cli::parse();

    for ref path in glob(&cli.file.clone().unwrap())
        .unwrap()
        .filter_map(Result::ok)
    {
        read_file(path, cli.text.clone());
    }
}

fn read_file(path: &PathBuf, text: Option<String>) {
    let file = File::open(path).unwrap();
    let reader = BufReader::new(file);

    let mut has_match = false;
    let mut matched_lines = Vec::new();

    if let Some(ref text) = text {
        let regex = Regex::new(&format!(r"{}", text)).unwrap();
        for (line_num, line) in reader.lines().enumerate() {
            let line = line.unwrap();
            if line.contains(text.as_str()) || regex.is_match(&line) {
                if !has_match {
                    // 第一次匹配时打印文件名
                    println!("{}:", path.display());
                    has_match = true;
                }
                matched_lines.push((line_num, line));
            }
        }
    }

    // 打印所有匹配的行
    for (line_num, line) in matched_lines {
        print_match_result(text.clone(), line_num, line);
    }
}

fn print_match_result(text: Option<String>, line_num: usize, line: String) {
    if let Some(text) = text {
        if line.contains(text.as_str()) {
            let pos = line.find(text.as_str()).unwrap();
            let (before, matched) = line.split_at(pos);
            let (matched, after) = matched.split_at(text.len());

            println!(
                "{}:{} {}{}{}",
                (line_num + 1).to_string().blue(),
                (pos + 1).to_string().green(),
                before,
                matched.red().bold(),
                after
            );
        } else {
            // 处理正则表达式匹配
            let regex = Regex::new(&format!(r"{}", text)).unwrap();
            if let Some(m) = regex.find(&line) {
                let (before, matched) = line.split_at(m.start());
                let (matched, after) = matched.split_at(m.end() - m.start());

                println!(
                    "{}:{} {}{}{}",
                    (line_num + 1).to_string().blue(),
                    (m.start() + 1).to_string().green(),
                    before,
                    matched.red().bold(),
                    after
                );
            }
        }
    }
}
