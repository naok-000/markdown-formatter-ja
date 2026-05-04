use std::fs;
use std::io::Write;
use std::process::{Command, Stdio};

fn binary() -> Command {
    Command::new(env!("CARGO_BIN_EXE_markdown-formatter-ja"))
}

#[test]
fn formats_markdown_from_stdin_to_stdout() {
    let mut child = binary()
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();

    child
        .stdin
        .as_mut()
        .unwrap()
        .write_all("これは日本語の文章です".as_bytes())
        .unwrap();

    let output = child.wait_with_output().unwrap();

    assert!(output.status.success());
    assert_eq!(
        String::from_utf8(output.stdout).unwrap(),
        "これは日本語の文章です"
    );
    assert_eq!(String::from_utf8(output.stderr).unwrap(), "");
}

#[test]
fn accepts_width_option() {
    let mut child = binary()
        .args(["--width", "10"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();

    child
        .stdin
        .as_mut()
        .unwrap()
        .write_all("これは日本語の文章です".as_bytes())
        .unwrap();

    let output = child.wait_with_output().unwrap();

    assert!(output.status.success());
    assert_eq!(
        String::from_utf8(output.stdout).unwrap(),
        "これは日本\n語の文章で\nす"
    );
}

#[test]
fn uses_default_width_when_width_is_omitted() {
    let mut child = binary()
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();

    child
        .stdin
        .as_mut()
        .unwrap()
        .write_all("a".repeat(81).as_bytes())
        .unwrap();

    let output = child.wait_with_output().unwrap();

    assert!(output.status.success());
    assert_eq!(
        String::from_utf8(output.stdout).unwrap(),
        format!("{}\na", "a".repeat(80))
    );
}

#[test]
fn formats_markdown_from_file_path_to_stdout() {
    let path = std::env::temp_dir().join(format!(
        "markdown-formatter-ja-cli-{}.md",
        std::process::id()
    ));
    fs::write(&path, "これは日本語の文章です").unwrap();

    let output = binary()
        .args(["--width", "10", path.to_str().unwrap()])
        .output()
        .unwrap();

    fs::remove_file(path).unwrap();

    assert!(output.status.success());
    assert_eq!(
        String::from_utf8(output.stdout).unwrap(),
        "これは日本\n語の文章で\nす"
    );
}

#[test]
fn rejects_invalid_cli_arguments() {
    let output = binary().arg("--unknown").output().unwrap();

    assert!(!output.status.success());
    assert_eq!(String::from_utf8(output.stdout).unwrap(), "");
    assert_eq!(
        String::from_utf8(output.stderr).unwrap(),
        "error: unknown argument: --unknown\n"
    );
}
