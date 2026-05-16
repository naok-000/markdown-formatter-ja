# markdown-formatter-ja

Markdown formatter for Japanese documents.

## Usage

By default, existing line breaks inside paragraphs and list items are ignored before wrapping.

Format Markdown from stdin:

```sh
nix run github:naok-000/markdown-formatter-ja -- --width 80 < input.md
```

Format a file and print the result to stdout:

```sh
nix run github:naok-000/markdown-formatter-ja -- --width 80 input.md
```

Overwrite a file in place:

```sh
nix run github:naok-000/markdown-formatter-ja -- --width 80 --write input.md
```

Preserve existing line breaks:

```sh
nix run github:naok-000/markdown-formatter-ja -- --width 80 --preserve-line-breaks < input.md
```

Use fewer CommonMark-safe backslash escapes:

```sh
nix run github:naok-000/markdown-formatter-ja -- --escape-policy minimal < input.md
```

## Install with Cargo

```sh
cargo install --git https://github.com/naok-000/markdown-formatter-ja --locked
```

When not using Nix, Rust stable is required.

## Development

```sh
nix develop
cargo test
```

## License

Apache-2.0
