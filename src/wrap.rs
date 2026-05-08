pub fn wrap_markdown_ignore_break_mode(markdown: &str, width: usize) -> String {
    wrap_markdown(markdown, width, LineBreakMode::Ignore)
}

pub fn wrap_markdown_preserving_line_breaks(markdown: &str, width: usize) -> String {
    wrap_markdown(markdown, width, LineBreakMode::Preserve)
}

enum LineBreakMode {
    Preserve,
    Ignore,
}

enum MarkdownSegment {
    WrappableLine(String),
    WrappableText(String),
    PrefixedText { prefix: String, text: String },
    PreservedLine(String),
}

fn wrap_markdown(markdown: &str, width: usize, mode: LineBreakMode) -> String {
    let (front_matter, body) = split_front_matter(markdown);
    let mut segments = front_matter_segments(front_matter);

    segments.extend(match mode {
        LineBreakMode::Preserve => preserve_line_break_segments(body),
        LineBreakMode::Ignore => ignore_line_break_segments(body),
    });

    wrap_markdown_segments(segments, width)
}

fn split_front_matter(markdown: &str) -> (&str, &str) {
    let Some(end) = front_matter_end(markdown) else {
        return ("", markdown);
    };

    markdown.split_at(end)
}

fn front_matter_end(markdown: &str) -> Option<usize> {
    if !markdown.starts_with("---\n") {
        return None;
    }

    let mut offset = "---\n".len();

    for line in markdown[offset..].split_inclusive('\n') {
        if line.trim_end_matches('\n') == "---" {
            return Some(offset + line.len());
        }

        offset += line.len();
    }

    None
}

fn front_matter_segments(front_matter: &str) -> Vec<MarkdownSegment> {
    front_matter
        .lines()
        .map(|line| MarkdownSegment::PreservedLine(line.to_string()))
        .collect()
}

fn preserve_line_break_segments(markdown: &str) -> Vec<MarkdownSegment> {
    let mut segments = Vec::new();
    let mut in_code_fence = false;

    for line in markdown.lines() {
        if line.starts_with("```") {
            in_code_fence = !in_code_fence;
            segments.push(MarkdownSegment::PreservedLine(line.to_string()));
        } else if in_code_fence {
            segments.push(MarkdownSegment::PreservedLine(line.to_string()));
        } else {
            segments.push(MarkdownSegment::WrappableLine(line.to_string()));
        }
    }

    segments
}

fn ignore_line_break_segments(markdown: &str) -> Vec<MarkdownSegment> {
    let mut segments = Vec::new();
    let mut in_code_fence = false;
    let mut paragraph = String::new();
    let mut list_item: Option<(String, String)> = None;

    for line in markdown.lines() {
        if line.starts_with("```") {
            flush_paragraph_segment(&mut paragraph, &mut segments);
            flush_list_item_segment(&mut list_item, &mut segments);

            in_code_fence = !in_code_fence;
            segments.push(MarkdownSegment::PreservedLine(line.to_string()));
        } else if in_code_fence {
            segments.push(MarkdownSegment::PreservedLine(line.to_string()));
        } else if line.is_empty() {
            flush_paragraph_segment(&mut paragraph, &mut segments);
            flush_list_item_segment(&mut list_item, &mut segments);
            segments.push(MarkdownSegment::PreservedLine(String::new()));
        } else if heading_parts(line).is_some() {
            flush_paragraph_segment(&mut paragraph, &mut segments);
            flush_list_item_segment(&mut list_item, &mut segments);
            segments.push(MarkdownSegment::WrappableLine(line.to_string()));
        } else if let Some((prefix, text)) = list_item_parts(line) {
            flush_paragraph_segment(&mut paragraph, &mut segments);
            flush_list_item_segment(&mut list_item, &mut segments);
            list_item = Some((prefix.to_string(), text.to_string()));
        } else if let Some((_, text)) = &mut list_item {
            text.push_str(line.trim_start());
        } else {
            paragraph.push_str(line);
        }
    }

    flush_paragraph_segment(&mut paragraph, &mut segments);
    flush_list_item_segment(&mut list_item, &mut segments);

    segments
}

fn flush_paragraph_segment(paragraph: &mut String, segments: &mut Vec<MarkdownSegment>) {
    if !paragraph.is_empty() {
        segments.push(MarkdownSegment::WrappableText(std::mem::take(paragraph)));
    }
}

fn flush_list_item_segment(
    list_item: &mut Option<(String, String)>,
    segments: &mut Vec<MarkdownSegment>,
) {
    if let Some((prefix, text)) = list_item.take() {
        segments.push(MarkdownSegment::PrefixedText { prefix, text });
    }
}

