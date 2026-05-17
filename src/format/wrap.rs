use std::borrow::Cow;

use comrak::Arena;
use comrak::nodes::{Ast, AstNode, LineColumn, ListType, NodeList, NodeValue};

use super::width::{ascii_word_len, starts_with_prohibited_line_start, text_width};
use super::{FormatOptions, LineBreakMode};

pub(super) fn wrap_document<'a>(
    arena: &'a Arena<'a>,
    root: &'a AstNode<'a>,
    options: FormatOptions,
) {
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
    matches!(&node.data.borrow().value, NodeValue::Paragraph) && !has_table_ancestor(node)
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

#[cfg(test)]
#[path = "wrap_tests.rs"]
mod tests;
