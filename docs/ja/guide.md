# ガイド

## インストール

```sh
cargo install nounsql        # コンパイラ
cargo install nounsql-lsp    # language server
```

バイナリは各 [release](https://github.com/misebox/nounsql/releases) にも付いています。
JavaScript から呼ぶなら npm パッケージがあります。

```sh
npm install nounsql
```

## コンパイルする

```text
nounsql <command> [options] <input>
```

| コマンド | 出力 |
|---|---|
| `check` | 診断だけ。何も落ちなければ最後に件数 |
| `sql` | DDL |
| `ir` | 解決済みのスキーマ。mixin と blueprint は展開され、名前は確定している |
| `ast` | 構文木 |

| オプション | 効果 |
|---|---|
| `--dialect <name>` | 出力先。既定は `postgres` |
| `-o, --output <path>` | 標準出力ではなくファイルへ書く。親ディレクトリは作る |
| `--deny-warnings` | 警告が1つでもあれば終了コードを非ゼロにする |
| `--json` | `ir` のみ。中間表現を JSON で出す |

入力に `-` を渡すと標準入力を読みます。診断は常に標準エラーへ出るので、
出力をパイプしても混ざりません。

```sh
nounsql sql schema.nsql -o schema.sql
cat schema.nsql | nounsql sql - > schema.sql
nounsql check schema.nsql --deny-warnings   # CI 用
```

## 診断を読む

エラーはまとめて出ます。1行1文の文法なので、失敗した行は飛ばして解析を続けられ、
1回の実行で分かるだけのことを言います。

```text
error: unknown attribute `foo`; expected type / null / default / on_update / comment
  --> schema.nsql:2:26
   2 |   column email type=text foo=1
     |                          ^^^
```

警告は既定では実行を失敗させません。辞書に無い名詞や、主キーの無いテーブルは
ここに出ます。`--deny-warnings` を付けると致命的になります。

## エディタで使う

`nounsql-lsp` は stdio 越しに Language Server Protocol を話します。

| 機能 | 内容 |
|---|---|
| 診断 | 構文・解決・警告を、保存する前に |
| 定義へ移動 | `use` から mixin へ、`belongs_to` からテーブルへ、`apply_blueprint` から blueprint へ |
| ホバー | 名詞がどのテーブルになるか |
| アウトライン | table・mixin・blueprint |
| 補完 | キーワード、属性、型名、mixin 名、名詞 |

VS Code 拡張は `editors/vscode` にあり、シンタックスハイライトと
language server クライアントを両方含みます。

```sh
cd editors/vscode
npm install
npm run compile
```

`nounsql-lsp` が `PATH` に無ければ `nounsql.server.path` で指してください。

## JavaScript から使う

npm パッケージ [`nounsql`](https://www.npmjs.com/package/nounsql) は
コンパイラを WebAssembly にしたものです。Rust のツールチェーン無しで、
ブラウザでも Node でも動きます。

```ts
import init, { compile } from "nounsql";

await init();
const { sql, ir, diagnostics } = compile(source, "postgres");
```

`ir` は解決済みのスキーマで、型が付いています。ORM のモデルを生成するとは、
これを読むということです。

テーブルは名前を3つの形で持ちます。`name` が DDL に出るもので、
`singular` と `plural` は元になった名詞です。モデル名やコレクション名は
たいていこちらから作ります。`name` は既定では単数形で、
`naming.table_name` を `plural` にすると複数形になります。

```ts
for (const table of ir.tables) {
  table.name;      // "post"   — テーブル
  table.singular;  // "post"   — モデル名に
  table.plural;    // "posts"  — コレクション名に
  table.comment;   // "Something an account wrote"
}
```

`columns` は宣言順のマップなので、辿れば DDL の列順になります。

```ts
for (const column of Object.values(table.columns)) {
  column.name;     // "account_id"
  column.type;     // "integer"
  column.null;     // false
  column.default;  // null / { literal: "0" } / { eval: "now()" }
  column.comment;  // "Who wrote it"
  column.origin;   // { kind: "generated", by: "belongs_to" }
}
```

`origin` はその列がどこから来たか——テーブルに直接書かれたのか、mixin が
持ち込んだのか、関連が生成したのか——を示します。関連を別の形で表す
モデルなら、生成された列を飛ばすのに使えます。

関連は両側に出ます。`foreignKeys` がキーを持つ側、`reverses` が指される側です。

```ts
for (const fk of table.foreignKeys) {
  fk.alias;       // "account"  — こちら側での関連名
  fk.columns;     // ["account_id"]
  fk.refTable;    // "account"
  fk.refColumns;  // ["id"]
  fk.onDelete;    // "cascade"
}

for (const reverse of table.reverses) {
  reverse.alias;      // "posts"
  reverse.fromTable;  // "post"     — キーがある側
  reverse.via;        // ["account_id"]
  reverse.unique;     // false      — true なら1対1
}
```

`nounsql ir --json` も同じものを出します。
