# 使い方

## インストール

```
cargo install nounsql        # コンパイラ
cargo install nounsql-lsp    # language server
```

バイナリは [Releases](https://github.com/misebox/nounsql/releases) からも取れる。JavaScript から使う場合は npm パッケージがある。

```
npm install nounsql
```

## コンパイルする

```
nounsql <サブコマンド> [オプション] <入力>
```

| サブコマンド | 出力 |
|---|---|
| `check` | 診断のみ。エラーがなければ件数を表示する |
| `sql` | DDL |
| `ir` | 中間表現。mixin と blueprint を展開し、名前を確定させた状態のスキーマ |
| `ast` | 構文木 |

| オプション | 内容 |
|---|---|
| `--dialect <名前>` | 出力ターゲット。既定は `postgres` |
| `-o, --output <PATH>` | 出力先。省略すると標準出力。親ディレクトリは自動で作る |
| `--deny-warnings` | 警告があっても終了コードを 1 にする |
| `--json` | `ir` のみ。中間表現を JSON で出す |

入力に `-` を渡すと標準入力から読む。診断は常に標準エラーに出るので、出力をパイプに繋いでも混ざらない。

```
nounsql sql schema.nsql -o schema.sql
cat schema.nsql | nounsql sql - > schema.sql
nounsql check schema.nsql --deny-warnings   # CI 向け
```

## 診断を読む

エラーは1回の実行でまとめて出る。1行1文なので、失敗した行を飛ばして次の文から解析を続ける。

```
error: 知らない属性キー `foo`。使えるのは type / null / default / on_update / comment
  --> schema.nsql:2:26
   2 |   column email type=text foo=1
     |                          ^^^
```

警告は既定では失敗にならない。名詞の登録漏れや、主キーの無いテーブルがここに出る。CI で止めたい場合は `--deny-warnings` を付ける。

## エディタ

`crates/nounsql-lsp` が stdio の language server。

| 機能 | 内容 |
|---|---|
| 診断 | 保存前に構文エラー・解決エラー・警告を出す |
| 定義へ移動 | `use` → mixin、`belongs_to` → table、`apply_blueprint` → blueprint |
| ホバー | 名詞から解決されるテーブル名 |
| アウトライン | table / mixin / blueprint |
| 補完 | キーワード・属性キー・型名・mixin 名・名詞 |

VS Code 拡張は `editors/vscode` にある。syntax highlight と LSP クライアントを含む。

```
cd editors/vscode
npm install
npm run compile
```

`nounsql-lsp` が PATH に無い場合は設定 `nounsql.server.path` で場所を指定する。

## JavaScript から使う

npm の [`nounsql`](https://www.npmjs.com/package/nounsql) は WebAssembly に落としたコンパイラ本体で、ブラウザでも Node でも動く。Rust のツールチェーンは要らない。

```ts
import init, { compile } from "nounsql";

await init();
const { sql, ir, diagnostics } = compile(source, "postgres");
```

`ir` は解決済みスキーマで、TypeScript の型が付いている。ORM のモデル生成などはこれを読んで書く。

```ts
for (const table of ir.tables) {
  // table.singular  … "user"   モデル名に使う
  // table.name      … "users"  テーブル名
  // table.columns   … 宣言順のカラム
  // table.foreignKeys / table.reverses … 両方向の関連
}
```

同じ内容は `nounsql ir --json` でも得られる。
