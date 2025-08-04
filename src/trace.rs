use crate::{utils, OutputArgs, Result};
use clap::Parser;
use log::debug;
use regex::Regex;
use std::collections::HashSet;
use std::fmt::Write;
use std::fs;
use std::io::{self, Read};
use std::path::Path;

#[derive(Parser, Debug)]
#[command(about = "Extracts code snippets from a piped-in stack trace.")]
pub struct TraceArgs {
    /// Number of context lines to show around the error line.
    #[arg(short, long, default_value_t = 5)]
    context: usize,
    #[command(flatten)]
    output_args: OutputArgs,
}

pub fn run(args: TraceArgs) -> Result<()> {
    debug!("Reading from stdin to process trace log...");
    let mut buffer = String::new();
    io::stdin().read_to_string(&mut buffer)?;

    let mut output = String::new();
    let re = Regex::new(r"([a-zA-Z0-9_/\.\-]+):(\d+)")?;
    let mut seen = HashSet::new();

    for cap in re.captures_iter(&buffer) {
        if let (Some(file_match), Some(line_match)) = (cap.get(1), cap.get(2)) {
            let file_path_str = file_match.as_str();
            if let Ok(line_num) = line_match.as_str().parse::<usize>() {
                if line_num == 0 { continue; }

                let file_path = Path::new(file_path_str);
                if !file_path.exists() || !seen.insert(file_path.to_path_buf()) { continue; }

                debug!("Found reference to {} at line {}", file_path.display(), line_num);
                writeln!(&mut output, "--- START FILE: {} (line {}) ---", file_path.display(), line_num)?;
                match fs::read_to_string(file_path) {
                    Ok(content) => {
                        let lines: Vec<&str> = content.lines().collect();
                        let start = (line_num - 1).saturating_sub(args.context);
                        let end = (line_num).saturating_add(args.context).min(lines.len());

                        for i in start..end {
                            let current_line_num = i + 1;
                            let prefix = if current_line_num == line_num { ">>" } else { "  " };
                            if let Some(line_content) = lines.get(i) {
                                writeln!(&mut output, "{:>4} {} {}", current_line_num, prefix, line_content)?;
                            }
                        }
                    }
                    Err(_) => {
                        writeln!(&mut output, "[Could not read file]")?;
                    }
                }
                writeln!(&mut output, "--- END FILE: {} ---\n", file_path.display())?;
            }
        }
    }
    utils::handle_output(output, &args.output_args, "Trace context")
}
