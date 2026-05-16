use std::borrow::Cow;
use std::ops::Range;

use comrak::nodes::{Ast, AstNode, LineColumn, ListType, NodeList, NodeValue};
use comrak::{Arena, Options, format_commonmark, parse_document};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct FormatOptions {
    pub width: usize,
    pub line_break_mode: LineBreakMode,
    pub escape_policy: EscapePolicy,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LineBreakMode {
    Ignore,
    Preserve,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum EscapePolicy {
    Conservative,
    Minimal,
}

pub fn format_markdown(markdown: &str, options: FormatOptions) -> String {
    let comrak_options = comrak_options();
    let arena = Arena::new();
    let root = parse_document(&arena, markdown, &comrak_options);

    wrap_document(&arena, root, options);

    let mut output = String::new();
    format_commonmark(root, &comrak_options, &mut output).unwrap();

    match options.escape_policy {
        EscapePolicy::Conservative => output,
        EscapePolicy::Minimal => minimize_backslash_escapes(&output),
    }
}

fn comrak_options() -> Options<'static> {
    let mut options = Options::default();

    options.extension.front_matter_delimiter = Some("---".to_owned());
    options.extension.table = true;
    options.extension.tasklist = true;
    options.extension.strikethrough = true;
    options.extension.autolink = true;
    options.extension.cjk_friendly_emphasis = true;
    options.render.prefer_fenced = true;
    options.render.width = 0;

    options
}

fn minimize_backslash_escapes(markdown: &str) -> String {
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
        b']' => true,
        b'@' => true,
        _ => false,
    }
}

fn is_intraword_escape(bytes: &[u8], index: usize) -> bool {
    index > 0
        && index + 2 < bytes.len()
        && bytes[index - 1].is_ascii_alphanumeric()
        && bytes[index + 2].is_ascii_alphanumeric()
}

fn is_before(bytes: &[u8], index: usize, expected: u8) -> bool {
    bytes.get(index).is_some_and(|byte| *byte == expected)
}

fn starts_link_or_reference(bytes: &[u8], index: usize) -> bool {
    let Some(label_end) = bytes[index..].iter().position(|byte| *byte == b']') else {
        return false;
    };
    let next = index + label_end + 1;

    is_before(bytes, next, b'(') || is_before(bytes, next, b'[')
}

fn backslash_literal_ranges(markdown: &str) -> Vec<Range<usize>> {
    let mut ranges = fenced_code_ranges(markdown);

    ranges.extend(inline_code_ranges(markdown));
    ranges.extend(angle_bracket_ranges(markdown));
    ranges.sort_by_key(|range| range.start);

    ranges
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

fn is_line_start_escape(bytes: &[u8], index: usize) -> bool {
    let line_start = line_start_index(bytes, index);

    bytes[line_start..index].iter().all(|byte| *byte == b' ')
}

fn is_ordered_list_marker_escape(bytes: &[u8], index: usize) -> bool {
    let line_start = line_start_index(bytes, index);

    line_start < index && bytes[line_start..index].iter().all(u8::is_ascii_digit)
}

fn line_start_index(bytes: &[u8], index: usize) -> usize {
    bytes[..index]
        .iter()
        .rposition(|byte| *byte == b'\n')
        .map_or(0, |position| position + 1)
}

fn wrap_document<'a>(arena: &'a Arena<'a>, root: &'a AstNode<'a>, options: FormatOptions) {
    let blocks = root
        .descendants()
        .filter(|node| is_wrappable_block(node))
        .collect::<Vec<_>>();

    for block in blocks {
        let widths = LineWidths::new(options.width, prefix_widths(block));
        let pieces = collect_inline_pieces(block, options.line_break_mode);

        replace_inline_children(arena, block, pieces, widths);
    }
}

fn is_wrappable_block<'a>(node: &'a AstNode<'a>) -> bool {
    let is_text_block = matches!(&node.data.borrow().value, NodeValue::Paragraph);

    is_text_block && !has_table_ancestor(node)
}

fn has_table_ancestor<'a>(node: &'a AstNode<'a>) -> bool {
    node.ancestors().skip(1).any(|ancestor| {
        matches!(
            &ancestor.data.borrow().value,
            NodeValue::Table(_) | NodeValue::TableRow(_) | NodeValue::TableCell
        )
    })
}