fn wrap_markdown_segments(segments: Vec<MarkdownSegment>, width: usize) -> String {
    segments
        .into_iter()
        .map(|segment| match segment {
            MarkdownSegment::WrappableLine(line) => wrap_markdown_line(&line, width),
            MarkdownSegment::WrappableText(text) => wrap_markdown_text(&text, width),
            MarkdownSegment::PrefixedText { prefix, text } => {
                wrap_with_prefix(&prefix, &text, width)
            }
            MarkdownSegment::PreservedLine(line) => line,
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

    if !first.is_ascii_alphabetic() {
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
    use super::{
        LineBreakMode, wrap_markdown, wrap_markdown_ignore_break_mode,
        wrap_markdown_preserving_line_breaks,
    };

    #[test]
    fn can_ignore_line_breaks_inside_paragraphs() {
        let markdown = "1行目\n2行目2行目2行目2行目2行目\n3行目";

        assert_eq!(
            wrap_markdown_ignore_break_mode(markdown, 10),
            "1行目2行目\n2行目2行目\n2行目2行目\n3行目"
        );
    }

    #[test]
    fn can_ignore_line_breaks_inside_list_items() {
        let markdown = "- ああ\n  あああ";

        assert_eq!(
            wrap_markdown(markdown, 6, LineBreakMode::Ignore),
            "- ああ\n  ああ\n  あ"
        );
    }

    #[test]
    fn can_preserve_line_breaks_with_internal_mode() {
        let markdown = "1行目\n2行目2行目2行目2行目2行目\n3行目";

        assert_eq!(
            wrap_markdown_preserving_line_breaks(markdown, 10),
            "1行目\n2行目2行目\n2行目2行目\n2行目\n3行目"
        );
    }

    #[test]
    fn keeps_list_items_separate_when_ignoring_line_breaks() {
        let markdown = "- ああ\n  ああ\n- いい\n  いい";

        assert_eq!(
            wrap_markdown_ignore_break_mode(markdown, 6),
            "- ああ\n  ああ\n- いい\n  いい"
        );
    }

    #[test]
    fn preserves_heading_marker_and_wraps_heading_text() {
        assert_eq!(
            wrap_markdown_ignore_break_mode("# これは日本語の見出しです", 10),
            "# これは日本\n語の見出し\nです"
        );
    }

    #[test]
    fn preserves_bullet_list_marker_and_wraps_item_text() {
        assert_eq!(
            wrap_markdown_ignore_break_mode("- これは日本語の項目です", 10),
            "- これは日\n  本語の項\n  目です"
        );
    }

    #[test]
    fn preserves_ordered_list_marker_and_wraps_item_text() {
        assert_eq!(
            wrap_markdown_ignore_break_mode("1. これは日本語の項目です", 10),
            "1. これは\n   日本語\n   の項目\n   です"
        );
    }

    #[test]
    fn keeps_ascii_words_intact() {
        assert_eq!(
            wrap_markdown_ignore_break_mode("これはmarkdownの文章です", 10),
            "これは\nmarkdownの\n文章です"
        );
    }

    #[test]
    fn keeps_ascii_word_like_tokens_intact() {
        assert_eq!(
            wrap_markdown_ignore_break_mode("foo_bar foo-bar example.com path/to/file", 8),
            "foo_bar \nfoo-bar \nexample.com\npath/to/file"
        );
    }

    #[test]
    fn allows_ascii_words_to_exceed_width() {
        assert_eq!(
            wrap_markdown_ignore_break_mode("short superlongword", 8),
            "short \nsuperlongword"
        );
    }

    #[test]
    fn counts_multi_digit_ordered_list_marker_in_width() {
        assert_eq!(
            wrap_markdown_ignore_break_mode("10. これは日本語の項目です", 10),
            "10. これは\n    日本語\n    の項目\n    です"
        );
    }

    #[test]
    fn counts_nested_list_indent_and_marker_in_width() {
        assert_eq!(
            wrap_markdown_ignore_break_mode("  - これは日本語の項目です", 10),
            "  - これは\n    日本語\n    の項目\n    です"
        );
    }

    #[test]
    fn preserves_text_inside_code_fences() {
        let markdown = "```text\nこれは日本語の長いコードです\n```\n";

        assert_eq!(
            wrap_markdown_ignore_break_mode(markdown, 10),
            "```text\nこれは日本語の長いコードです\n```"
        );
    }

    #[test]
    fn preserves_front_matter_at_document_start() {
        let markdown = "---\ntitle: \"タイトル\"\nauthor: \"著者\"\ndate: \"2024-06-01\"\noutput: html_document\n---\n\n123456789";

        assert_eq!(
            wrap_markdown_ignore_break_mode(markdown, 5),
            "---\ntitle: \"タイトル\"\nauthor: \"著者\"\ndate: \"2024-06-01\"\noutput: html_document\n---\n\n12345\n6789"
        );
    }

    #[test]
    fn preserves_front_matter_when_preserving_line_breaks() {
        let markdown = "---\ntitle: \"タイトル\"\n---\n\n123456789";

        assert_eq!(
            wrap_markdown_preserving_line_breaks(markdown, 5),
            "---\ntitle: \"タイトル\"\n---\n\n12345\n6789"
        );
    }

    #[test]
    fn does_not_wrap_inside_inline_code() {
        assert_eq!(
            wrap_markdown_ignore_break_mode("これは`日本語のコード`です", 10),
            "これは\n`日本語のコード`\nです"
        );
    }

    #[test]
    fn does_not_wrap_inside_links() {
        assert_eq!(
            wrap_markdown_ignore_break_mode("これは[日本語のリンク](https://example.com/)です", 10),
            "これは\n[日本語のリンク](https://example.com/)\nです"
        );
    }
}
