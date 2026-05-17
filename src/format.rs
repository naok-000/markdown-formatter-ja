mod escape;
mod html;
mod line;
mod literal;
mod width;
mod wrap;

use comrak::{Arena, Options, format_commonmark, parse_document};

use self::escape::minimize_backslash_escapes;
use self::wrap::wrap_document;

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

#[cfg(test)]
pub(super) fn parse_for_test<'a>(
    arena: &'a Arena<'a>,
    markdown: &str,
) -> &'a comrak::nodes::AstNode<'a> {
    parse_document(arena, markdown, &comrak_options())
}
