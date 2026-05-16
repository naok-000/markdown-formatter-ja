use markdown_formatter_ja::{EscapePolicy, FormatOptions, LineBreakMode, format_markdown};

// These cases track CommonMark 0.31.2 elements plus the comrak extensions
// enabled by this crate, so future escape-policy changes can be explicit.

struct Case {
    name: &'static str,
    input: &'static str,
    expected: &'static str,
}

fn format(input: &str) -> String {
    format_with_policy(input, LineBreakMode::Ignore, EscapePolicy::Conservative)
}

fn format_minimal(input: &str) -> String {
    format_with_policy(input, LineBreakMode::Ignore, EscapePolicy::Minimal)
}

fn format_preserving_line_breaks(input: &str) -> String {
    format_with_policy(input, LineBreakMode::Preserve, EscapePolicy::Conservative)
}

fn format_with_policy(
    input: &str,
    line_break_mode: LineBreakMode,
    escape_policy: EscapePolicy,
) -> String {
    format_markdown(
        input,
        FormatOptions {
            width: 1000,
            line_break_mode,
            escape_policy,
        },
    )
}

#[test]
fn captures_commonmark_block_element_rendering() {
    let cases = [
        Case {
            name: "thematic break",
            input: "---",
            expected: "-----\n",
        },
        Case {
            name: "atx heading",
            input: "# 見出し",
            expected: "# 見出し\n",
        },
        Case {
            name: "setext heading",
            input: "見出し\n===",
            expected: "# 見出し\n",
        },
        Case {
            name: "indented code block",
            input: "    code_line",
            expected: "```\ncode_line\n```\n",
        },
        Case {
            name: "fenced code block",
            input: "```rust\nfn main() {}\n```",
            expected: "```rust\nfn main() {}\n```\n",
        },
        Case {
            name: "html block",
            input: "<div>\nHTML\n</div>",
            expected: "<div>\nHTML\n</div>\n",
        },
        Case {
            name: "link reference definition used by reference link",
            input: "[label]: https://example.com/a_b\n\n[label]",
            expected: "[label](https://example.com/a_b)\n",
        },
        Case {
            name: "unused link reference definition",
            input: "[label]: https://example.com/a_b",
            expected: "",
        },
        Case {
            name: "paragraph",
            input: "これは本文です",
            expected: "これは本文です\n",
        },
        Case {
            name: "block quote",
            input: "> 引用です",
            expected: "> 引用です\n",
        },
        Case {
            name: "bullet list",
            input: "- 項目です",
            expected: "- 項目です\n",
        },
        Case {
            name: "ordered list",
            input: "1. 項目です",
            expected: "1. 項目です\n",
        },
    ];

    for case in cases {
        assert_eq!(format(case.input), case.expected, "{}", case.name);
    }
}

#[test]
fn captures_commonmark_inline_element_rendering() {
    let cases = [
        Case {
            name: "code span",
            input: "これは`a_b > c`です",
            expected: "これは`a_b > c`です\n",
        },
        Case {
            name: "emphasis",
            input: "これは*強調*です",
            expected: "これは*強調*です\n",
        },
        Case {
            name: "strong emphasis",
            input: "これは**強調**です",
            expected: "これは**強調**です\n",
        },
        Case {
            name: "link",
            input: "[a_b > c](https://example.com/a_b?q=1)",
            expected: "[a\\_b \\> c](https://example.com/a_b?q=1)\n",
        },
        Case {
            name: "link destination parentheses",
            input: "[link](https://example.com/a_(b))",
            expected: "[link](https://example.com/a_\\(b\\))\n",
        },
        Case {
            name: "link destination space",
            input: "[link](<https://example.com/a b>)",
            expected: "[link](https://example.com/a%20b)\n",
        },
        Case {
            name: "image",
            input: "![a_b > c](https://example.com/image.png \"a > b\")",
            expected: "![a\\_b \\> c](https://example.com/image.png \"a \\> b\")\n",
        },
        Case {
            name: "autolink uri",
            input: "<https://example.com/a_b?q=1>",
            expected: "<https://example.com/a_b?q=1>\n",
        },
        Case {
            name: "autolink email",
            input: "<user_name@example.com>",
            expected: "<user_name@example.com>\n",
        },
        Case {
            name: "raw html inline",
            input: "これは<span data-x=\"a_b\">HTML</span>です",
            expected: "これは<span data-x=\"a_b\">HTML</span>です\n",
        },
        Case {
            name: "hard line break",
            input: "foo\\\nbar",
            expected: "foo\\\nbar\n",
        },
    ];

    for case in cases {
        assert_eq!(format(case.input), case.expected, "{}", case.name);
    }
}

#[test]
fn captures_text_escaping_rendering() {
    let cases = [
        Case {
            name: "backslash escapes",
            input: "\\*not emphasized* and 1\\. not list and \\<br/>",
            expected: "\\*not emphasized\\* and 1. not list and \\<br/\\>\n",
        },
        Case {
            name: "entity references",
            input: "&copy; &amp; &notanentity;",
            expected: "© & &notanentity;\n",
        },
        Case {
            name: "ascii punctuation in text",
            input: "foo_bar > quote # hash [bracket] `tick` !bang &alpha; @user",
            expected: "foo\\_bar \\> quote \\# hash \\[bracket\\] `tick` \\!bang α \\@user\n",
        },
        Case {
            name: "bare urls from autolink extension",
            input: "see https://example.com/a_b?q=1 and www.example.com/a_b",
            expected: "see <https://example.com/a_b?q=1> and [www.example.com/a\\_b](http://www.example.com/a_b)\n",
        },
    ];

    for case in cases {
        assert_eq!(format(case.input), case.expected, "{}", case.name);
    }
}

