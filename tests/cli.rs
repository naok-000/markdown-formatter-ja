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
        "これは日本語の文章です\n"
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
        "これは日本\n語の文章で\nす\n"
    );
}

#[test]
fn ignores_existing_line_breaks_by_default() {
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
        .write_all("1行目\n2行目2行目2行目2行目2行目\n3行目".as_bytes())
        .unwrap();

    let output = child.wait_with_output().unwrap();

    assert!(output.status.success());
    assert_eq!(
        String::from_utf8(output.stdout).unwrap(),
        "1行目2行目\n2行目2行目\n2行目2行目\n3行目\n"
    );
}

#[test]
fn preserves_existing_line_breaks_with_option() {
    let mut child = binary()
        .args(["--width", "10", "--preserve-line-breaks"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();

    child
        .stdin
        .as_mut()
        .unwrap()
        .write_all("1行目\n2行目2行目2行目2行目2行目\n3行目".as_bytes())
        .unwrap();

    let output = child.wait_with_output().unwrap();

    assert!(output.status.success());
    assert_eq!(
        String::from_utf8(output.stdout).unwrap(),
        "1行目\n2行目2行目\n2行目2行目\n2行目\n3行目\n"
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
        .write_all("あ".repeat(41).as_bytes())
        .unwrap();

    let output = child.wait_with_output().unwrap();

    assert!(output.status.success());
    assert_eq!(
        String::from_utf8(output.stdout).unwrap(),
        format!("{}\nあ\n", "あ".repeat(40))
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
        "これは日本\n語の文章で\nす\n"
    );
}

#[test]
fn writes_formatted_output_with_write_option() {
    let path = std::env::temp_dir().join(format!(
        "markdown-formatter-ja-cli-write-{}.md",
        std::process::id()
    ));
    fs::write(&path, "これは日本語の文章です").unwrap();

    let output = binary()
        .args(["--width", "10", "--write", path.to_str().unwrap()])
        .output()
        .unwrap();

    let file_content = fs::read_to_string(&path).unwrap();
    fs::remove_file(path).unwrap();

    assert!(output.status.success());
    assert_eq!(String::from_utf8(output.stdout).unwrap(), "");
    assert_eq!(file_content, "これは日本\n語の文章で\nす\n");
}

#[test]
fn rejects_write_without_file_path() {
    let output = binary().arg("--write").output().unwrap();

    assert!(!output.status.success());
    assert_eq!(String::from_utf8(output.stdout).unwrap(), "");
    assert!(
        String::from_utf8(output.stderr)
            .unwrap()
            .contains("--write")
    );
}

#[test]
fn rejects_invalid_cli_arguments() {
    let output = binary().arg("--unknown").output().unwrap();

    assert!(!output.status.success());
    assert_eq!(String::from_utf8(output.stdout).unwrap(), "");
    assert!(
        String::from_utf8(output.stderr)
            .unwrap()
            .contains("--unknown")
    );
}

#[test]
fn prints_help() {
    let output = binary().arg("--help").output().unwrap();

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("Usage:"));
    assert!(stdout.contains("--width"));
    assert!(stdout.contains("--preserve-line-breaks"));
    assert!(stdout.contains("--write"));
    assert_eq!(String::from_utf8(output.stderr).unwrap(), "");
}
