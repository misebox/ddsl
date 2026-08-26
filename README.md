# ddsl

[![ci](https://github.com/misebox/ddsl/actions/workflows/ci.yml/badge.svg)](https://github.com/misebox/ddsl/actions/workflows/ci.yml)
[![pages](https://github.com/misebox/ddsl/actions/workflows/pages.yml/badge.svg)](https://github.com/misebox/ddsl/actions/workflows/pages.yml)
[![docs](https://img.shields.io/badge/docs-misebox.github.io%2Fddsl-2f6f4f)](https://misebox.github.io/ddsl)
[![rust](https://img.shields.io/badge/rust-edition%202024-b7410e)](https://doc.rust-lang.org/edition-guide/)

**Data Definition Schema Language is a DSL for Database Schema Design.**

`.ddsl` から PostgreSQL の DDL を生成するコンパイラ。

[ドキュメント](https://misebox.github.io/ddsl): [概要](docs/index.md) / [仕様](docs/spec.md) / [ツール](docs/tooling.md)

## 構成

| パス | 内容 |
|---|---|
| `crates/ddsl-core` | lexer / parser / resolver / codegen |
| `crates/ddsl-cli` | `ddsl` コマンド |
| `crates/ddsl-lsp` | language server |
| `crates/ddsl-site` | ドキュメントサイトの生成（`cargo run -p ddsl-site` → `dist/`） |
| `editors/vscode` | VS Code 拡張（syntax highlight + LSP クライアント） |
| `docs/ddsl.gbnf` | GBNF 文法（制約付きデコード用） |
| `examples/` | サンプルと生成結果 |

## 使い方

```
cargo run -p ddsl -- check examples/sample.ddsl   # 診断のみ
cargo run -p ddsl -- sql   examples/sample.ddsl   # DDL を出力
cargo run -p ddsl -- --dialect postgres sql examples/sample.ddsl
cargo run -p ddsl -- ir    examples/sample.ddsl   # 中間表現（解決済みスキーマ）
cargo run -p ddsl -- ast   examples/sample.ddsl   # 構文木
```

## パイプライン

```
source
 → lexer      改行を文終端とする。eval(...) の中身は生のまま1トークンにする
 → parser     手書き再帰下降。行単位で回復して診断をまとめて出す
 → resolver   blueprint展開 → mixin展開(use/except/override) → relation展開 → 命名解決
 → ir         中間表現。mixin と blueprint が消え、名前がすべて確定したスキーマ
 → codegen    PostgreSQL DDL
```

## 実装済み

- `column` / `pk` / `index` / `use` / `override` / `except` / `except index`
- `belongs_to` / `unique_belongs_to`（`fk=` / `alias=`）と `has_many` / `has_one`（`via=` / `alias=`）
- `mixin`（`use` の位置に展開、循環検出）
- `blueprint`（`let` + `noun()`、仮引数と名詞の衝突検出）
- `associate` / `apply_blueprint`
- `naming` / `constraints` の既定値と上書き
- 出力ターゲットは `--dialect` で選択（既定 `postgres`）。型名・予約語・FK型の解決はターゲット側が持つ
- `nouns` 辞書と複合名詞 `noun(a, b, ...)`（数は文脈が決める）。規則変化に落ちたら警告
- DDL 出力: CREATE TABLE / INDEX / FK / COMMENT、`on_update=` はトリガに落とす
# ddsl
