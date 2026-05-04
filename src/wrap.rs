pub fn wrap_text(text: &str, width: usize) -> String {
    let mut output = String::new();
    let mut line_width = 0;

    for character in text.chars() {
        let character_width = display_width(character);

        if line_width > 0
            && line_width + character_width > width
            && !is_prohibited_line_start(character)
        {
            output.push('\n');
            line_width = 0;
        }

        output.push(character);
        line_width += character_width;
    }

    output
}

pub fn wrap_paragraphs(markdown: &str, width: usize) -> String {
    markdown
        .split("\n\n")
        .map(|paragraph| {
            if paragraph.is_empty() {
                String::new()
            } else {
                wrap_text(paragraph, width)
            }
        })
        .collect::<Vec<_>>()
        .join("\n\n")
}

pub fn wrap_markdown(markdown: &str, width: usize) -> String {
    let mut in_code_fence = false;

    markdown
        .lines()
        .map(|line| {
            if line.starts_with("```") {
                in_code_fence = !in_code_fence;
                return line.to_string();
            }

            if in_code_fence {
                return line.to_string();
            }

            wrap_markdown_line(line, width)
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn display_width(character: char) -> usize {
    if character.is_ascii() { 1 } else { 2 }
}

fn is_prohibited_line_start(character: char) -> bool {
    matches!(
        character,
        '、' | '。' | '，' | '．' | ',' | '.' | ')' | '）' | ']' | '】' | '}' | '」' | '』'
    )
}

fn heading_parts(line: &str) -> Option<(&str, &str)> {
    let marker_len = line
        .chars()
        .take_while(|character| *character == '#')
        .count();

    if (1..=6).contains(&marker_len) && line.as_bytes().get(marker_len) == Some(&b' ') {
        Some(line.split_at(marker_len + 1))
    } else {
        None
    }
}

fn wrap_markdown_line(line: &str, width: usize) -> String {
    if let Some((marker, text)) = heading_parts(line) {
        format!("{marker}{}", wrap_markdown_text(text, width))
    } else if let Some((prefix, text)) = list_item_parts(line) {
        wrap_with_prefix(prefix, text, width)
    } else {
        wrap_markdown_text(line, width)
    }
}

fn list_item_parts(line: &str) -> Option<(&str, &str)> {
    let indent_len = line
        .chars()
        .take_while(|character| *character == ' ')
        .map(char::len_utf8)
        .sum();
    let marker_start = &line[indent_len..];

    if marker_start.starts_with("- ") {
        return Some(line.split_at(indent_len + 2));
    }

    let period_index = marker_start.find('.')?;

    if period_index > 0
        && marker_start[..period_index]
            .chars()
            .all(|character| character.is_ascii_digit())
        && marker_start.as_bytes().get(period_index + 1) == Some(&b' ')
    {
        Some(line.split_at(indent_len + period_index + 2))
    } else {
        None
    }
}

fn wrap_with_prefix(prefix: &str, text: &str, width: usize) -> String {
    let prefix_width = text_width(prefix);
    let text_width = width.saturating_sub(prefix_width).max(1);
    let continuation = " ".repeat(prefix_width);

    format!(
        "{prefix}{}",
        wrap_markdown_text(text, text_width).replace('\n', &format!("\n{continuation}"))
    )
}

fn wrap_markdown_text(text: &str, width: usize) -> String {
    let mut output = String::new();
    let mut line_width = 0;
    let mut rest = text;

    while !rest.is_empty() {
        if let Some(token_end) = atomic_markdown_token_end(rest) {
            let token = &rest[..=token_end];
            let token_width = text_width(token);

            if line_width > 0 && line_width + token_width > width {
                output.push('\n');
                line_width = 0;
            }

            output.push_str(token);
            line_width += token_width;
            rest = &rest[token_end + 1..];
        } else {
            let character = rest.chars().next().unwrap();
            let character_width = display_width(character);

            if character == ' ' && line_width + character_width > width {
                output.push('\n');
                line_width = 0;
                rest = &rest[character.len_utf8()..];
                continue;
            }

            if line_width > 0
                && line_width + character_width > width
                && !is_prohibited_line_start(character)
            {
                output.push('\n');
                line_width = 0;
            }

            output.push(character);
            line_width += character_width;
            rest = &rest[character.len_utf8()..];
        }
    }

    output
}

fn atomic_markdown_token_end(text: &str) -> Option<usize> {
    inline_code_end(text)
        .or_else(|| link_end(text))
        .or_else(|| ascii_word_end(text))
}

fn inline_code_end(text: &str) -> Option<usize> {
    if !text.starts_with('`') {
        return None;
    }

    text[1..].find('`').map(|index| index + 1)
}

fn link_end(text: &str) -> Option<usize> {
    if !text.starts_with('[') {
        return None;
    }

    let label_end = text.find("](")?;
    let destination_start = label_end + 2;

    text[destination_start..]
        .find(')')
        .map(|index| destination_start + index)
}

fn ascii_word_end(text: &str) -> Option<usize> {
    let mut characters = text.char_indices();
    let (_, first) = characters.next()?;

    if !first.is_ascii_alphanumeric() {
        return None;
    }

    let mut end = first.len_utf8() - 1;

    for (index, character) in characters {
        if is_ascii_word_character(character) {
            end = index + character.len_utf8() - 1;
        } else {
            break;
        }
    }

    Some(end)
}

fn is_ascii_word_character(character: char) -> bool {
    character.is_ascii_alphanumeric() || matches!(character, '_' | '-' | '.' | '/')
}

fn text_width(text: &str) -> usize {
    text.chars().map(display_width).sum()
}

#[cfg(test)]
mod tests {
    use super::{wrap_markdown, wrap_paragraphs, wrap_text};

    #[test]
    fn wraps_text_by_display_width() {
        assert_eq!(
            wrap_text("これは日本語の文章です", 10),
            "これは日本\n語の文章で\nす"
        );
    }

    #[test]
    fn leaves_text_within_width_unchanged() {
        assert_eq!(wrap_text("これは日本語", 12), "これは日本語");
    }

    #[test]
    fn wraps_ascii_text_by_width() {
        assert_eq!(wrap_text("abcdef", 3), "abc\ndef");
    }

    #[test]
    fn does_not_start_wrapped_line_with_punctuation() {
        assert_eq!(
            wrap_text("これは日本、文章です", 10),
            "これは日本、\n文章です"
        );
    }

    #[test]
    fn does_not_start_wrapped_line_with_closing_bracket() {
        assert_eq!(
            wrap_text("これは日本）文章です", 10),
            "これは日本）\n文章です"
        );
    }

    #[test]
    fn wraps_multiple_paragraphs_independently() {
        let markdown = "abcdef\n\nこれは日本語の文章です";

        assert_eq!(
            wrap_paragraphs(markdown, 10),
            "abcdef\n\nこれは日本\n語の文章で\nす"
        );
    }

    #[test]
    fn preserves_blank_lines_between_paragraphs() {
        let markdown = "abcdef\n\n\n\nこれは日本語";

        assert_eq!(
            wrap_paragraphs(markdown, 10),
            "abcdef\n\n\n\nこれは日本\n語"
        );
    }

    #[test]
    fn preserves_heading_marker_and_wraps_heading_text() {
        assert_eq!(
            wrap_markdown("# これは日本語の見出しです", 10),
            "# これは日本\n語の見出し\nです"
        );
    }

    #[test]
    fn preserves_bullet_list_marker_and_wraps_item_text() {
        assert_eq!(
            wrap_markdown("- これは日本語の項目です", 10),
            "- これは日\n  本語の項\n  目です"
        );
    }

    #[test]
    fn preserves_ordered_list_marker_and_wraps_item_text() {
        assert_eq!(
            wrap_markdown("1. これは日本語の項目です", 10),
            "1. これは\n   日本語\n   の項目\n   です"
        );
    }

    #[test]
    fn keeps_ascii_words_intact() {
        assert_eq!(
            wrap_markdown("これはmarkdownの文章です", 10),
            "これは\nmarkdownの\n文章です"
        );
    }

    #[test]
    fn keeps_ascii_word_like_tokens_intact() {
        assert_eq!(
            wrap_markdown("foo_bar foo-bar example.com path/to/file", 8),
            "foo_bar \nfoo-bar \nexample.com\npath/to/file"
        );
    }

    #[test]
    fn allows_ascii_words_to_exceed_width() {
        assert_eq!(
            wrap_markdown("short superlongword", 8),
            "short \nsuperlongword"
        );
    }

    #[test]
    fn counts_multi_digit_ordered_list_marker_in_width() {
        assert_eq!(
            wrap_markdown("10. これは日本語の項目です", 10),
            "10. これは\n    日本語\n    の項目\n    です"
        );
    }

    #[test]
    fn counts_nested_list_indent_and_marker_in_width() {
        assert_eq!(
            wrap_markdown("  - これは日本語の項目です", 10),
            "  - これは\n    日本語\n    の項目\n    です"
        );
    }

    #[test]
    fn preserves_text_inside_code_fences() {
        let markdown = "```text\nこれは日本語の長いコードです\n```\n";

        assert_eq!(
            wrap_markdown(markdown, 10),
            "```text\nこれは日本語の長いコードです\n```"
        );
    }

    #[test]
    fn does_not_wrap_inside_inline_code() {
        assert_eq!(
            wrap_markdown("これは`日本語のコード`です", 10),
            "これは\n`日本語のコード`\nです"
        );
    }

    #[test]
    fn does_not_wrap_inside_links() {
        assert_eq!(
            wrap_markdown("これは[日本語のリンク](https://example.com/)です", 10),
            "これは\n[日本語のリンク](https://example.com/)\nです"
        );
    }
}
