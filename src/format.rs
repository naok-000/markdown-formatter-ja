use std::borrow::Cow;

use comrak::nodes::{Ast, AstNode, LineColumn, ListType, NodeList, NodeValue};
use comrak::{Arena, Options, format_commonmark, parse_document};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct FormatOptions {
    pub width: usize,
    pub line_break_mode: LineBreakMode,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LineBreakMode {
    Ignore,
    Preserve,
}

pub fn format_markdown(markdown: &str, options: FormatOptions) -> String {
    let comrak_options = comrak_options();
    let arena = Arena::new();
    let root = parse_document(&arena, markdown, &comrak_options);

    wrap_document(&arena, root, options);

    let mut output = String::new();
    format_commonmark(root, &comrak_options, &mut output).unwrap();
    output
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
    use super::{FormatOptions, LineBreakMode, format_markdown};

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
}
