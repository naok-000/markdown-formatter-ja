pub(super) fn display_width(character: char) -> usize {
    if character.is_ascii() { 1 } else { 2 }
}

pub(super) fn text_width(text: &str) -> usize {
    text.chars().map(display_width).sum()
}

pub(super) fn starts_with_prohibited_line_start(text: &str) -> bool {
    text.chars().next().is_some_and(is_prohibited_line_start)
}

fn is_prohibited_line_start(character: char) -> bool {
    matches!(
        character,
        '、' | '。' | '，' | '．' | ',' | '.' | ')' | '）' | ']' | '】' | '}' | '」' | '』'
    )
}

pub(super) fn ascii_word_len(text: &str) -> Option<usize> {
    let mut characters = text.char_indices();
    let (_, first) = characters.next()?;

    if !first.is_ascii_alphabetic() {
        return None;
    }

    let mut end = first.len_utf8();

    for (index, character) in characters {
        if is_ascii_word_character(character) {
            end = index + character.len_utf8();
        } else {
            break;
        }
    }

    Some(end)
}

fn is_ascii_word_character(character: char) -> bool {
    character.is_ascii_alphanumeric() || matches!(character, '_' | '-' | '.' | '/')
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn text_width_counts_ascii_as_one_and_non_ascii_as_two() {
        assert_eq!(display_width('a'), 1);
        assert_eq!(display_width('あ'), 2);
        assert_eq!(text_width("aあ。"), 5);
    }

    #[test]
    fn ascii_word_len_accepts_word_like_ascii_tokens() {
        assert_eq!(ascii_word_len("markdownの文章"), Some(8));
        assert_eq!(ascii_word_len("foo_bar foo"), Some(7));
        assert_eq!(ascii_word_len("path/to/file"), Some(12));
        assert_eq!(ascii_word_len("あmarkdown"), None);
        assert_eq!(ascii_word_len("1markdown"), None);
        assert_eq!(ascii_word_len(""), None);
    }

    #[test]
    fn detects_prohibited_line_start_characters() {
        assert!(is_prohibited_line_start('。'));
        assert!(is_prohibited_line_start('）'));
        assert!(starts_with_prohibited_line_start("、続き"));
        assert!(!is_prohibited_line_start('あ'));
        assert!(!starts_with_prohibited_line_start("本文"));
        assert!(!starts_with_prohibited_line_start(""));
    }
}
