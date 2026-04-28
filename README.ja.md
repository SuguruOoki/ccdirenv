# ccdirenv

[![CI](https://github.com/SuguruOoki/ccdirenv/actions/workflows/ci.yml/badge.svg)](https://github.com/SuguruOoki/ccdirenv/actions/workflows/ci.yml)
[![crates.io](https://img.shields.io/crates/v/ccdirenv.svg)](https://crates.io/crates/ccdirenv)
[![License: MIT OR Apache-2.0](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue.svg)](#ライセンス)

**[English](./README.md) | 日本語**

> direnv 風にディレクトリ単位で Claude Code のアカウントを自動切り替え。`cd` するだけで適切なアカウントで `claude` が起動します。[ghq](https://github.com/x-motemen/ghq) でも素の git でも、両方併用でも OK。

`ccdirenv` は現在のディレクトリに応じて Claude Code のアカウントを透過的に選択します。**GitHub の owner（ユーザー / Org）単位で 1 度だけプロファイルを紐付けておけば、その owner 配下のリポジトリは現在も将来 clone するものも含めてすべて正しいアカウントで起動します。** 個人リポは個人アカウント、業務リポは業務アカウント、クライアント案件はクライアントアカウント、というように。

## 2 つの検出方式

インストール時に好きな方式を選べます:

- **git モード（既定）** — git remote を持つ任意のリポジトリで動作。`.git/config` を直接読み（`git` コマンドの subprocess は使用しない）、worktree / submodule も自動 follow して `<host>/<owner>` を抽出。
- **ghq モード** — 最速。`~/ghq/<host>/<owner>/<repo>` というパス構造から `<host>/<owner>` を取り出すだけ。すでに ghq を使っているなら推奨。
- **both** — 一方が外れたらもう一方を試す。ghq 配下のリポと外のリポが混在する人向け。
- **off** — 自動検出なし。`[directories]` glob と `.ccdirenv` マーカーのみ。

どのモードでも同じ `[owners]` マップが使われます。

## 仕組み

1. 各アカウントは独立したプロファイルディレクトリ `~/.ccdirenv/profiles/<name>/` に隔離されます。
2. `PATH` の前段に置かれた小さな Rust 製シムが、現在のディレクトリからプロファイルを解決し、`CLAUDE_CONFIG_DIR` を設定した上で本物の `claude` バイナリに処理を引き渡します。
3. 解決の優先順位（先にマッチしたものが勝ち）:
   1. 環境変数 `CCDIRENV_PROFILE`（強制指定）
   2. 現在ディレクトリまたは親ディレクトリにある `.ccdirenv` マーカーファイル
   3. `~/.ccdirenv/config.toml` の `[directories]` glob
   4. **owner 検出**（git / ghq、`discovery_priority` の順で実行）→ `[owners]` ルックアップ
   5. `default_profile`、未指定なら `default`

Claude Code 自身のインストール先・自動更新経路には一切触りません。

## インストール

```sh
# crates.io
cargo install ccdirenv

# Homebrew（macOS / Linuxbrew）
brew tap SuguruOoki/tap
brew install ccdirenv

# Nix（flakes）
nix profile install github:SuguruOoki/ccdirenv
# インストールせずワンショット実行する場合:
nix run github:SuguruOoki/ccdirenv -- which

# Shell installer（macOS / Linux のビルド済みバイナリ）
curl --proto '=https' --tlsv1.2 -LsSf \
  https://github.com/SuguruOoki/ccdirenv/releases/latest/download/ccdirenv-installer.sh | sh
```

### ghq を使う場合

`ccdirenv init` で ghq モード（または both）を選ぶと、ghq が PATH に無ければ自動インストールします（Homebrew → `go install` の順）。あとから明示的に再チェックしたい場合は `ccdirenv ghq install`。CI などで自動インストールを抑止するには `CCDIRENV_SKIP_GHQ_AUTOINSTALL=1`。

### 素の git で使う場合

git 以外に追加の依存はありません。worktree（`.git` がファイル）も submodule もそのまま動きます。

## セットアップ

```sh
# 1. プロファイルディレクトリ + シムをセットアップし、検出モードを選ぶ
#    `ccdirenv init` は対話式。プロンプトをスキップしたい場合は --mode=git|ghq|both|off
ccdirenv init

# 2. シェル rc (~/.zshrc, ~/.bashrc 等) に追加:
#    export PATH="$HOME/.ccdirenv/bin:$PATH"

# 3. (任意) 既存の ~/.claude/ ログイン情報を 'default' プロファイルに移行
ccdirenv import default

# 4. 追加プロファイルを作成・ログイン
ccdirenv login work
ccdirenv login personal

# 5. GitHub owner をプロファイルに紐付け（git/ghq どちらのモードでも同じマップを参照）
ccdirenv owners map github.com/your-personal-handle  default
ccdirenv owners map github.com/your-employer         work
ccdirenv owners map github.com/a-client-org          client-acme
```

これで完了。マップ済み owner のリポジトリは `cd` した瞬間から正しいアカウントで動きます。

`init` で表示されるプロンプト:

```
How do you organize Git repositories?
  1) ghq   — uses ~/ghq/<host>/<owner>/<repo> layout (ghq priority + git fallback)
  2) git   — any repository with a git remote (default)
  3) both  — both methods enabled (git priority, ghq fallback)
  4) off   — no repo-aware detection
Choose [1-4, default 2]:
```

あとから `ccdirenv mode set <ghq|git|both|off>` で切替可能。

## 設定ファイル

```toml
# ~/.ccdirenv/config.toml
default_profile = "default"
discovery_priority = "git"          # "git" | "ghq" — 両方有効時にどちらを先に試すか

[ghq]
enabled = false                     # ghq path layout 検出
# root = "~/ghq"                    # 任意の上書き。未指定なら自動検出

[git]
enabled = true                      # .git/config の remote から検出
remote = "origin"                   # 参照する remote 名

# 共通の owner → profile マップ（git/ghq の両モードが参照）
[owners]
"github.com/SuguruOoki" = "default"
"github.com/TheMoshInc" = "mosh"
"github.com/AcmeCorp"   = "work"

# 任意: 明示的なディレクトリオーバーライド（owner 検出より優先）
[directories]
"~/sandbox/**" = "default"
```

## CLI

```sh
ccdirenv mode show                               # 現在の検出モード
ccdirenv mode set ghq|git|both|off               # モード切替

ccdirenv owners list                             # owner マッピング一覧
ccdirenv owners map github.com/Acme work         # 追加・更新
ccdirenv owners unmap github.com/Acme            # 削除

ccdirenv git show                                # git 検出設定の表示
ccdirenv git enable / disable
ccdirenv git remote upstream                     # 参照 remote を変更

ccdirenv ghq list                                # ghq 設定 + owner マップ
ccdirenv ghq enable / disable
ccdirenv ghq root /custom/path                   # ghq root を上書き（"" でクリア）
ccdirenv ghq install                             # ghq が無ければインストール
```

後方互換: `ccdirenv ghq map / unmap` は `ccdirenv owners map / unmap` のエイリアスとして引き続き動作します。v0.2 系の `[ghq.owners]` ブロックは config 読込時に自動で `[owners]` にマージされます。

### git 検出の挙動

cwd から親方向に `.git` を探す → `.git/config` を直接パース → `[remote "<configured>"]` の URL を抽出 → `<host>/<owner>` をルックアップ。subprocess は使いません。

サポートする URL 形式:
- `git@github.com:Acme/widget.git`
- `https://github.com/Acme/widget.git`
- `ssh://git@github.com/Acme/widget.git`
- セルフホストのホスト（`[owners]` に対応する `<host>/<owner>` を書けば動作）

`.git` がファイルの場合（**linked worktree** や **submodule**）は `gitdir:` と `commondir` を辿って実際の config に到達します。設定不要。

### リポ単位のオーバーライド

```sh
# A: 単一リポをマーカーで指定（最優先）
cd ~/some/repo && ccdirenv use personal

# B: config.toml に明示 glob（owner 検出より優先）
```

```toml
[directories]
"~/projects/sandbox/**" = "personal"
```

## その他のコマンド

```sh
ccdirenv which            # 現在ディレクトリで解決されるプロファイル
ccdirenv list             # プロファイル一覧（ログインメール付き）
ccdirenv use <profile>    # cwd にマーカーで紐付け
ccdirenv unuse            # マーカーを削除
ccdirenv config           # ~/.ccdirenv/config.toml を $EDITOR で開く
ccdirenv doctor           # 診断（PATH, claude 解決, 設定ファイル有無 等）
```

## 環境変数

| 変数 | 効果 |
|---|---|
| `CCDIRENV_PROFILE=<name>` | この呼び出しで使うプロファイルを強制指定 |
| `CCDIRENV_DISABLE=1` | 解決を完全にスキップ（デバッグ用） |
| `CCDIRENV_DEBUG=1` | exec 前に選択されたプロファイル名等を stderr に出力 |
| `CCDIRENV_HOME=<path>` | データルートの `~/.ccdirenv/` を別ディレクトリに差し替え（テスト用） |
| `CCDIRENV_SKIP_GHQ_AUTOINSTALL=1` | `ccdirenv init` / `ccdirenv ghq install` での ghq 自動インストールを抑止 |
| `GHQ_ROOT=<path>` | `[ghq] root` 未設定時の ghq root |

## トラブルシューティング

`ccdirenv doctor` を実行してください。シム / PATH / 本物の `claude` / 設定ファイルの状態を診断します。

シムを入れたのに `claude` が違うアカウントを拾ってしまう場合、`~/.ccdirenv/bin` が `~/.local/bin`（あるいは Claude Code のインストール先）**より前** に来ているかを確認。

意図せず `default` に解決される場合は `ccdirenv mode show` で現在のモードを確認 → `ccdirenv owners list` で owner が登録されているか確認。ghq モードなら `ccdirenv ghq list` の root が想定どおりか、git モードなら `ccdirenv git show` で remote 名を確認。

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
