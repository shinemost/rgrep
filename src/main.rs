use clap::Parser;
use colored::Colorize;
use glob::glob;
use regex::Regex;
use std::sync::Arc;
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

// 将匹配的两种情况封装到结构体中
struct SearchPattern {
    text: String,
    regex: Regex,
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    // 提前编译正则表达式并包装在Arc中以便共享，在线程间共享需要将变量包裹在Arc里
    let search_pattern = cli.pattern.as_ref().map(|pattern| {
        Arc::new(SearchPattern {
            text: pattern.clone(),
            regex: Regex::new(pattern).unwrap_or_else(|e| {
                eprintln!("Invalid regex pattern '{}': {}", pattern, e);
                std::process::exit(1);
            }),
        })
    });

    if let Some(files_pattern) = &cli.files {
        let files = glob(files_pattern)
            .unwrap()
            .filter_map(Result::ok)
            .collect::<Vec<_>>();

        let mut tasks = Vec::new();

        // 使用 tokio来异步解析文件
        for path in files {
            if let Some(sp) = search_pattern.clone() {
                tasks.push(tokio::spawn(async move {
                    process_file(&path, &sp).await;
                }))
            }
        }

        for task in tasks {
            let _ = task.await;
        }
    }
}

async fn process_file(path: &PathBuf, search_pattern: &Arc<SearchPattern>) {
    let file = match File::open(path) {
        Ok(file) => file,
        Err(e) => {
            eprintln!("Failed to open {}: {}", path.display(), e);
            return;
        }
    };

    let mut has_matches = false;
    let reader = BufReader::new(file);
    let mut matched_lines = Vec::new();

    for (line_num, line) in reader.lines().enumerate() {
        // 使用 match 模式匹配代替 unwrap，添加错误处理
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

        if let Some(highlighted) = highlight_match(&line, line_num, search_pattern) {
            if !has_matches {
                has_matches = true;
            }
            matched_lines.push(highlighted);
        }
    }
    // 一次性输出所有匹配行，减少锁竞争
    if has_matches {
        println!("{}:", path.display());
        for line in matched_lines {
            println!("{}", line);
        }
    }
}

fn highlight_match(
    line: &str,
    line_num: usize,
    search_pattern: &Arc<SearchPattern>,
) -> Option<String> {
    if line.contains(&search_pattern.text) {
        // 字符串精准匹配
        let pos = line.find(&search_pattern.text).unwrap();
        Some(format!(
            "{}:{} {}{}{}",
            (line_num + 1).to_string().blue(),
            (pos + 1).to_string().green(),
            // 使用切片，减少临时 vec
            &line[..pos],
            &line[pos..pos + search_pattern.text.len()].red().bold(),
            &line[pos + search_pattern.text.len()..]
        ))
    } else if let Some(m) = search_pattern.regex.find(line) {
        // 正则表达式匹配
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
