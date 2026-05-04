use std::env;
use std::fs;
use std::io::{self, Read, Write};
use std::process::ExitCode;

use markdown_formatter_ja::wrap_markdown;

const DEFAULT_WIDTH: usize = 80;

struct Config {
    width: usize,
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
    let config = parse_args(env::args().skip(1))?;
    let input = read_input(config.path.as_deref())?;
    let output = wrap_markdown(&input, config.width);

    io::stdout()
        .write_all(output.as_bytes())
        .map_err(|error| error.to_string())
}

fn parse_args(args: impl IntoIterator<Item = String>) -> Result<Config, String> {
    let mut width = DEFAULT_WIDTH;
    let mut path = None;
    let mut args = args.into_iter();

    while let Some(arg) = args.next() {
        if arg == "--width" {
            let value = args
                .next()
                .ok_or_else(|| "missing value for --width".to_string())?;
            width = value
                .parse()
                .map_err(|_| format!("invalid width: {value}"))?;
        } else if arg.starts_with('-') {
            return Err(format!("unknown argument: {arg}"));
        } else if path.is_some() {
            return Err(format!("unexpected argument: {arg}"));
        } else {
            path = Some(arg);
        }
    }

    Ok(Config { width, path })
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