#[derive(Clone, Copy)]
struct PrefixWidths {
    first: usize,
    continuation: usize,
}

fn prefix_widths<'a>(node: &'a AstNode<'a>) -> PrefixWidths {
    let mut first = 0;
    let mut continuation = 0;
    for ancestor in node.ancestors().skip(1) {
        match &ancestor.data.borrow().value {
            NodeValue::Item(list) => {
                let marker_width = list_marker_width(list);

                first += marker_width;
                continuation += marker_width;
            }
            NodeValue::TaskItem(_) => {
                let marker_width = task_item_marker_width(ancestor);

                first += marker_width + 4;
                continuation += marker_width;
            }
            NodeValue::BlockQuote => {
                first += 2;
                continuation += 2;
            }
            _ => {}
        }
    }

    PrefixWidths {
        first,
        continuation,
    }
}

fn task_item_marker_width<'a>(node: &'a AstNode<'a>) -> usize {
    node.parent()
        .and_then(|parent| match &parent.data.borrow().value {
            NodeValue::List(list) => Some(list_marker_width(list)),
            _ => None,
        })
        .unwrap_or(0)
}

fn list_marker_width(list: &NodeList) -> usize {
    match list.list_type {
        ListType::Bullet => 2,
        ListType::Ordered => decimal_digits(list.start) + 2,
    }
}

fn decimal_digits(number: usize) -> usize {
    number.to_string().len()
}

#[derive(Clone, Copy)]
struct LineWidths {
    first: usize,
    continuation: usize,
}

impl LineWidths {
    fn new(width: usize, prefixes: PrefixWidths) -> Self {
        Self {
            first: width.saturating_sub(prefixes.first).max(1),
            continuation: width.saturating_sub(prefixes.continuation).max(1),
        }
    }
}

#[derive(Clone, Copy)]
enum BreakKind {
    Soft,
    Hard,
}

enum InlinePiece<'a> {
    Text(String),
    Atom(&'a AstNode<'a>),
    Break(BreakKind),
}

fn collect_inline_pieces<'a>(node: &'a AstNode<'a>, mode: LineBreakMode) -> Vec<InlinePiece<'a>> {
    let mut pieces = Vec::new();

    for child in node.children() {
        match &child.data.borrow().value {
            NodeValue::Text(text) => push_text_pieces(text, &mut pieces),
            NodeValue::SoftBreak if mode == LineBreakMode::Preserve => {
                pieces.push(InlinePiece::Break(BreakKind::Soft));
            }
            NodeValue::SoftBreak => {}
            NodeValue::LineBreak => pieces.push(InlinePiece::Break(BreakKind::Hard)),
            _ => pieces.push(InlinePiece::Atom(child)),
        }
    }

    pieces
}

fn push_text_pieces<'a>(text: &str, pieces: &mut Vec<InlinePiece<'a>>) {
    let mut rest = text;

    while !rest.is_empty() {
        if let Some(token_len) = ascii_word_len(rest) {
            pieces.push(InlinePiece::Text(rest[..token_len].to_string()));
            rest = &rest[token_len..];
        } else {
            let character = rest.chars().next().unwrap();
            pieces.push(InlinePiece::Text(character.to_string()));
            rest = &rest[character.len_utf8()..];
        }
    }
}

fn replace_inline_children<'a>(
    arena: &'a Arena<'a>,
    node: &'a AstNode<'a>,
    pieces: Vec<InlinePiece<'a>>,
    widths: LineWidths,
) {
    for child in node.children().collect::<Vec<_>>() {
        child.detach();
    }

    let mut current_width = 0;
    let mut current_line_width = widths.first;

    for piece in pieces {
        match piece {
            InlinePiece::Text(text) => {
                if text == " " && current_width + 1 > current_line_width {
                    append_break(arena, node, BreakKind::Soft);
                    current_width = 0;
                    current_line_width = widths.continuation;
                    continue;
                }

                let piece_width = text_width(&text);

                if current_width > 0
                    && current_width + piece_width > current_line_width
                    && !starts_with_prohibited_line_start(&text)
                {
                    append_break(arena, node, BreakKind::Soft);
                    current_width = 0;
                    current_line_width = widths.continuation;
                }

                append_text(arena, node, text);
                current_width += piece_width;
            }
            InlinePiece::Atom(atom) => {
                let piece_width = inline_markdown_width(atom);

                if current_width > 0 && current_width + piece_width > current_line_width {
                    append_break(arena, node, BreakKind::Soft);
                    current_width = 0;
                    current_line_width = widths.continuation;
                }

                node.append(atom);
                current_width += piece_width;
            }
            InlinePiece::Break(kind) => {
                append_break(arena, node, kind);
                current_width = 0;
                current_line_width = widths.continuation;
            }
        }
    }
}

