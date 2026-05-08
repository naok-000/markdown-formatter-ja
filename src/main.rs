use std::fs;
use std::io::{self, Read, Write};
use std::process::ExitCode;

use clap::Parser;
use markdown_formatter_ja::{
    wrap_markdown_ignore_break_mode, wrap_markdown_preserving_line_breaks,
};

const DEFAULT_WIDTH: usize = 80;

#[derive(Parser)]
#[command(about = "Format Markdown text for Japanese documents")]
struct Config {
    #[arg(long, default_value_t = DEFAULT_WIDTH)]
    width: usize,
    #[arg(long)]
    preserve_line_breaks: bool,
    #[arg(long, requires = "path")]
    write: bool,
    path: Option<String>,
}

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(message) => {
            eprintln!("error: {message}");
            ExitCode::FAILURE
        }
    }
}

fn run() -> Result<(), String> {
    let config = Config::parse();
    let input = read_input(config.path.as_deref())?;
    let output = if config.preserve_line_breaks {
        wrap_markdown_preserving_line_breaks(&input, config.width)
    } else {
        wrap_markdown_ignore_break_mode(&input, config.width)
    };

    if let Some(path) = &config.path
        && config.write
    {
        fs::write(path, output).map_err(|error| error.to_string())?;
        return Ok(());
    }

    io::stdout()
        .write_all(output.as_bytes())
        .map_err(|error| error.to_string())
}

fn read_input(path: Option<&str>) -> Result<String, String> {
    if let Some(path) = path {
        fs::read_to_string(path).map_err(|error| error.to_string())
    } else {
        let mut input = String::new();
        io::stdin()
            .read_to_string(&mut input)
            .map_err(|error| error.to_string())?;
        Ok(input)
    }
}
