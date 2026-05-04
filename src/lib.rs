use comrak::nodes::{ListType, NodeValue};
use comrak::{Arena, Options, format_commonmark, parse_document};

type MarkdownNode<'a> = comrak::nodes::AstNode<'a>;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BlockKind {
    Paragraph,
    Heading,
    BulletList,
    OrderedList,
    CodeBlock,
    BlockQuote,
    ThematicBreak,
    Table,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Block {
    pub kind: BlockKind,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InlineKind {
    Code,
    Link,
    Image,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Inline {
    pub kind: InlineKind,
    pub literal: String,
    pub destination: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BlockRole {
    FormatTarget,
    Preserve,
}

impl BlockKind {
    pub fn role(self) -> BlockRole {
        match self {
            BlockKind::Paragraph
            | BlockKind::Heading
            | BlockKind::BulletList
            | BlockKind::OrderedList => BlockRole::FormatTarget,
            BlockKind::CodeBlock
            | BlockKind::BlockQuote
            | BlockKind::ThematicBreak
            | BlockKind::Table => BlockRole::Preserve,
        }
    }
}

pub fn parse_blocks(markdown: &str) -> Vec<Block> {
    let arena = Arena::new();
    let options = markdown_options();
    let root = parse_document(&arena, markdown, &options);

    root.children()
        .filter_map(|node| {
            let kind = match &node.data.borrow().value {
                NodeValue::Paragraph => BlockKind::Paragraph,
                NodeValue::Heading(_) => BlockKind::Heading,
                NodeValue::List(list) => match list.list_type {
                    ListType::Bullet => BlockKind::BulletList,
                    ListType::Ordered => BlockKind::OrderedList,
                },
                NodeValue::CodeBlock(_) => BlockKind::CodeBlock,
                NodeValue::BlockQuote => BlockKind::BlockQuote,
                NodeValue::ThematicBreak => BlockKind::ThematicBreak,
                NodeValue::Table(_) => BlockKind::Table,
                _ => return None,
            };

            Some(Block { kind })
        })
        .collect()
}

pub fn format_markdown(markdown: &str) -> String {
    let arena = Arena::new();
    let options = markdown_options();
    let root = parse_document(&arena, markdown, &options);
    let mut output = String::new();

    format_commonmark(root, &options, &mut output).unwrap();

    output
}

pub fn parse_inlines(markdown: &str) -> Vec<Inline> {
    let arena = Arena::new();
    let options = markdown_options();
    let root = parse_document(&arena, markdown, &options);
    let mut inlines = Vec::new();

    collect_inlines(root, &mut inlines);

    inlines
}

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

fn collect_inlines<'a>(node: &'a MarkdownNode<'a>, inlines: &mut Vec<Inline>) {
    if let Some(inline) = inline_from_node(node) {
        inlines.push(inline);
    }

    for child in node.children() {
        collect_inlines(child, inlines);
    }
}

fn inline_from_node<'a>(node: &'a MarkdownNode<'a>) -> Option<Inline> {
    match &node.data.borrow().value {
        NodeValue::Code(code) => Some(Inline {
            kind: InlineKind::Code,
            literal: code.literal.clone(),
            destination: None,
        }),
        NodeValue::Link(link) => Some(Inline {
            kind: InlineKind::Link,
            literal: inline_text(node),
            destination: Some(link.url.clone()),
        }),
        NodeValue::Image(link) => Some(Inline {
            kind: InlineKind::Image,
            literal: inline_text(node),
            destination: Some(link.url.clone()),
        }),
        _ => None,
    }
}

fn inline_text<'a>(node: &'a MarkdownNode<'a>) -> String {
    let mut text = String::new();

    collect_inline_text(node, &mut text);

    text
}

fn collect_inline_text<'a>(node: &'a MarkdownNode<'a>, text: &mut String) {
    match &node.data.borrow().value {
        NodeValue::Text(value) => text.push_str(value),
        NodeValue::Code(code) => text.push_str(&code.literal),
        _ => {}
    }

    for child in node.children() {
        collect_inline_text(child, text);
    }
}

fn markdown_options() -> Options<'static> {
    let mut options = Options::default();
    options.extension.table = true;
    options
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

fn bullet_list_parts(line: &str) -> Option<(&str, &str)> {
    if line.starts_with("- ") {
        Some(line.split_at(2))
    } else {
        None
    }
}

fn ordered_list_parts(line: &str) -> Option<(&str, &str)> {
    let period_index = line.find('.')?;

    if period_index > 0
        && line[..period_index]
            .chars()
            .all(|character| character.is_ascii_digit())
        && line.as_bytes().get(period_index + 1) == Some(&b' ')
    {
        Some(line.split_at(period_index + 2))
    } else {
        None
    }
}

fn wrap_markdown_line(line: &str, width: usize) -> String {
    if let Some((marker, text)) = heading_parts(line) {
        format!("{marker}{}", wrap_markdown_text(text, width))
    } else if let Some((marker, text)) = bullet_list_parts(line) {
        wrap_with_marker(marker, "  ", text, width)
    } else if let Some((marker, text)) = ordered_list_parts(line) {
        wrap_with_marker(marker, &" ".repeat(marker.len()), text, width)
    } else {
        wrap_markdown_text(line, width)
    }
}

