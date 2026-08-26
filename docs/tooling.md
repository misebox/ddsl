# ツール

## CLI

```
nounsql <サブコマンド> [オプション] <入力>
```

| サブコマンド | 出力 |
|---|---|
| `check` | 診断のみ。エラーがなければ件数を表示する |
| `sql` | DDL |
| `ir` | 中間表現（intermediate representation）。mixin と blueprint を展開し、テーブル名・FK列名・index名を確定させた状態のスキーマ。`--json` で機械可読な形になる |
| `ast` | 構文木 |

| オプション | 内容 |
|---|---|
| `--dialect <名前>` | 出力ターゲット。既定は `postgres`。型名・予約語・FK の型解決はターゲットが持つ |
| `-o, --output <PATH>` | 出力先。省略すると標準出力。親ディレクトリは自動で作る |
| `--deny-warnings` | 警告があっても終了コードを 1 にする |
| `--json` | `ir` のみ。中間表現を JSON で出す |

入力に `-` を渡すと標準入力から読む。診断は常に標準エラーに出るので、出力をパイプに繋いでも混ざらない。

```
nounsql sql schema.nsql -o schema.sql
cat schema.nsql | nounsql sql - > schema.sql
nounsql check schema.nsql --deny-warnings   # CI 向け
nounsql ir schema.nsql --json -o schema.json
```

診断は 1 回の実行でまとめて出る。1行1文なので、エラーの出た行を読み飛ばして次の文から解析を続ける。

```
error: 知らない属性キー `foo`。使えるのは type / null / default / on_update / comment
  --> schema.nsql:2:26
   2 |   column email type=text foo=1
     |                          ^^^
```

## Language Server

`crates/nounsql-lsp` が stdio の LSP サーバ。

| 機能 | 内容 |
|---|---|
| 診断 | 保存前に構文エラー・解決エラー・警告を出す |
| 定義へ移動 | `use` → mixin、`belongs_to` → table、`apply_blueprint` → blueprint |
| ホバー | 名詞から解決されるテーブル名 |
| アウトライン | table / mixin / blueprint |
| 補完 | キーワード・属性キー・型名・mixin 名・名詞 |

```
cargo install --path crates/nounsql-lsp
```

## VS Code

`editors/vscode` に拡張がある。syntax highlight（TextMate grammar）と LSP クライアントを含む。

```
cd editors/vscode
npm install
npm run compile
```

`nounsql-lsp` が PATH に無い場合は設定 `nounsql.server.path` で場所を指定する。

## 例

`examples/` に動く例がある。すべて警告なしで通る。

| ファイル | 内容 |
|---|---|
| `minimal.nsql` | 最小の構成 |
| `relations.nsql` | 同一テーブルへの複数参照、自己参照、複合主キー、`associate` |
| `blueprint.nsql` | 1つの blueprint から3テーブル。2つの名詞に適用する |
| `config.nsql` | 命名規則と制約を既定から外して効果を見る |
| `sample.nsql` | 仕様に出てくる構文をすべて含む |

```
nounsql sql examples/blueprint.nsql
```

## プレイグラウンド

[プレイグラウンド](playground.html)はブラウザの中でコンパイルする。入力はどこにも送られない。

`crates/nounsql-wasm` が `nounsql-core` を WebAssembly に落としたもので、CLI と同じ字句解析・解決・コード生成・シンタックスハイライトを使う。`nounsql-core` の依存は `indexmap` だけで I/O を持たないため、そのまま wasm32 に載る。

```
wasm-pack build crates/nounsql-wasm --target web --out-dir pkg --release
cargo run -p nounsql-site
```

`wasm-pack` の生成物はサイト生成時に `dist/` へ複製される。

## npm パッケージ

WebAssembly 版は npm の [`nounsql`](https://www.npmjs.com/package/nounsql) として公開している。ブラウザでも Node でも、Rust のツールチェーン無しでコンパイルできる。

```
npm install nounsql
```

```ts
import init, { compile } from "nounsql";

await init();
const { sql, ir, diagnostics } = compile(source, "postgres");
```

`ir` は解決済みスキーマで、TypeScript の型が付いている。ORM のモデル生成などはこれを読んで書く。

```ts
for (const table of ir.tables) {
  // table.singular  … "user"        モデル名に使う
  // table.name      … "users"       テーブル名
  // table.columns   … 宣言順のカラム
  // table.foreignKeys / table.reverses … 両方向の関連
}
```

同じ内容は CLI の `nounsql ir --json` でも得られる。

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
