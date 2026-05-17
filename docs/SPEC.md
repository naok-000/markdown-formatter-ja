# SPEC.md

## 目的

日本語を含むMarkdownのraw textを読みやすく整形するCLIを作る．

## 解決したい課題

既存のMarkdown formatterは，日本語文書に対して読みやすい位置で改行しづらい．
日本語は単語間にspaceがないため，空白位置だけを基準にすると改行位置が偏り，raw textとして読みづらくなる．

## 方針

- Markdownとして正しいfileが入力されることを前提にする
- raw textで読んだときの読みやすさを優先する
- 実装詳細や判断理由はADRに残す

## 要件

- 1行の長さを指定できる
- 通常段落とリスト項目では，既存の改行位置を無視して指定幅で改行する
- option指定により，既存の改行位置を維持して指定幅で改行できる
- file先頭のmetadataは改行を加えず維持する
- CommonMarkとして意味が変わらない範囲で，出力時のbackslash escape方針を選択できる
- 半角文字は幅1，全角文字は幅2として扱う
- Markdown記法の行頭マーカーも行幅に含める
- 句読点の前や閉カッコの前では改行しない
- 英単語の途中では改行しない
- CLIとして使用できる
- 標準入力からMarkdownを読み，整形結果を標準出力へ書ける
- file pathを指定した場合，通常時は整形結果を標準出力へ書ける
- option指定により，指定したMarkdown fileを直接書き換えられる
- `--help` で使い方とoption説明を表示できる
- NixとCargo install --gitで利用できる

## Optional

- 文節を意識した改行
- conform.nvim からの利用

## 関連ADR

- [0002. Markdownの行頭マーカーと英単語を折り返し単位として扱う](./adr/0002-wrap-markdown-prefixes-and-english-words.md)
- [0004. CLIのファイル指定時も整形結果を標準出力する](./adr/0004-print-formatted-output-for-file-input.md)
- [0005. CI/CDと配布方法をNixとCargo install --gitに絞る](./adr/0005-ci-release-and-distribution.md)
- [0006. SPEC.md, TODO.md, ADRの役割を分ける](./adr/0006-document-roles-for-spec-todo-and-adr.md)
- [0007. CommonMark互換の範囲でescape policyを選択可能にする](./adr/0007-make-commonmark-escape-policy-selectable.md)
