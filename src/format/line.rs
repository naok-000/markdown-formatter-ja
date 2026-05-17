pub(super) fn is_before(bytes: &[u8], index: usize, expected: u8) -> bool {
    bytes.get(index).is_some_and(|byte| *byte == expected)
}

pub(super) fn logical_line_start_index(bytes: &[u8], index: usize) -> usize {
    let mut start = line_start_index(bytes, index);

    loop {
        let marker_start = skip_spaces(bytes, start, index, 3);

        if is_before(bytes, marker_start, b'>') {
            start = skip_optional_space(bytes, marker_start + 1, index);
            continue;
        }

        if let Some(after_list_marker) = unordered_list_marker_end(bytes, marker_start, index) {
            start = after_list_marker;
            continue;
        }

        if let Some(after_list_marker) = ordered_list_marker_end(bytes, marker_start, index) {
            start = after_list_marker;
            continue;
        }

        if let Some(after_task_marker) = task_marker_end(bytes, marker_start, index) {
            start = after_task_marker;
            continue;
        }

        return marker_start;
    }
}

fn line_start_index(bytes: &[u8], index: usize) -> usize {
    bytes[..index]
        .iter()
        .rposition(|byte| *byte == b'\n')
        .map_or(0, |position| position + 1)
}

pub(super) fn skip_spaces(bytes: &[u8], mut index: usize, end: usize, limit: usize) -> usize {
    let mut skipped = 0;

    while index < end && skipped < limit && bytes[index] == b' ' {
        index += 1;
        skipped += 1;
    }

    index
}

fn skip_optional_space(bytes: &[u8], index: usize, end: usize) -> usize {
    if index < end && bytes[index] == b' ' {
        index + 1
    } else {
        index
    }
}

fn unordered_list_marker_end(bytes: &[u8], index: usize, end: usize) -> Option<usize> {
    if index + 1 < end && matches!(bytes[index], b'-' | b'+' | b'*') && bytes[index + 1] == b' ' {
        Some(index + 2)
    } else {
        None
    }
}

fn ordered_list_marker_end(bytes: &[u8], index: usize, end: usize) -> Option<usize> {
    let mut marker_end = index;

    while marker_end < end && bytes[marker_end].is_ascii_digit() && marker_end - index < 9 {
        marker_end += 1;
    }

    if marker_end > index
        && marker_end + 1 < end
        && matches!(bytes[marker_end], b'.' | b')')
        && bytes[marker_end + 1] == b' '
    {
        Some(marker_end + 2)
    } else {
        None
    }
}

fn task_marker_end(bytes: &[u8], index: usize, end: usize) -> Option<usize> {
    if index + 3 < end
        && bytes[index] == b'['
        && matches!(bytes[index + 1], b' ' | b'x' | b'X')
        && bytes[index + 2] == b']'
        && bytes[index + 3] == b' '
    {
        Some(index + 4)
    } else {
        None
    }
}
