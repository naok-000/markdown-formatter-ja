use comrak::nodes::{ListType, NodeValue};
use comrak::{Arena, Options, format_commonmark, parse_document};

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

#[cfg(test)]
mod tests {
    use super::{BlockKind, BlockRole, format_markdown, parse_blocks, wrap_text};

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
}
