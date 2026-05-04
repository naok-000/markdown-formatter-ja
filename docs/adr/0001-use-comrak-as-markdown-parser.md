# 0001. Markdown parserとしてcomrakを採用する

## Status

Accepted

## Context

markdown-formatter-ja は，日本語を含むMarkdownのraw textを読みやすく整形するCLIである．
そのため，単なる文字列処理ではなく，Markdownのブロック構造とインライン構造を壊さずに本文を折り返す必要がある．

検討した主な候補は以下である．

- `pulldown-cmark`
  - RustのMarkdown parserとして定番で，高速かつ軽量
  - pull parserであり，Markdownへの再構築やAST編集は自前で組む必要がある
- `comrak`
  - Rust製のCommonMark / GitHub Flavored Markdown parser, renderer, formatter
  - ASTを扱え，CommonMarkとして再出力できる
- `markdown` / `markdown-rs`
  - `micromark` のRust sibling project
  - mdast ASTを扱えるが，Rust Markdown formatter用途での定番感は `pulldown-cmark` や `comrak` より弱い
- `micromark`
  - JavaScript製のCommonMark parser
  - unified / remark ecosystemでは有力だが，Rust製CLIに直接組み込むにはNode runtimeや別プロセス化が必要になる

## Decision

Markdown parserには `comrak` を採用する．

## Consequences

- MarkdownをASTとして扱い，整形対象ブロックと非整形対象ブロックを分類しやすくなる
- CommonMark / GitHub Flavored Markdownに対応しやすくなる
- Markdownへの再出力は `comrak` のCommonMark formatterを利用できる
- 再出力結果は元Markdownの完全な字句表現を保持するものではなく，CommonMarkとして正規化される可能性がある
- 差分最小化が重要になった場合は，`pulldown-cmark` のsource rangeや独自再構築を再検討する
