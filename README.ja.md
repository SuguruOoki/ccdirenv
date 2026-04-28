# ccdirenv

[![CI](https://github.com/SuguruOoki/ccdirenv/actions/workflows/ci.yml/badge.svg)](https://github.com/SuguruOoki/ccdirenv/actions/workflows/ci.yml)
[![crates.io](https://img.shields.io/crates/v/ccdirenv.svg)](https://crates.io/crates/ccdirenv)
[![License: MIT OR Apache-2.0](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue.svg)](#ライセンス)

**[English](./README.md) | 日本語**

> [ghq](https://github.com/x-motemen/ghq) を前提に、direnv 風にディレクトリ単位で Claude Code のアカウントを自動切り替え。`cd` するだけで適切なアカウントで `claude` が起動します。

`ccdirenv` は ghq のレイアウト（`~/ghq/<host>/<owner>/<repo>`）を起点に、現在のディレクトリに応じて Claude Code のアカウントを透過的に選択します。**ghq の owner ごとにプロファイルを 1 度だけ紐付けておけば、その owner 配下のリポジトリは将来 `ghq get` するものも含めてすべて自動的に正しいアカウントで起動します。** 個人リポは個人アカウント、業務リポは業務アカウント、クライアント案件はクライアントアカウント、というように。

ghq は必須ではありません（ディレクトリ glob やリポ単位のマーカーも使えます）が、本プロジェクトは ghq を既定のメンタルモデルとして設計・最適化されています。

## 仕組み

1. 各アカウントは独立したプロファイルディレクトリ `~/.ccdirenv/profiles/<name>/` に隔離されます。
2. `PATH` の前段に置かれた小さな Rust 製シムが、現在のディレクトリからプロファイルを解決し、`CLAUDE_CONFIG_DIR` を設定した上で本物の `claude` バイナリに処理を引き渡します。
3. 解決の優先順位（先にマッチしたものが勝ち）:
   1. 環境変数 `CCDIRENV_PROFILE`（強制指定）
   2. 現在ディレクトリまたは親ディレクトリにある `.ccdirenv` マーカーファイル
   3. `~/.ccdirenv/config.toml` の `[directories]` glob
   4. **`[ghq.owners]` マッピング（ghq root 配下の `<host>/<owner>` ルックアップ）**
   5. `default_profile`、未指定なら `default`

Claude Code 自身のインストール先・自動更新経路には一切触りません — シムは `PATH` に1つエントリを足し、自分自身を除いた上で見つかった `claude` を exec するだけです。

## インストール

```sh
# crates.io（推奨）
cargo install ccdirenv

# GitHub Releases のビルド済みバイナリでも可（macOS / Linux）:
curl --proto '=https' --tlsv1.2 -LsSf \
  https://github.com/SuguruOoki/ccdirenv/releases/latest/download/ccdirenv-installer.sh | sh

# Homebrew（tap 公開後に利用可能）
brew tap SuguruOoki/tap
brew install ccdirenv
```

[ghq](https://github.com/x-motemen/ghq) は本プロジェクトの前提です。**`ccdirenv init` 実行時に ghq が PATH に無ければ自動インストールします** — Homebrew → `go install` の順で試行し、いずれも使えない場合のみ手動インストール案内を表示します。あとから明示的にチェックしたい場合は `ccdirenv ghq install` を実行してください。

CI などで自動インストールを抑止したい場合は `CCDIRENV_SKIP_GHQ_AUTOINSTALL=1` を設定します。

## セットアップ

```sh
# 1. プロファイルディレクトリを作成し、シムを ~/.ccdirenv/bin/claude にインストール
#    （必要なら ghq もこのタイミングで自動インストールされる）
ccdirenv init

# 2. 表示される指示どおり、シェル rc (~/.zshrc, ~/.bashrc など) に追加:
#    export PATH="$HOME/.ccdirenv/bin:$PATH"

# 3. (任意) 既存の ~/.claude/ ログイン情報を 'default' プロファイルに移行
ccdirenv import default

# 4. 追加プロファイルを作成・ログイン
ccdirenv login work
ccdirenv login personal

# 5. ghq の owner をプロファイルに紐付け
ccdirenv ghq map github.com/your-personal-handle  default
ccdirenv ghq map github.com/your-employer         work
ccdirenv ghq map github.com/a-client-org          client-acme
```

これで完了です。マップ済み owner 配下のすべてのリポジトリは、`cd` した瞬間から正しい Claude Code アカウントで動作します。

## ghq 連携（メインのワークフロー）

```toml
# ~/.ccdirenv/config.toml
default_profile = "default"

[ghq.owners]
"github.com/SuguruOoki" = "default"
"github.com/TheMoshInc" = "mosh"
"github.com/AcmeCorp"   = "work"
```

ghq root は自動検出されます: `$GHQ_ROOT` が設定されていればそれ、なければ `~/ghq`（ghq の既定）。別の場所に置いている場合は `[ghq] root = "/custom/path"` で上書きできます。

CLI:

```sh
ccdirenv ghq list                                # 現在のマッピングと ghq root を表示
ccdirenv ghq map github.com/Acme work            # マッピングの追加・更新
ccdirenv ghq unmap github.com/Acme               # マッピング削除
ccdirenv ghq root /custom/ghq                    # ghq root を上書き
ccdirenv ghq root ""                             # 上書きを解除（$GHQ_ROOT / ~/ghq に戻る）
ccdirenv ghq install                             # ghq が無ければインストールを試行
```

マップ済みリポのサブディレクトリも同じプロファイルに解決されます。`cd ~/ghq/github.com/Acme/widget/src/lib` でも `work` が選ばれます。

### リポ単位のオーバーライド

マップ済み owner の中に、フォークやサンドボックスなど別プロファイルにしたいリポがある場合は2通り:

```sh
# A: 単一リポをマーカーファイルでピン留め（最優先）
cd ~/ghq/github.com/Acme/sandbox && ccdirenv use personal

# B: config.toml に明示 glob を書く（ghq マッピングより優先）
```

```toml
[directories]
"~/ghq/github.com/Acme/sandbox/**" = "personal"
```

## その他のコマンド

```sh
ccdirenv which            # 現在ディレクトリで解決されるプロファイル
ccdirenv list             # プロファイル一覧（ログインメール付き）
ccdirenv use <profile>    # cwd にマーカーで紐付け
ccdirenv unuse            # マーカーを削除
ccdirenv config           # ~/.ccdirenv/config.toml を $EDITOR で開く
ccdirenv doctor           # 診断（PATH, claude 解決, 設定ファイル有無, 等）
```

シムが `PATH` に乗っていれば、どのディレクトリで `claude` を実行しても解決済みプロファイルで起動します。

## 環境変数

| 変数 | 効果 |
|---|---|
| `CCDIRENV_PROFILE=<name>` | マーカー / 設定 / ghq を無視して、この呼び出しで使うプロファイルを強制指定 |
| `CCDIRENV_DISABLE=1` | 解決を完全にスキップ。シムは継承された `CLAUDE_CONFIG_DIR` のまま本物の `claude` を透過的に起動。デバッグ用 |
| `CCDIRENV_DEBUG=1` | exec 前に選択されたプロファイル名・プロファイルディレクトリ・本物の claude パスを stderr に出力 |
| `CCDIRENV_HOME=<path>` | データルートの `~/.ccdirenv/` を別ディレクトリに差し替え（テスト用） |
| `CCDIRENV_SKIP_GHQ_AUTOINSTALL=1` | `ccdirenv init` / `ccdirenv ghq install` での ghq 自動インストールを抑止 |
| `GHQ_ROOT=<path>` | `[ghq] root` 未設定時の ghq root として参照 |

## トラブルシューティング

`ccdirenv doctor` を実行してください。以下を診断します:

- `~/.ccdirenv/bin/claude` にシムがインストールされているか
- `PATH` に `~/.ccdirenv/bin` が含まれているか
- シム以外の `claude` バイナリが `PATH` から到達可能か
- `config.toml` が存在するか

シムを入れたのに `claude` が違うアカウントを拾ってしまう場合、`~/.ccdirenv/bin` が `~/.local/bin`（あるいは Claude Code のインストール先）**より前** に来ているかを確認してください。

ghq 配下のディレクトリで意図せず `default` に解決される場合は、`ccdirenv ghq list` で owner マッピングが登録されていること、ccdirenv が認識する ghq root が想定どおりであることを確認してください。

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