fn wrap_with_marker(marker: &str, continuation: &str, text: &str, width: usize) -> String {
    format!(
        "{marker}{}",
        wrap_markdown_text(text, width).replace('\n', &format!("\n{continuation}"))
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
    inline_code_end(text).or_else(|| link_end(text))
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

fn text_width(text: &str) -> usize {
    text.chars().map(display_width).sum()
}

#[cfg(test)]
mod tests {
    use super::{
        BlockKind, BlockRole, Inline, InlineKind, format_markdown, parse_blocks, parse_inlines,
        wrap_markdown, wrap_paragraphs, wrap_text,
    };

    #[test]
    fn splits_markdown_into_top_level_blocks() {
        let markdown = "# Title\n\nHello.\n\n- one\n- two\n\n```rust\nfn main() {}\n```\n";

        let kinds: Vec<_> = parse_blocks(markdown)
            .into_iter()
            .map(|block| block.kind)
            .collect();

        assert_eq!(
            kinds,
            vec![
                BlockKind::Heading,
                BlockKind::Paragraph,
                BlockKind::BulletList,
                BlockKind::CodeBlock,
            ]
        );
    }

    #[test]
    fn distinguishes_ordered_lists_from_bullet_lists() {
        let markdown = "- bullet\n\n1. ordered\n";

        let kinds: Vec<_> = parse_blocks(markdown)
            .into_iter()
            .map(|block| block.kind)
            .collect();

        assert_eq!(kinds, vec![BlockKind::BulletList, BlockKind::OrderedList]);
    }

    #[test]
    fn detects_tables_as_preserved_blocks() {
        let markdown = "| key | value |\n| --- | --- |\n| a | b |\n";

        let blocks = parse_blocks(markdown);

        assert_eq!(
            blocks,
            vec![super::Block {
                kind: BlockKind::Table
            }]
        );
        assert_eq!(blocks[0].kind.role(), BlockRole::Preserve);
    }

    #[test]
    fn detects_preserved_block_kinds() {
        let markdown = "> quote\n\n---\n";

        let kinds: Vec<_> = parse_blocks(markdown)
            .into_iter()
            .map(|block| block.kind)
            .collect();

        assert_eq!(kinds, vec![BlockKind::BlockQuote, BlockKind::ThematicBreak]);
    }

    #[test]
    fn ignores_markdown_syntax_inside_code_fences() {
        let markdown = "```markdown\n# not a heading\n- not a list\n```\n";

        let kinds: Vec<_> = parse_blocks(markdown)
            .into_iter()
            .map(|block| block.kind)
            .collect();

        assert_eq!(kinds, vec![BlockKind::CodeBlock]);
    }

    #[test]
    fn reconstructs_markdown_from_parse_result() {
        let markdown = "# Title\n\nHello.\n\n- one\n- two\n";

        assert_eq!(format_markdown(markdown), markdown);
    }

    #[test]
    fn classifies_blocks_as_format_targets_or_preserved_blocks() {
        assert_eq!(BlockKind::Paragraph.role(), BlockRole::FormatTarget);
        assert_eq!(BlockKind::Heading.role(), BlockRole::FormatTarget);
        assert_eq!(BlockKind::BulletList.role(), BlockRole::FormatTarget);
        assert_eq!(BlockKind::OrderedList.role(), BlockRole::FormatTarget);

        assert_eq!(BlockKind::CodeBlock.role(), BlockRole::Preserve);
        assert_eq!(BlockKind::BlockQuote.role(), BlockRole::Preserve);
        assert_eq!(BlockKind::ThematicBreak.role(), BlockRole::Preserve);
        assert_eq!(BlockKind::Table.role(), BlockRole::Preserve);
    }

    #[test]
    fn detects_inline_code() {
        let markdown = "本文の `code()` を検出する\n";

        assert_eq!(
            parse_inlines(markdown),
            vec![Inline {
                kind: InlineKind::Code,
                literal: "code()".to_string(),
                destination: None,
            }]
        );
    }

    #[test]
    fn detects_links() {
        let markdown = "[Rust](https://www.rust-lang.org/) を検出する\n";

        assert_eq!(
            parse_inlines(markdown),
            vec![Inline {
                kind: InlineKind::Link,
                literal: "Rust".to_string(),
                destination: Some("https://www.rust-lang.org/".to_string()),
            }]
        );
    }

    #[test]
    fn detects_images() {
        let markdown = "![代替テキスト](./image.png)\n";

        assert_eq!(
            parse_inlines(markdown),
            vec![Inline {
                kind: InlineKind::Image,
                literal: "代替テキスト".to_string(),
                destination: Some("./image.png".to_string()),
            }]
        );
    }

    #[test]
    fn reconstructs_markdown_with_inline_syntax() {
        let markdown = "本文の `code()` と [link](https://example.com/) と ![alt](./image.png) と **強調**。\n";

        assert_eq!(format_markdown(markdown), markdown);
    }

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
            "- これは日本\n  語の項目で\n  す"
        );
    }

    #[test]
    fn preserves_ordered_list_marker_and_wraps_item_text() {
        assert_eq!(
            wrap_markdown("1. これは日本語の項目です", 10),
            "1. これは日本\n   語の項目で\n   す"
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