fn append_text<'a>(arena: &'a Arena<'a>, parent: &'a AstNode<'a>, text: String) {
    parent.append(new_node(arena, NodeValue::Text(Cow::Owned(text))));
}

fn append_break<'a>(arena: &'a Arena<'a>, parent: &'a AstNode<'a>, kind: BreakKind) {
    let value = match kind {
        BreakKind::Soft => NodeValue::SoftBreak,
        BreakKind::Hard => NodeValue::LineBreak,
    };

    parent.append(new_node(arena, value));
}

fn new_node<'a>(arena: &'a Arena<'a>, value: NodeValue) -> &'a AstNode<'a> {
    arena.alloc(Ast::new(value, LineColumn::default()).into())
}

fn inline_markdown_width<'a>(node: &'a AstNode<'a>) -> usize {
    match &node.data.borrow().value {
        NodeValue::Text(text) => text_width(text),
        NodeValue::Code(code) => code.num_backticks * 2 + text_width(&code.literal),
        NodeValue::HtmlInline(html) | NodeValue::Raw(html) => text_width(html),
        NodeValue::LineBreak | NodeValue::SoftBreak => 0,
        NodeValue::TaskItem(_) => 4,
        NodeValue::Link(link) => {
            let title_width = if link.title.is_empty() {
                0
            } else {
                text_width(&link.title) + 3
            };

            inline_children_width(node) + text_width(&link.url) + title_width + 4
        }
        NodeValue::Image(link) => {
            let title_width = if link.title.is_empty() {
                0
            } else {
                text_width(&link.title) + 3
            };

            inline_children_width(node) + text_width(&link.url) + title_width + 5
        }
        NodeValue::Emph => inline_children_width(node) + 2,
        NodeValue::Strong => inline_children_width(node) + 4,
        NodeValue::Strikethrough => inline_children_width(node) + 4,
        NodeValue::Superscript | NodeValue::Subscript => inline_children_width(node) + 2,
        _ => inline_children_width(node),
    }
}

fn inline_children_width<'a>(node: &'a AstNode<'a>) -> usize {
    node.children().map(inline_markdown_width).sum()
}

fn display_width(character: char) -> usize {
    if character.is_ascii() { 1 } else { 2 }
}

fn starts_with_prohibited_line_start(text: &str) -> bool {
    text.chars().next().is_some_and(is_prohibited_line_start)
}

fn is_prohibited_line_start(character: char) -> bool {
    matches!(
        character,
        '、' | '。' | '，' | '．' | ',' | '.' | ')' | '）' | ']' | '】' | '}' | '」' | '』'
    )
}

