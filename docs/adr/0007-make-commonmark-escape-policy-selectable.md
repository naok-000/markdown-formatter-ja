# 0007. CommonMark互換の範囲でescape policyを選択可能にする

## Status

Accepted

## Context

markdown-formatter-ja は，Markdownのraw textを読みやすく整形するCLIである．
Markdownでは，同じ意味を表す複数のraw text表現が存在する．
特にbackslash escapeは，Markdown記法として解釈される文字を通常文字として扱うために必要になる一方，過剰に出力されるとraw textの可読性を下げる．

CommonMark 0.31.2では，ASCII punctuation characterはbackslash escapeできる．
escapeされた文字は通常のMarkdown上の意味を失い，通常文字として扱われる．
一方，code blocks，code spans，autolinks，raw HTMLではbackslash escapeは効かない．
そのため，単純に「escapeする」「escapeしない」を全体へ一括適用すると，CommonMarkとしての意味が変わる可能性がある．

現在は `comrak` のCommonMark formatterに出力を任せており，安全側に倒したescapeが出力される場合がある．
これはCommonMarkとして壊れにくいが，raw textの読みやすさを重視する利用者には冗長に見える．

将来的には，escapeを最小化した出力をdefaultにする可能性がある．
その場合でも，既存の保守的な出力を必要とする利用者が明示的に選べる必要がある．

## Decision

出力時のescape方針を `EscapePolicy` として扱い，ユーザが選択できるようにする．

最初に提供するpolicyは以下の2つとする．

- `conservative`
  - 現在のdefaultと同等の方針
  - Markdown記法として解釈され得る文字を安全側にescapeする
- `minimal`
  - CommonMark 0.31.2として意味が変わらない範囲でescapeを減らす
  - 「escapeしない」ではなく，「削っても意味が変わらないescapeだけを削る」方針とする

初期defaultは `conservative` とする．
将来的に `minimal` をdefaultへ変更する可能性は残す．

CLI optionはboolean flagではなく，値を取るoptionにする．

```console
markdown-formatter-ja --escape-policy conservative README.md
markdown-formatter-ja --escape-policy minimal README.md
```

`--minimal-escape` のようなboolean flagは採用しない．
`minimal` がdefaultになった場合にoption名の意味が薄くなり，保守的な出力へ戻す明示手段も別途必要になるためである．

どちらのpolicyでも，出力はCommonMark 0.31.2として妥当であることを必須とする．
`minimal` policyでは，escapeを削った出力を再parseしてもMarkdown構造が変わらないことを確認する．

Entity referenceの正規化やURLのpercent encodingは，backslash escapeとは別の正規化として扱う．
このADRではbackslash escapeの出力方針を対象にする．

参考仕様: [CommonMark 0.31.2 Backslash escapes](https://spec.commonmark.org/0.31.2/#backslash-escapes)

## Consequences

- 利用者はraw textの読みやすさと保守的なCommonMark出力のどちらを優先するか選べる
- 初期defaultを維持しつつ，`--escape-policy minimal` で新しい出力方針を試せる
- 将来 `minimal` をdefaultに変更しても，`--escape-policy conservative` により従来方針を明示できる
- default変更は出力差分を大きく変える可能性があるため，変更時には別ADRまたはrelease noteで判断を残す
- testは，option省略時のdefault挙動と，各policyを明示した挙動を分けて書く
- `minimal` policyでは，escape削除後にCommonMark構造が変わらないことを検証する処理が必要になる
- `minimal` policyの検証は処理量を増やす可能性がある
