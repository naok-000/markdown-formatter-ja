use std::ops::Range;

use super::line::{is_before, skip_spaces};

pub(super) fn html_block_ranges(markdown: &str) -> Vec<Range<usize>> {
    let mut ranges = Vec::new();
    let mut block_start = None;
    let mut block_end = HtmlBlockEnd::BlankLine;
    let mut line_start = 0;

    for line in markdown.split_inclusive('\n') {
        let line_end = line_start + line.len();
        let content = line.trim_end_matches('\n').trim_end_matches('\r');

        if let Some(start) = block_start {
            if block_end.is_complete(content.as_bytes()) {
                ranges.push(start..line_end);
                block_start = None;
            }
        } else if let Some(end) = html_block_start(content.as_bytes()) {
            if end.is_complete(content.as_bytes()) {
                ranges.push(line_start..line_end);
            } else {
                block_start = Some(line_start);
                block_end = end;
            }
        }

        line_start = line_end;
    }

    if let Some(start) = block_start {
        ranges.push(start..markdown.len());
    }

    ranges
}

#[derive(Clone, Copy)]
enum HtmlBlockEnd {
    BlankLine,
    Contains(&'static [u8]),
}

impl HtmlBlockEnd {
    fn is_complete(self, line: &[u8]) -> bool {
        match self {
            Self::BlankLine => line.is_empty(),
            Self::Contains(pattern) => contains_ignore_ascii_case(line, pattern),
        }
    }
}

fn html_block_start(line: &[u8]) -> Option<HtmlBlockEnd> {
    let start = skip_spaces(line, 0, line.len(), 3);

    if !is_before(line, start, b'<') {
        return None;
    }

    let rest = &line[start..];

    if starts_complete_open_tag(rest, b"script") {
        return Some(HtmlBlockEnd::Contains(b"</script>"));
    }
    if starts_complete_open_tag(rest, b"pre") {
        return Some(HtmlBlockEnd::Contains(b"</pre>"));
    }
    if starts_complete_open_tag(rest, b"style") {
        return Some(HtmlBlockEnd::Contains(b"</style>"));
    }
    if starts_complete_open_tag(rest, b"textarea") {
        return Some(HtmlBlockEnd::Contains(b"</textarea>"));
    }
    if rest.starts_with(b"<!--") {
        return Some(HtmlBlockEnd::Contains(b"-->"));
    }
    if rest.starts_with(b"<?") {
        return Some(HtmlBlockEnd::Contains(b"?>"));
    }
    if rest.starts_with(b"<![CDATA[") {
        return Some(HtmlBlockEnd::Contains(b"]]>"));
    }
    if rest.len() > 2 && rest.starts_with(b"<!") && rest[2].is_ascii_alphabetic() {
        return Some(HtmlBlockEnd::Contains(b">"));
    }
    if html_block_tag_name(rest, 1).is_some_and(is_commonmark_block_tag) {
        return Some(HtmlBlockEnd::BlankLine);
    }
    if is_complete_tag_line(rest) {
        return Some(HtmlBlockEnd::BlankLine);
    }

    None
}

fn starts_complete_open_tag(line: &[u8], tag: &[u8]) -> bool {
    html_block_tag_name(line, 1)
        .is_some_and(|name| name.eq_ignore_ascii_case(tag) && !is_before(line, 1, b'/'))
}

fn html_block_tag_name(line: &[u8], index: usize) -> Option<&[u8]> {
    let range = html_block_tag_name_range(line, index)?;

    Some(&line[range])
}

fn html_block_tag_name_range(line: &[u8], mut index: usize) -> Option<std::ops::Range<usize>> {
    if is_before(line, index, b'/') {
        index += 1;
    }

    let start = index;

    if index >= line.len() || !line[index].is_ascii_alphabetic() {
        return None;
    }

    index += 1;

    while index < line.len() && (line[index].is_ascii_alphanumeric() || line[index] == b'-') {
        index += 1;
    }

    if start == index || !is_html_tag_boundary(line, index) {
        None
    } else {
        Some(start..index)
    }
}

fn is_html_tag_boundary(line: &[u8], index: usize) -> bool {
    index == line.len() || matches!(line[index], b' ' | b'\t' | b'\n' | b'\r' | b'/' | b'>')
}

fn is_complete_tag_line(line: &[u8]) -> bool {
    let Some(name_range) = html_block_tag_name_range(line, 1) else {
        return false;
    };
    let mut index = name_range.end;
    let mut quote = None;

    while index < line.len() {
        if quote.is_some_and(|quoted| line[index] == quoted) {
            quote = None;
        } else if quote.is_none() && matches!(line[index], b'\'' | b'"') {
            quote = Some(line[index]);
        } else if quote.is_none() && line[index] == b'>' {
            return line[index + 1..]
                .iter()
                .all(|byte| byte.is_ascii_whitespace());
        }
        index += 1;
    }

    false
}

fn is_commonmark_block_tag(name: &[u8]) -> bool {
    const TAGS: &str = "address article aside base basefont blockquote body caption center col \
        colgroup dd details dialog dir div dl dt fieldset figcaption figure footer form frame \
        frameset h1 h2 h3 h4 h5 h6 head header hr html iframe legend li link main menu menuitem \
        nav noframes ol optgroup option p param search section summary table tbody td tfoot th \
        thead title tr track ul script pre style textarea video source";

    TAGS.split_ascii_whitespace()
        .any(|tag| name.eq_ignore_ascii_case(tag.as_bytes()))
}

fn contains_ignore_ascii_case(haystack: &[u8], needle: &[u8]) -> bool {
    haystack
        .windows(needle.len())
        .any(|window| window.eq_ignore_ascii_case(needle))
}