fn ascii_word_len(text: &str) -> Option<usize> {
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

fn text_width(text: &str) -> usize {
    text.chars().map(display_width).sum()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, Eq, PartialEq)]
    enum PieceSummary {
        Text(String),
        Atom(&'static str),
        SoftBreak,
        HardBreak,
    }

    fn parse<'a>(arena: &'a Arena<'a>, markdown: &str) -> &'a AstNode<'a> {
        parse_document(arena, markdown, &comrak_options())
    }

    fn first_node<'a>(
        root: &'a AstNode<'a>,
        predicate: impl Fn(&AstNode<'a>) -> bool,
    ) -> &'a AstNode<'a> {
        root.descendants().find(|node| predicate(node)).unwrap()
    }

    fn paragraphs<'a>(root: &'a AstNode<'a>) -> Vec<&'a AstNode<'a>> {
        root.descendants()
            .filter(|node| matches!(&node.data.borrow().value, NodeValue::Paragraph))
            .collect()
    }

    fn summarize_pieces(pieces: Vec<InlinePiece<'_>>) -> Vec<PieceSummary> {
        pieces.into_iter().map(summarize_piece).collect()
    }

    fn summarize_piece(piece: InlinePiece<'_>) -> PieceSummary {
        match piece {
            InlinePiece::Text(text) => PieceSummary::Text(text),
            InlinePiece::Atom(node) => PieceSummary::Atom(atom_name(node)),
            InlinePiece::Break(BreakKind::Soft) => PieceSummary::SoftBreak,
            InlinePiece::Break(BreakKind::Hard) => PieceSummary::HardBreak,
        }
    }

    fn summarize_children<'a>(node: &'a AstNode<'a>) -> Vec<PieceSummary> {
        node.children()
            .map(|child| match &child.data.borrow().value {
                NodeValue::Text(text) => PieceSummary::Text(text.to_string()),
                NodeValue::SoftBreak => PieceSummary::SoftBreak,
                NodeValue::LineBreak => PieceSummary::HardBreak,
                _ => PieceSummary::Atom(atom_name(child)),
            })
            .collect()
    }

    fn atom_name<'a>(node: &'a AstNode<'a>) -> &'static str {
        match &node.data.borrow().value {
            NodeValue::Code(_) => "code",
            NodeValue::Emph => "emph",
            NodeValue::Link(_) => "link",
            NodeValue::Image(_) => "image",
            NodeValue::Strong => "strong",
            _ => "atom",
        }
    }

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
    fn push_text_pieces_splits_japanese_by_character_and_keeps_ascii_words() {
        let mut pieces = Vec::new();

        push_text_pieces("あmarkdown。", &mut pieces);

        assert_eq!(
            summarize_pieces(pieces),
            vec![
                PieceSummary::Text("あ".to_owned()),
                PieceSummary::Text("markdown".to_owned()),
                PieceSummary::Text("。".to_owned())
            ]
        );
    }

    #[test]
    fn line_widths_subtract_prefixes_and_keep_one_column() {
        let widths = LineWidths::new(
            10,
            PrefixWidths {
                first: 4,
                continuation: 2,
            },
        );

        assert_eq!(widths.first, 6);
        assert_eq!(widths.continuation, 8);

        let narrow_widths = LineWidths::new(
            1,
            PrefixWidths {
                first: 4,
                continuation: 2,
            },
        );

        assert_eq!(narrow_widths.first, 1);
        assert_eq!(narrow_widths.continuation, 1);
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

    #[test]
    fn list_marker_width_counts_bullets_and_ordered_digits() {
        let arena = Arena::new();
        let root = parse(&arena, "- item\n\n123. item");
        let lists = root
            .descendants()
            .filter(|node| matches!(&node.data.borrow().value, NodeValue::List(_)))
            .collect::<Vec<_>>();

        let NodeValue::List(bullet_list) = &lists[0].data.borrow().value else {
            unreachable!();
        };
        assert_eq!(list_marker_width(bullet_list), 2);

        let NodeValue::List(ordered_list) = &lists[1].data.borrow().value else {
            unreachable!();
        };
        assert_eq!(list_marker_width(ordered_list), 5);
    }

    #[test]
    fn prefix_widths_count_list_blockquote_and_task_markers() {
        let arena = Arena::new();
        let bullet = parse(&arena, "- 子");
        let bullet_widths = prefix_widths(paragraphs(bullet)[0]);

        assert_eq!(bullet_widths.first, 2);
        assert_eq!(bullet_widths.continuation, 2);

        let arena = Arena::new();
        let ordered = parse(&arena, "10. 子");
        let ordered_widths = prefix_widths(paragraphs(ordered)[0]);

        assert_eq!(ordered_widths.first, 4);
        assert_eq!(ordered_widths.continuation, 4);

        let arena = Arena::new();
        let blockquote = parse(&arena, "> 子");
        let blockquote_widths = prefix_widths(paragraphs(blockquote)[0]);

        assert_eq!(blockquote_widths.first, 2);
        assert_eq!(blockquote_widths.continuation, 2);

        let arena = Arena::new();
        let task = parse(&arena, "- [x] 子");
        let task_widths = prefix_widths(paragraphs(task)[0]);

        assert_eq!(task_widths.first, 6);
        assert_eq!(task_widths.continuation, 2);
    }

    #[test]
    fn wrappable_blocks_are_paragraphs_outside_tables() {
        let arena = Arena::new();
        let root = parse(&arena, "# 見出し\n\n本文\n\n| 見出し |\n| --- |\n| 値 |");
        let paragraph = paragraphs(root)[0];
        let heading = first_node(root, |node| {
            matches!(&node.data.borrow().value, NodeValue::Heading(_))
        });
        let table_text = first_node(
            root,
            |node| matches!(&node.data.borrow().value, NodeValue::Text(text) if text == "値"),
        );

        assert!(is_wrappable_block(paragraph));
        assert!(!is_wrappable_block(heading));
        assert!(has_table_ancestor(table_text));
    }

    #[test]
    fn collect_inline_pieces_respects_line_break_mode() {
        let arena = Arena::new();
        let root = parse(&arena, "a\nb");
        let paragraph = paragraphs(root)[0];

        assert_eq!(
            summarize_pieces(collect_inline_pieces(paragraph, LineBreakMode::Ignore)),
            vec![
                PieceSummary::Text("a".to_owned()),
                PieceSummary::Text("b".to_owned())
            ]
        );
        assert_eq!(
            summarize_pieces(collect_inline_pieces(paragraph, LineBreakMode::Preserve)),
            vec![
                PieceSummary::Text("a".to_owned()),
                PieceSummary::SoftBreak,
                PieceSummary::Text("b".to_owned())
            ]
        );

        let arena = Arena::new();
        let root = parse(&arena, "a  \nb");
        let paragraph = paragraphs(root)[0];

        assert_eq!(
            summarize_pieces(collect_inline_pieces(paragraph, LineBreakMode::Ignore)),
            vec![
                PieceSummary::Text("a".to_owned()),
                PieceSummary::HardBreak,
                PieceSummary::Text("b".to_owned())
            ]
        );
    }

    #[test]
    fn inline_markdown_width_counts_inline_markup() {
        let arena = Arena::new();
        let root = parse(&arena, "`日本語` [日本語](https://example.com/) *強調*");
        let code = first_node(root, |node| {
            matches!(&node.data.borrow().value, NodeValue::Code(_))
        });
        let link = first_node(root, |node| {
            matches!(&node.data.borrow().value, NodeValue::Link(_))
        });
        let emphasis = first_node(root, |node| {
            matches!(&node.data.borrow().value, NodeValue::Emph)
        });

        assert_eq!(inline_markdown_width(code), 8);
        assert_eq!(inline_markdown_width(link), 30);
        assert_eq!(inline_markdown_width(emphasis), 6);
    }

    #[test]
    fn replace_inline_children_wraps_text_and_keeps_prohibited_starts_attached() {
        let arena = Arena::new();
        let paragraph = new_node(&arena, NodeValue::Paragraph);

        replace_inline_children(
            &arena,
            paragraph,
            vec![
                InlinePiece::Text("あ".to_owned()),
                InlinePiece::Text("あ".to_owned()),
                InlinePiece::Text("あ".to_owned()),
            ],
            LineWidths {
                first: 4,
                continuation: 4,
            },
        );

        assert_eq!(
            summarize_children(paragraph),
            vec![
                PieceSummary::Text("あ".to_owned()),
                PieceSummary::Text("あ".to_owned()),
                PieceSummary::SoftBreak,
                PieceSummary::Text("あ".to_owned())
            ]
        );

        let arena = Arena::new();
        let paragraph = new_node(&arena, NodeValue::Paragraph);

        replace_inline_children(
            &arena,
            paragraph,
            vec![
                InlinePiece::Text("あ".to_owned()),
                InlinePiece::Text("あ".to_owned()),
                InlinePiece::Text("。".to_owned()),
            ],
            LineWidths {
                first: 4,
                continuation: 4,
            },
        );

        assert_eq!(
            summarize_children(paragraph),
            vec![
                PieceSummary::Text("あ".to_owned()),
                PieceSummary::Text("あ".to_owned()),
                PieceSummary::Text("。".to_owned())
            ]
        );
    }
}
