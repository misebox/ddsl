# nounsql

[![ci](https://github.com/misebox/nounsql/actions/workflows/ci.yml/badge.svg)](https://github.com/misebox/nounsql/actions/workflows/ci.yml)
[![pages](https://github.com/misebox/nounsql/actions/workflows/pages.yml/badge.svg)](https://github.com/misebox/nounsql/actions/workflows/pages.yml)
[![docs](https://img.shields.io/badge/docs-misebox.github.io%2Fnounsql-2f6f4f)](https://misebox.github.io/nounsql)
[![rust](https://img.shields.io/badge/rust-edition%202024-b7410e)](https://doc.rust-lang.org/edition-guide/)

**NounSQL is a DSL for Database Schema Design.**

`.nsql` から PostgreSQL の DDL を生成するコンパイラ。

[ドキュメント](https://misebox.github.io/nounsql)

## 構成

| パス | 内容 |
|---|---|
| `crates/nounsql-core` | lexer / parser / resolver / codegen |
| `crates/nounsql-cli` | `nounsql` コマンド |
| `crates/nounsql-lsp` | language server |
| `crates/nounsql-wasm` | WebAssembly バインディング（ドキュメントのプレイグラウンド） |
| `crates/nounsql-site` | ドキュメントサイトの生成（`cargo run -p nounsql-site` → `dist/`） |
| `editors/vscode` | VS Code 拡張（syntax highlight + LSP クライアント） |
| `docs/nounsql.gbnf` | GBNF 文法（制約付きデコード用） |
| `examples/` | サンプルと生成結果 |
| `bin/preview` | ドキュメントサイトをローカルで見る |
| `bin/release` | リリース（検査・バージョン更新・crates.io / npm 公開・push） |

## インストール

```
cargo install nounsql        # CLI
cargo install nounsql-lsp    # language server
npm install nounsql          # WebAssembly 版（JS / TS から使う）
```

バイナリは [Releases](https://github.com/misebox/nounsql/releases) にもある。

## 使い方

```
cargo run -p nounsql -- check examples/sample.nsql   # 診断のみ
cargo run -p nounsql -- sql   examples/sample.nsql   # DDL を出力
cargo run -p nounsql -- ir    examples/sample.nsql   # 中間表現（解決済みスキーマ）
cargo run -p nounsql -- ast   examples/sample.nsql   # 構文木

nounsql sql schema.nsql -o schema.sql       # 出力先を指定する
cat schema.nsql | nounsql sql -             # 標準入力から読む
nounsql check schema.nsql --deny-warnings   # 警告も失敗にする
nounsql ir schema.nsql --json            # 中間表現を JSON で出す
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
# nounsql

## ドキュメント

```
bin/preview             # 生成して配信し、変更を見て作り直す
bin/preview --no-wasm   # WebAssembly を作り直さない。起動が速い
```

## リリース

```
bin/release patch           # 0.1.0 -> 0.1.1
bin/release minor           # 0.1.0 -> 0.2.0
bin/release major           # 0.1.0 -> 1.0.0
bin/release 0.2.0           # バージョンを直接指定する
bin/release patch --check   # 検査のみ、何も書き換えない
```

`CARGO_REGISTRY_TOKEN` に crates.io の API トークンを入れておく。cargo が直接読む変数。

公開は `nounsql-core` → `nounsql-lsp` → `nounsql` の順に行い、後続がレジストリで解決できるまで待つ。
公開済みのバージョンは飛ばすので、途中で失敗しても再実行できる。

npm へは WebAssembly 版を出す。`npm login` が要る。`--skip-npm` で飛ばせる。

タグの push で CI がバイナリをビルドし、GitHub Release を作る。

## ライセンス

[MIT](LICENSE-MIT) または [Apache-2.0](LICENSE-APACHE) のどちらかを選べる。
