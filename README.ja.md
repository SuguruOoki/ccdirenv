# ccdirenv

[![CI](https://github.com/SuguruOoki/ccdirenv/actions/workflows/ci.yml/badge.svg)](https://github.com/SuguruOoki/ccdirenv/actions/workflows/ci.yml)
[![crates.io](https://img.shields.io/crates/v/ccdirenv.svg)](https://crates.io/crates/ccdirenv)
[![License: MIT OR Apache-2.0](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue.svg)](#ライセンス)

**[English](./README.md) | 日本語**

> direnv 風にディレクトリ単位で Claude Code / Codex CLI のアカウントを自動切り替え。`cd` するだけで適切なアカウントで `claude` / `codex` が起動します。[ghq](https://github.com/x-motemen/ghq) でも素の git でも、両方併用でも OK。

`ccdirenv` は現在のディレクトリに応じて Claude Code / Codex CLI のアカウントを透過的に選択します。**GitHub の owner（ユーザー / Org）単位で 1 度だけプロファイルを紐付けておけば、その owner 配下のリポジトリは現在も将来 clone するものも含めてすべて正しいアカウントで起動します。** 個人リポは個人アカウント、業務リポは業務アカウント、クライアント案件はクライアントアカウント、というように。

## 2 つの検出方式

インストール時に好きな方式を選べます:

- **git モード（既定）** — git remote を持つ任意のリポジトリで動作。`.git/config` を直接読み（`git` コマンドの subprocess は使用しない）、worktree / submodule も自動 follow して `<host>/<owner>` を抽出。
- **ghq モード** — 最速。`~/ghq/<host>/<owner>/<repo>` というパス構造から `<host>/<owner>` を取り出すだけ。すでに ghq を使っているなら推奨。
- **both** — 一方が外れたらもう一方を試す。ghq 配下のリポと外のリポが混在する人向け。
- **off** — 自動検出なし。`[directories]` glob と `.ccdirenv` マーカーのみ。

どのモードでも同じ `[owners]` マップが使われます。

## 仕組み

1. プロファイルごとに Claude Code と Codex CLI の設定ディレクトリを分けて保存します（下表参照）。ログインはツールごとに行います。
2. `PATH` の前段に置かれた小さな Rust 製シムが、現在のディレクトリからプロファイルを解決し、`claude` には `CLAUDE_CONFIG_DIR`、`codex` には `CODEX_HOME` を設定した上で本物のバイナリに処理を引き渡します。
3. 解決の優先順位（先にマッチしたものが勝ち）:
   1. 環境変数 `CCDIRENV_PROFILE`（強制指定）
   2. 現在ディレクトリまたは親ディレクトリにある `.ccdirenv` マーカーファイル
   3. `~/.ccdirenv/config.toml` の `[directories]` glob
   4. **owner 検出**（git / ghq、`discovery_priority` の順で実行）→ `[owners]` ルックアップ
   5. `default_profile`、未指定なら `default`

各ツール自身のインストール先・更新経路は維持します。シムのディレクトリと ccdirenv 自身を除外して、PATH 上の本物の実行ファイルを探します。

## 対応ツール

| | Claude Code | Codex CLI |
|---|---|---|
| 自動切り替えするコマンド | `claude` | `codex` |
| 設定する環境変数 | `CLAUDE_CONFIG_DIR` | `CODEX_HOME` |
| 保存先 | `~/.ccdirenv/profiles/<name>/` | `~/.ccdirenv/profiles/<name>/codex/` |
| ログイン | `ccdirenv login work` | `ccdirenv login work --tool codex` |
| 既存設定の移行 | `ccdirenv import work` | `ccdirenv import work --tool codex` |
| 選択・状態・インストールの確認 | `ccdirenv which / list / doctor` | 各コマンドに `--tool codex` を追加 |

owner マッピング、ディレクトリ glob、マーカー、`CCDIRENV_PROFILE` は両ツールで共通です。既存の Claude 用保存先とコマンドは引き続き使えます。`--tool` の既定値は `claude` です。ccdirenv のプロファイルはアカウントと保存先を選び、Codex 自身の `--profile` は選択された Codex home 内の設定を選びます。

Codex CLI 対応は v0.4.0 から利用できます。ccdirenv を更新後、`ccdirenv init` を再実行すると両ツールのシムが追加されます。

## インストール

```sh
# crates.io
cargo install ccdirenv

# Homebrew（macOS / Linuxbrew）
brew tap SuguruOoki/ccdirenv
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

Claude Code / Codex CLI 本体は別途インストールしてください。`ccdirenv init` は両方のシムを作成しますが、利用する CLI だけインストールされていれば動作します。

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

### Codex のセットアップ・既存環境への追加

このバージョンのインストール後に `ccdirenv init` を再実行すると Codex のシムが追加されます。`--mode` を明示しなければ、既存の検出設定・マッピング・プロファイル内容を維持します。

```sh
ccdirenv init --no-prompt
export PATH="$HOME/.ccdirenv/bin:$PATH"

# 任意: ~/.codex を default にコピー。移行先にファイルがある場合は上書きしません。
ccdirenv import default --tool codex
# 別の移行元を指定する場合は --from /path/to/codex-home を追加。

ccdirenv login work --tool codex
ccdirenv owners map github.com/your-employer work

# 業務リポジトリ内で:
ccdirenv which --tool codex
codex
ccdirenv list --tool codex
ccdirenv doctor --tool codex

# -- 以降をログインコマンドに渡す（デバイス認証、標準入力から API キーを渡す例）:
ccdirenv login work --tool codex -- --device-auth
printenv OPENAI_API_KEY | ccdirenv login work --tool codex -- --with-api-key
```

新しい Codex home は空の状態で作成します。ユーザー設定・ルール・skills・MCP 設定を引き継ぐ場合は `import` するか各 home に設定してください。Claude の設定を Codex 用に自動変換する機能ではありません。独自ラッパーで既に `profiles/<name>/codex` を使っている場合は、その保存先を引き続き使用します。

Codex は `CODEX_HOME` に設定やローカル状態を保存します。認証情報の保存先は `auth.json` または OS の資格情報ストアです。ファイルの移行では OS キーリング内の認証情報は移せないため、必要に応じて再ログインしてください。`list --tool codex` は `codex login status` で状態を確認し、トークンやメールアドレスは表示しません。根拠: [Codex の設定仕様](https://developers.openai.com/codex/config-advanced/)、[認証仕様](https://developers.openai.com/codex/auth/)。

移行時は skills などの通常のシンボリックリンクを維持します。認証ファイルがリンクの場合は、プロファイル間で認証情報が共有されるのを防ぐためエラーにします。移行先は移行元ディレクトリの外に指定してください。Claude プロファイル直下の `codex` は Codex 用の予約ディレクトリです。

プロファイルはシム起動時のカレントディレクトリから選択します。`codex -C` などツール側の作業ディレクトリ指定では変わらないため、先に `cd` するか `CCDIRENV_PROFILE` を指定してください。選択されたツールの `CODEX_HOME` / `CLAUDE_CONFIG_DIR` は継承値よりプロファイルの保存先を優先します。`CCDIRENV_DISABLE=1` では継承値を維持します。他の環境変数はそのまま引き継ぎます。

対応範囲は macOS / Linux でシム経由で起動する CLI です。デスクトップアプリ、IDE 拡張、本体の絶対パスによる起動など、PATH 上のシムを通らない起動では自動切り替えされません。

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
ccdirenv doctor --tool codex # Codex CLI の同じ診断
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

Claude Code は `ccdirenv doctor`、Codex CLI は `ccdirenv doctor --tool codex` を実行してください。シム / PATH の優先順位 / 本体の実行ファイル / 設定ファイルの状態を診断します。

シムを入れたのに `claude` / `codex` が違うアカウントを拾ってしまう場合、`~/.ccdirenv/bin` が `~/.local/bin`（あるいは CLI 本体のインストール先）**より前** に来ているかを確認。

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
