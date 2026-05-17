use std::ops::Range;

use super::html::html_block_ranges;

pub(super) fn backslash_literal_ranges(markdown: &str) -> Vec<Range<usize>> {
    let mut ranges = front_matter_ranges(markdown);

    ranges.extend(fenced_code_ranges(markdown));
    ranges.extend(html_block_ranges(markdown));
    ranges.extend(inline_code_ranges(markdown));
    ranges.extend(angle_bracket_ranges(markdown));
    ranges.sort_by_key(|range| range.start);

    ranges
}

fn front_matter_ranges(markdown: &str) -> Vec<Range<usize>> {
    let mut line_start = 0;

    for (line_number, line) in markdown.split_inclusive('\n').enumerate() {
        let line_end = line_start + line.len();
        let content = line.trim_end_matches('\n').trim_end_matches('\r');

        if line_number == 0 && content != "---" {
            return Vec::new();
        }

        if line_number > 0 && content == "---" {
            return std::iter::once(0..line_end).collect();
        }

        line_start = line_end;
    }

    Vec::new()
}

fn fenced_code_ranges(markdown: &str) -> Vec<Range<usize>> {
    let mut ranges = Vec::new();
    let mut fence_start = None;
    let mut fence_marker = b'\0';
    let mut line_start = 0;

    for line in markdown.split_inclusive('\n') {
        let line_end = line_start + line.len();
        let content = line.trim_end_matches('\n').trim_end_matches('\r');
        let bytes = content.as_bytes();

        if let Some(start) = fence_start {
            if starts_with_fence(bytes, fence_marker) {
                ranges.push(start..line_end);
                fence_start = None;
            }
        } else if starts_with_fence(bytes, b'`') || starts_with_fence(bytes, b'~') {
            fence_start = Some(line_start);
            fence_marker = bytes[0];
        }

        line_start = line_end;
    }

    if let Some(start) = fence_start {
        ranges.push(start..markdown.len());
    }

    ranges
}

fn starts_with_fence(line: &[u8], marker: u8) -> bool {
    line.len() >= 3 && line[0] == marker && line[1] == marker && line[2] == marker
}

fn inline_code_ranges(markdown: &str) -> Vec<Range<usize>> {
    let mut ranges = Vec::new();
    let bytes = markdown.as_bytes();
    let mut index = 0;

    while index < bytes.len() {
        if bytes[index] == b'`' {
            let run_len = backtick_run_len(bytes, index);
            let search_start = index + run_len;

            if let Some(end) = find_backtick_run(bytes, search_start, run_len) {
                ranges.push(index..end + run_len);
                index = end + run_len;
                continue;
            }
        }

        index += 1;
    }

    ranges
}

fn backtick_run_len(bytes: &[u8], index: usize) -> usize {
    bytes[index..]
        .iter()
        .take_while(|byte| **byte == b'`')
        .count()
}

fn find_backtick_run(bytes: &[u8], start: usize, run_len: usize) -> Option<usize> {
    let mut index = start;

    while index + run_len <= bytes.len() {
        if bytes[index..index + run_len]
            .iter()
            .all(|byte| *byte == b'`')
        {
            return Some(index);
        }

        index += 1;
    }

    None
}

fn angle_bracket_ranges(markdown: &str) -> Vec<Range<usize>> {
    let mut ranges = Vec::new();
    let bytes = markdown.as_bytes();
    let mut index = 0;

    while index < bytes.len() {
        if bytes[index] == b'<'
            && let Some(end) = bytes[index + 1..]
                .iter()
                .position(|byte| *byte == b'>' || *byte == b'\n')
            && bytes[index + 1 + end] == b'>'
        {
            ranges.push(index..index + end + 2);
            index += end + 2;
            continue;
        }

        index += 1;
    }

    ranges
}
