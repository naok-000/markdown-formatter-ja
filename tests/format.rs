use markdown_formatter_ja::{FormatOptions, LineBreakMode, format_markdown};

fn ignore(width: usize) -> FormatOptions {
    FormatOptions {
        width,
        line_break_mode: LineBreakMode::Ignore,
    }
}

fn preserve(width: usize) -> FormatOptions {
    FormatOptions {
        width,
        line_break_mode: LineBreakMode::Preserve,
    }
}

#[test]
fn can_ignore_line_breaks_inside_paragraphs() {
    let markdown = "1行目\n2行目2行目2行目2行目2行目\n3行目";

    assert_eq!(
        format_markdown(markdown, ignore(10)),
        "1行目2行目\n2行目2行目\n2行目2行目\n3行目\n"
    );
}

#[test]
fn can_ignore_line_breaks_inside_list_items() {
    let markdown = "- ああ\n  あああ";

    assert_eq!(
        format_markdown(markdown, ignore(6)),
        "- ああ\n  ああ\n  あ\n"
    );
}

#[test]
fn can_preserve_line_breaks_with_internal_mode() {
    let markdown = "1行目\n2行目2行目2行目2行目2行目\n3行目";

    assert_eq!(
        format_markdown(markdown, preserve(10)),
        "1行目\n2行目2行目\n2行目2行目\n2行目\n3行目\n"
    );
}

#[test]
fn keeps_list_items_separate_when_ignoring_line_breaks() {
    let markdown = "- ああ\n  ああ\n- いい\n  いい";

    assert_eq!(
        format_markdown(markdown, ignore(6)),
        "- ああ\n  ああ\n- いい\n  いい\n"
    );
}

#[test]
fn preserves_heading_as_single_commonmark_block() {
    assert_eq!(
        format_markdown("# これは日本語の見出しです", ignore(10)),
        "# これは日本語の見出しです\n"
    );
}

#[test]
fn preserves_bullet_list_marker_and_wraps_item_text() {
    assert_eq!(
        format_markdown("- これは日本語の項目です", ignore(10)),
        "- これは日\n  本語の項\n  目です\n"
    );
}

#[test]
fn preserves_ordered_list_marker_and_wraps_item_text() {
    assert_eq!(
        format_markdown("1. これは日本語の項目です", ignore(10)),
        "1. これは\n   日本語\n   の項目\n   です\n"
    );
}

#[test]
fn keeps_ascii_words_intact() {
    assert_eq!(
        format_markdown("これはmarkdownの文章です", ignore(10)),
        "これは\nmarkdownの\n文章です\n"
    );
}

#[test]
fn keeps_ascii_word_like_tokens_intact() {
    assert_eq!(
        format_markdown("foo_bar foo-bar example.com path/to/file", ignore(8)),
        "foo\\_bar \nfoo-bar \nexample.com\npath/to/file\n"
    );
}

#[test]
fn allows_ascii_words_to_exceed_width() {
    assert_eq!(
        format_markdown("short superlongword", ignore(8)),
        "short \nsuperlongword\n"
    );
}

#[test]
fn counts_multi_digit_ordered_list_marker_in_width() {
    assert_eq!(
        format_markdown("10. これは日本語の項目です", ignore(10)),
        "10. これは\n    日本語\n    の項目\n    です\n"
    );
}

#[test]
fn normalizes_top_level_list_indent() {
    assert_eq!(
        format_markdown("  - これは日本語の項目です", ignore(10)),
        "- これは日\n  本語の項\n  目です\n"
    );
}

#[test]
fn counts_nested_list_prefix_in_width() {
    let markdown = "- 親\n  - これは日本語の項目です";

    assert_eq!(
        format_markdown(markdown, ignore(10)),
        "- 親\n  - これは\n    日本語\n    の項目\n    です\n"
    );
}

#[test]
fn counts_blockquote_marker_in_width() {
    assert_eq!(
        format_markdown("> これは日本語の引用です", ignore(10)),
        "> これは日\n> 本語の引\n> 用です\n"
    );
}

#[test]
fn preserves_text_inside_code_fences() {
    let markdown = "```text\nこれは日本語の長いコードです\n```\n";

    assert_eq!(
        format_markdown(markdown, ignore(10)),
        "```text\nこれは日本語の長いコードです\n```\n"
    );
}

#[test]
fn preserves_front_matter_at_document_start() {
    let markdown = "---\ntitle: \"タイトル\"\nauthor: \"著者\"\ndate: \"2024-06-01\"\noutput: html_document\n---\n\n123456789";

    assert_eq!(
        format_markdown(markdown, ignore(5)),
        "---\ntitle: \"タイトル\"\nauthor: \"著者\"\ndate: \"2024-06-01\"\noutput: html_document\n---\n\n12345\n6789\n"
    );
}

#[test]
fn preserves_front_matter_when_preserving_line_breaks() {
    let markdown = "---\ntitle: \"タイトル\"\n---\n\n123456789";

    assert_eq!(
        format_markdown(markdown, preserve(5)),
        "---\ntitle: \"タイトル\"\n---\n\n12345\n6789\n"
    );
}

#[test]
fn does_not_wrap_inside_inline_code() {
    assert_eq!(
        format_markdown("これは`日本語のコード`です", ignore(10)),
        "これは\n`日本語のコード`\nです\n"
    );
}

#[test]
fn does_not_wrap_inside_links() {
    assert_eq!(
        format_markdown(
            "これは[日本語のリンク](https://example.com/)です",
            ignore(10)
        ),
        "これは\n[日本語のリンク](https://example.com/)\nです\n"
    );
}

#[test]
fn preserves_tables_as_markdown_structure() {
    let markdown = "| 見出し | 値 |\n| --- | --- |\n| これは長いセルです | 1 |\n";

    assert_eq!(
        format_markdown(markdown, ignore(10)),
        "| 見出し | 値 |\n| --- | --- |\n| これは長いセルです | 1 |\n"
    );
}

#[test]
fn preserves_task_list_markers() {
    assert_eq!(
        format_markdown("- [x] これは日本語の項目です", ignore(12)),
        "- [x] これは\n  日本語の項\n  目です\n"
    );
}