#[test]
fn minimal_escape_policy_removes_safe_backslash_escapes() {
    let input = "foo_bar > quote # hash [bracket] !bang @user";

    assert_eq!(
        format_minimal(input),
        "foo_bar > quote # hash [bracket] !bang @user\n"
    );
}

#[test]
fn minimal_escape_policy_keeps_structural_escapes() {
    let input = "\\# heading\n\n\\- item\n\n1\\. item";

    assert_eq!(
        format_minimal(input),
        "\\# heading\n\n\\- item\n\n1\\. item\n"
    );
}

#[test]
fn minimal_escape_policy_keeps_code_span_content() {
    assert_eq!(format_minimal("`\\*code\\*`"), "`\\*code\\*`\n");
}

#[test]
fn minimal_escape_policy_removes_safe_escapes_around_code_spans() {
    assert_eq!(
        format_minimal("foo_bar and `\\*code\\*`"),
        "foo_bar and `\\*code\\*`\n"
    );
}

#[test]
fn minimal_escape_policy_removes_safe_escapes_around_raw_html() {
    assert_eq!(
        format_minimal("foo_bar <span data-x=\"\\*\">HTML</span>"),
        "foo_bar <span data-x=\"\\*\">HTML</span>\n"
    );
}

#[test]
fn minimal_escape_policy_removes_safe_escapes_around_structural_escapes() {
    let input = "foo_bar\n\n\\# heading\n\n1\\. item";

    assert_eq!(
        format_minimal(input),
        "foo_bar\n\n\\# heading\n\n1\\. item\n"
    );
}

#[test]
fn minimal_escape_policy_removes_safe_escapes_when_unsafe_escapes_are_present() {
    assert_eq!(
        format_minimal("foo_bar \\*not emphasized\\*"),
        "foo_bar \\*not emphasized\\*\n"
    );
}

#[test]
fn minimal_escape_policy_keeps_at_escape_that_suppresses_autolinks() {
    assert_eq!(format_minimal("foo\\@bar.com"), "foo\\@bar.com\n");
}

#[test]
fn minimal_escape_policy_keeps_closing_bracket_escape_in_link_labels() {
    let cases = [
        ("[foo\\]]: /url\n\n[foo\\]]", "[foo\\]](/url)\n"),
        ("[foo\\]bar]: /url\n\n[foo\\]bar]", "[foo\\]bar](/url)\n"),
    ];

    for (input, expected) in cases {
        assert_eq!(format_minimal(input), expected);
    }
}

#[test]
fn minimal_escape_policy_keeps_container_line_start_marker_escapes() {
    let cases = [
        ("> \\# not heading", "> \\# not heading\n"),
        ("> 1\\. not list", "> 1\\. not list\n"),
        ("- \\# not heading", "- \\# not heading\n"),
        ("- 1\\. not list", "- 1\\. not list\n"),
    ];

    for (input, expected) in cases {
        assert_eq!(format_minimal(input), expected);
    }
}

#[test]
fn minimal_escape_policy_preserves_conservative_rendering_after_reparse() {
    let cases = [
        ("escaped emphasis markers", "\\*not emphasized\\*"),
        ("escaped link marker", "\\[not link](https://example.com/)"),
        (
            "escaped image marker",
            "\\![not image](https://example.com/i.png)",
        ),
        ("table pipe", "| a |\n| --- |\n| \\| |"),
        ("code span", "`\\*code\\*`"),
        ("raw html", "<span data-x=\"\\*\">HTML</span>"),
    ];

    for (name, input) in cases {
        assert_eq!(format(&format_minimal(input)), format(input), "{}", name);
    }
}

#[test]
fn captures_commonmark_soft_line_break_when_line_breaks_are_preserved() {
    assert_eq!(format_preserving_line_breaks("foo\nbar"), "foo\nbar\n");
}

#[test]
fn captures_line_start_marker_escaping_when_line_breaks_are_preserved() {
    let input = "\\- literal\n\\+ literal\n\\= literal\n1\\. literal\n2\\) literal";

    assert_eq!(
        format_preserving_line_breaks(input),
        "\\- literal\n\\+ literal\n\\= literal\n1\\. literal\n2\\) literal\n"
    );
}

#[test]
fn captures_enabled_comrak_extension_rendering() {
    let cases = [
        Case {
            name: "front matter",
            input: "---\ntitle: a_b > c\n---\n\n本文",
            expected: "---\ntitle: a_b > c\n---\n\n本文\n",
        },
        Case {
            name: "table",
            input: "| a_b | x > y |\n| --- | --- |\n| c_d | e > f |",
            expected: "| a\\_b | x \\> y |\n| --- | --- |\n| c\\_d | e \\> f |\n",
        },
        Case {
            name: "task list",
            input: "- [x] a_b > c",
            expected: "- [x] a\\_b \\> c\n",
        },
        Case {
            name: "strikethrough",
            input: "これは~~a_b > c~~です",
            expected: "これは~~a\\_b \\> c~~です\n",
        },
        Case {
            name: "cjk friendly emphasis",
            input: "これは*強調*です",
            expected: "これは*強調*です\n",
        },
    ];

    for case in cases {
        assert_eq!(format(case.input), case.expected, "{}", case.name);
    }
}
