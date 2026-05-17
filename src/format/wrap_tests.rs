use comrak::Arena;
use comrak::nodes::{AstNode, NodeValue};

use super::super::parse_for_test;
use super::*;

#[derive(Debug, Eq, PartialEq)]
enum PieceSummary {
    Text(String),
    Atom(&'static str),
    SoftBreak,
    HardBreak,
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
    pieces
        .into_iter()
        .map(|piece| match piece {
            InlinePiece::Text(text) => PieceSummary::Text(text),
            InlinePiece::Atom(node) => PieceSummary::Atom(atom_name(node)),
            InlinePiece::Break(BreakKind::Soft) => PieceSummary::SoftBreak,
            InlinePiece::Break(BreakKind::Hard) => PieceSummary::HardBreak,
        })
        .collect()
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
fn list_marker_width_counts_bullets_and_ordered_digits() {
    let arena = Arena::new();
    let root = parse_for_test(&arena, "- item\n\n123. item");
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
    let bullet = parse_for_test(&arena, "- 子");
    let bullet_widths = prefix_widths(paragraphs(bullet)[0]);

    assert_eq!(bullet_widths.first, 2);
    assert_eq!(bullet_widths.continuation, 2);

    let arena = Arena::new();
    let ordered = parse_for_test(&arena, "10. 子");
    let ordered_widths = prefix_widths(paragraphs(ordered)[0]);

    assert_eq!(ordered_widths.first, 4);
    assert_eq!(ordered_widths.continuation, 4);

    let arena = Arena::new();
    let blockquote = parse_for_test(&arena, "> 子");
    let blockquote_widths = prefix_widths(paragraphs(blockquote)[0]);

    assert_eq!(blockquote_widths.first, 2);
    assert_eq!(blockquote_widths.continuation, 2);

    let arena = Arena::new();
    let task = parse_for_test(&arena, "- [x] 子");
    let task_widths = prefix_widths(paragraphs(task)[0]);

    assert_eq!(task_widths.first, 6);
    assert_eq!(task_widths.continuation, 2);
}

#[test]
fn wrappable_blocks_are_paragraphs_outside_tables() {
    let arena = Arena::new();
    let root = parse_for_test(&arena, "# 見出し\n\n本文\n\n| 見出し |\n| --- |\n| 値 |");
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
    let root = parse_for_test(&arena, "a\nb");
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
    let root = parse_for_test(&arena, "a  \nb");
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
    let root = parse_for_test(&arena, "`日本語` [日本語](https://example.com/) *強調*");
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
