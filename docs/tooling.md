# 内部

コンパイラの構成と、リポジトリの中身。使い方は[使い方](guide.md)、動く例は `examples/` にある。

## WebAssembly

`crates/nounsql-wasm` が `nounsql-core` を WebAssembly に落としたもの。CLI と同じ字句解析・解決・コード生成・シンタックスハイライトを使う。`nounsql-core` の依存は `indexmap` だけで I/O を持たないため、そのまま wasm32 に載る。

npm パッケージとドキュメントサイトのプレイグラウンドの両方がこれを使う。ブラウザ向けとバンドラ向けで要る形が違うので、出力ターゲットを分けている。

```
wasm-pack build crates/nounsql-wasm --target web     --out-dir pkg     --release  # サイト用
wasm-pack build crates/nounsql-wasm --target bundler --out-dir pkg-npm --release  # npm 用
```

## GBNF

[`nounsql.gbnf`](nounsql.gbnf) は llama.cpp 系の制約付きデコード用の文法。LLM に NounSQL を生成させるときに構文を外させないために使う。

## パイプライン

```
source
 → lexer      改行を文終端とする。eval(...) の中身は括弧の対応だけ見て1トークンで抜く
 → parser     手書き再帰下降。行単位で回復して診断をまとめる
 → resolver   blueprint展開 → mixin展開 → relation展開 → 命名解決
 → ir         中間表現。mixin と blueprint が消え、名前がすべて確定したスキーマ
 → codegen    ターゲットごとの DDL
```

`belongs_to` は宣言位置で FK 列の枠だけ先に確保し、参照先の主キーの型が判ってから型を埋める。これで `use` と同じく宣言順どおりの列順になる。

## ドキュメントを書くとき

```
bin/preview
```

Vite の開発サーバが立つ。`docs/*.md` を書き換えると即座に反映される。プレイグラウンドの WebAssembly も一緒にビルドする。

| オプション | 内容 |
|---|---|
| `--port <番号>` | 既定は 4321 |
| `--build` | 本番と同じものをビルドして配信する |
| `--no-wasm` | WebAssembly を作り直さない。起動が速い |

サイトの実装は `site/` にある。`docs/*.md` と `examples/*.nsql` は写しを持たず、リポジトリの実物を `import.meta.glob` で読む。markdown の変換は marked、コードブロックの色付けは WebAssembly に落としたコンパイラ本体の字句解析を使う。

ドキュメントは静的なページの方が向いているので SPA にしていない。Vite の複数ページビルドで、1ページ1 HTML を出している。

## crate 構成

| パス | 内容 |
|---|---|
| `crates/nounsql-core` | lexer / parser / resolver / codegen |
| `crates/nounsql-cli` | `nounsql` コマンド |
| `crates/nounsql-lsp` | language server |
| `crates/nounsql-wasm` | WebAssembly バインディング（npm パッケージとプレイグラウンド） |
| `site` | ドキュメントサイト。bun + Vite + SolidJS + marked |
| `editors/vscode` | VS Code 拡張 |
