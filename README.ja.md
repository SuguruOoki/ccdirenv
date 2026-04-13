# ccdirenv

[![CI](https://github.com/SuguruOoki/ccdirenv/actions/workflows/ci.yml/badge.svg)](https://github.com/SuguruOoki/ccdirenv/actions/workflows/ci.yml)
[![crates.io](https://img.shields.io/crates/v/ccdirenv.svg)](https://crates.io/crates/ccdirenv)
[![License: MIT OR Apache-2.0](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue.svg)](#ライセンス)

**[English](./README.md) | 日本語**

> direnv 風にディレクトリ単位で Claude Code のアカウントを自動切り替え。`cd` するだけで適切なアカウントで `claude` が起動します。

`ccdirenv` は、作業中のディレクトリに応じて Claude Code のアカウントを透過的に選択します。切り替えコマンドもシェル小細工も不要 — プロジェクトに `cd` するだけで、そのディレクトリに紐付いたアイデンティティで `claude` が立ち上がります。

## なぜ必要か

個人用・業務用・クライアント用など、複数の Claude Code アカウントを使い分ける現状の選択肢は、毎回ログアウト＆再ログインするか、`claude` を起動する前に明示的な切り替えコマンドを叩くかの二択です。`ccdirenv` はアカウントをディレクトリに紐付け、正しいアカウントを自動で選びます — 環境変数の世界で [direnv](https://direnv.net/) が定着させたのと同じパターンを、Claude Code のアカウントに適用したものです。

## 仕組み

1. 各アカウントは独立したプロファイルディレクトリ `~/.ccdirenv/profiles/<name>/` に隔離されます。
2. `PATH` の前段に置かれた小さな Rust 製シムが、現在のディレクトリからプロファイルを解決し、`CLAUDE_CONFIG_DIR` を設定した上で本物の `claude` バイナリに処理を引き渡します。
3. 解決の優先順位:
   1. 環境変数 `CCDIRENV_PROFILE`（単発の強制指定）
   2. 現在ディレクトリまたは親ディレクトリにある `.ccdirenv` マーカーファイル
   3. `~/.ccdirenv/config.toml` の glob パターン（ファイル内で先にマッチしたものが勝ち）
   4. `config.toml` の `default_profile`、未指定なら `default`

Claude Code 自身のインストール先・自動更新経路には一切触りません — シムは `PATH` に1つエントリを足し、自分自身を除いた上で見つかった `claude` を exec するだけです。

## インストール

```sh
# crates.io（現時点で推奨）
cargo install ccdirenv

# Homebrew（macOS / Linux、tap 公開後に利用可能になります）
brew tap SuguruOoki/tap
brew install ccdirenv
```

## セットアップ

```sh
# 1. プロファイルディレクトリを作成し、シムを ~/.ccdirenv/bin/claude にインストール
ccdirenv init

# 2. 表示される指示どおり、シェル rc (~/.zshrc, ~/.bashrc など) に以下を追加:
#    export PATH="$HOME/.ccdirenv/bin:$PATH"
#    追加後、シェルをリロード

# 3. (任意) 既存の ~/.claude/ ログイン情報を 'default' プロファイルに移行
ccdirenv import default

# 4. 追加プロファイルを作成・ログイン
ccdirenv login work
ccdirenv login personal
```

## 日常の使い方

```sh
# 現在のディレクトリ（と配下）を 'work' プロファイルに紐付け
cd ~/clients/acme && ccdirenv use work

# このディレクトリで解決されるプロファイルを確認
ccdirenv which

# 紐付けを解除
ccdirenv unuse

# 全プロファイルとログインメールアドレスを一覧
ccdirenv list

# 何かおかしい場合の診断
ccdirenv doctor
```

一度ディレクトリを紐付けておけば、その配下で `claude` を実行すると自動的に正しいアカウントで起動します。紐付けのないディレクトリでは、設定済みプロファイルまたは `default` が使われます。

## 中央設定ファイルによる glob パターン

`.ccdirenv` マーカーをあちこちに置く代わりに、`~/.ccdirenv/config.toml` で glob パターンをまとめて宣言できます:

```toml
default_profile = "personal"

[directories]
"~/clients/acme/**"   = "acme"
"~/clients/globex/**" = "globex"
"~/work/**"           = "work"
"~/oss/**"            = "personal"
```

ディレクトリ直下の `.ccdirenv` マーカーは中央設定より常に優先されます。中央設定のパターンはファイルに書かれた順で評価され、最初にマッチしたものが勝ちます。

どのディレクトリからでも `ccdirenv config` で編集できます。

## 環境変数

| 変数 | 効果 |
|---|---|
| `CCDIRENV_PROFILE=<name>` | マーカー・設定を無視して、この呼び出しで使うプロファイルを強制指定 |
| `CCDIRENV_DISABLE=1` | 解決を完全にスキップ。シムは継承された `CLAUDE_CONFIG_DIR` のまま本物の `claude` を透過的に起動。デバッグ用途 |
| `CCDIRENV_DEBUG=1` | exec 前に選択されたプロファイル名・プロファイルディレクトリ・本物の claude パスを stderr に出力 |
| `CCDIRENV_HOME=<path>` | データルートの `~/.ccdirenv/` を別ディレクトリに差し替え。テスト用途 |

## トラブルシューティング

`ccdirenv doctor` を実行してください。以下を診断します:

- `~/.ccdirenv/bin/claude` にシムがインストールされているか
- `PATH` に `~/.ccdirenv/bin` が含まれているか
- シム以外の `claude` バイナリが `PATH` から到達可能か
- `config.toml` が存在するか

シムを入れたのに `claude` が違うアカウントを拾ってしまう場合、`~/.ccdirenv/bin` が `~/.local/bin`（あるいは Claude Code のインストール先）**より前** に来ているかを確認してください。

## アンインストール

```sh
cargo uninstall ccdirenv
rm -rf ~/.ccdirenv/bin
# シェル rc から PATH の1行を削除
# ~/.ccdirenv/profiles/ 配下のプロファイルはそのまま残るので、不要なら手動で削除
```

## ライセンス

以下のいずれかのライセンスのもとに提供されます。利用者がどちらかを選択できます。

- Apache License, Version 2.0 ([LICENSE-APACHE](./LICENSE-APACHE) または <https://www.apache.org/licenses/LICENSE-2.0>)
- MIT License ([LICENSE-MIT](./LICENSE-MIT) または <https://opensource.org/licenses/MIT>)

### コントリビューション

特に明示的な表明がない限り、あなたが Apache-2.0 ライセンスの定義に従って本プロジェクトに意図的に提出した貢献物は、上記のデュアルライセンスの下で提供されるものとします。追加の条項・条件は一切付きません。
