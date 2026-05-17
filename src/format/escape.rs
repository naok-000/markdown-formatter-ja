use super::line::{is_before, logical_line_start_index};
use super::literal::backslash_literal_ranges;

pub(super) fn minimize_backslash_escapes(markdown: &str) -> String {
    remove_candidate_backslash_escapes(markdown)
}

fn remove_candidate_backslash_escapes(markdown: &str) -> String {
    let mut output = String::with_capacity(markdown.len());
    let mut start = 0;
    let literal_ranges = backslash_literal_ranges(markdown);
    let mut literal_range_index = 0;
    let bytes = markdown.as_bytes();

    for index in 0..bytes.len().saturating_sub(1) {
        while literal_range_index < literal_ranges.len()
            && literal_ranges[literal_range_index].end <= index
        {
            literal_range_index += 1;
        }

        let is_literal = literal_ranges
            .get(literal_range_index)
            .is_some_and(|range| range.start <= index && index < range.end);

        if bytes[index] == b'\\'
            && bytes[index + 1].is_ascii_punctuation()
            && !is_literal
            && is_removable_escape(markdown, index)
        {
            output.push_str(&markdown[start..index]);
            start = index + 1;
        }
    }

    output.push_str(&markdown[start..]);
    output
}

fn is_removable_escape(markdown: &str, index: usize) -> bool {
    let bytes = markdown.as_bytes();

    match bytes[index + 1] {
        b'_' => is_intraword_escape(bytes, index),
        b'>' | b'#' => !is_line_start_escape(bytes, index),
        b'.' | b')' => !is_ordered_list_marker_escape(bytes, index),
        b'!' => !is_before(bytes, index + 2, b'['),
        b'[' => !starts_link_or_reference(bytes, index + 2),
        b']' => !ends_link_label_escape(bytes, index),
        b'@' => !looks_like_email_autolink(bytes, index),
        _ => false,
    }
}

fn is_intraword_escape(bytes: &[u8], index: usize) -> bool {
    index > 0
        && index + 2 < bytes.len()
        && bytes[index - 1].is_ascii_alphanumeric()
        && bytes[index + 2].is_ascii_alphanumeric()
}

fn starts_link_or_reference(bytes: &[u8], index: usize) -> bool {
    let Some(label_end) = bytes[index..].iter().position(|byte| *byte == b']') else {
        return false;
    };
    let next = index + label_end + 1;

    is_before(bytes, next, b'(') || is_before(bytes, next, b'[')
}

fn ends_link_label_escape(bytes: &[u8], index: usize) -> bool {
    let mut current = index + 2;

    while current < bytes.len() && bytes[current] != b'\n' {
        if bytes[current] == b']'
            && (is_before(bytes, current + 1, b'(') || is_before(bytes, current + 1, b'['))
        {
            return true;
        }

        current += 1;
    }

    false
}

fn looks_like_email_autolink(bytes: &[u8], index: usize) -> bool {
    index > 0
        && index + 2 < bytes.len()
        && bytes[index - 1].is_ascii_alphanumeric()
        && bytes[index + 2].is_ascii_alphanumeric()
}

fn is_line_start_escape(bytes: &[u8], index: usize) -> bool {
    let line_start = logical_line_start_index(bytes, index);

    bytes[line_start..index].iter().all(|byte| *byte == b' ')
}

fn is_ordered_list_marker_escape(bytes: &[u8], index: usize) -> bool {
    let line_start = logical_line_start_index(bytes, index);

    line_start < index && bytes[line_start..index].iter().all(u8::is_ascii_digit)
}
