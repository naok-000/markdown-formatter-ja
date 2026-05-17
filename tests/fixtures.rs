use markdown_formatter_ja::{EscapePolicy, FormatOptions, LineBreakMode, format_markdown};

struct Case {
    name: &'static str,
    input: &'static str,
    expected: &'static str,
    options: FormatOptions,
}

const IGNORE: FormatOptions = FormatOptions {
    width: 12,
    line_break_mode: LineBreakMode::Ignore,
    escape_policy: EscapePolicy::Conservative,
};

const MINIMAL: FormatOptions = FormatOptions {
    width: 1000,
    line_break_mode: LineBreakMode::Ignore,
    escape_policy: EscapePolicy::Minimal,
};

#[test]
fn formats_fixture_cases() {
    let cases = [
        Case {
            name: "front matter with code and list",
            input: include_str!("fixtures/front_matter_code_list.input.md"),
            expected: include_str!("fixtures/front_matter_code_list.expected.md"),
            options: IGNORE,
        },
        Case {
            name: "list blockquote task markers",
            input: include_str!("fixtures/list_blockquote_task.input.md"),
            expected: include_str!("fixtures/list_blockquote_task.expected.md"),
            options: IGNORE,
        },
        Case {
            name: "html block types under minimal escape",
            input: include_str!("fixtures/html_blocks.input.md"),
            expected: include_str!("fixtures/html_blocks.expected.md"),
            options: MINIMAL,
        },
        Case {
            name: "minimal escape preserves conservative meaning",
            input: include_str!("fixtures/minimal_escape_meaning.input.md"),
            expected: include_str!("fixtures/minimal_escape_meaning.expected.md"),
            options: MINIMAL,
        },
    ];

    for case in cases {
        assert_eq!(
            format_markdown(case.input, case.options),
            case.expected,
            "{}",
            case.name
        );
    }
}
