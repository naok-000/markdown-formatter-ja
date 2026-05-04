# 0005. CI/CDと配布方法をNixとCargo install --gitに絞る

## Status

Accepted

## Context

markdown-formatter-ja は，CLIとして簡単に試せて，継続的に壊れていないことを確認できる状態にしたい．

配布方法は，まず crates.io への公開ではなく，GitHub repository から直接利用できる形を優先する．
また，このprojectにはすでに `flake.nix` があるため，Nixを開発環境，CI，配布経路に使うと，Rust toolchainやbuild手順を揃えやすい．

初期対応platformは，必要最小限として Apple Silicon Mac を重視する．
追加で Linux amd64 もCI対象に含める．
Intel Mac は初期対応から外す．

## Decision

CIはGitHub Actionsで実行する．

通常のpushとpull requestでは，以下を確認する．

- `nix flake check`
- `nix build`
- `nix develop -c cargo fmt --check`
- `nix develop -c cargo clippy -- -D warnings`
- `nix develop -c cargo test`
- `nix develop -c cargo build --release`
- `nix develop -c cargo install --path . --locked`

CI対象platformは以下とする．

- Linux amd64
- Darwin aarch64

Nix flakeでは以下を公開する．

- `packages.default`
- `apps.default`
- `devShells.default`

これにより，次の利用方法を正式にサポートする．

- `nix run github:naok-000/markdown-formatter-ja`
- `cargo install --git https://github.com/naok-000/markdown-formatter-ja`

Rust toolchainは，`rust-toolchain.toml` ではなく，Nix flakeで管理する．
Nixを使わずに `cargo install --git` する利用者には，Rust stableが必要であることをREADMEに記載する．

ReleaseはGitHub Release機能を使う．
シンプルさを優先し，`v*` tag pushを起点にGitHub ActionsでReleaseを作成する．
Release用バイナリの添付は必須にしない．
必要になった時点で追加する．

## Consequences

- 開発環境，CI，Nix配布で同じRust toolchainを使いやすい
- `nix run github:naok-000/markdown-formatter-ja` でinstallなしに実行できる
- `cargo install --git` により，crates.io公開前でもCargo経由で配布できる
- crates.io公開やRelease assetの配布は，必要になるまで実装しない
- Intel Mac対応は初期CI対象に含めない
