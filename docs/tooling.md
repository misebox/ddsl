# ツール

## CLI

```
cargo run -p ddsl -- <サブコマンド> [--dialect <名前>] <入力>
```

| サブコマンド | 出力 |
|---|---|
| `check` | 診断のみ。エラーがなければ件数を表示する |
| `build` | DDL |
| `ir` | 解決済みスキーマ（テーブル名・FK列名・index名が確定した状態） |
| `ast` | 構文木 |

`--dialect` は出力ターゲットを選ぶ。既定は `postgres`。型名・予約語・FK の型解決はターゲットが持つ。

診断は 1 回の実行でまとめて出る。1行1文なので、エラーの出た行を読み飛ばして次の文から解析を続ける。

```
error: 知らない属性キー `foo`。使えるのは type / null / default / on_update / comment
  --> schema.ddsl:2:26
   2 |   column email type=text foo=1
     |                          ^^^
```

## Language Server

`crates/ddsl-lsp` が stdio の LSP サーバ。

| 機能 | 内容 |
|---|---|
| 診断 | 保存前に構文エラー・解決エラー・警告を出す |
| 定義へ移動 | `use` → mixin、`belongs_to` → table、`apply_blueprint` → blueprint |
| ホバー | entity から解決されるテーブル名 |
| アウトライン | table / mixin / blueprint |
| 補完 | キーワード・属性キー・型名・mixin 名・entity 名 |

```
cargo install --path crates/ddsl-lsp
```

## VS Code

`editors/vscode` に拡張がある。syntax highlight（TextMate grammar）と LSP クライアントを含む。

```
cd editors/vscode
npm install
npm run compile
```

`ddsl-lsp` が PATH に無い場合は設定 `ddsl.server.path` で場所を指定する。

## GBNF

[`ddsl.gbnf`](ddsl.gbnf) は llama.cpp 系の制約付きデコード用の文法。LLM に DDSL を生成させるときに構文を外させないために使う。

## パイプライン

```
source
 → lexer      改行を文終端とする。eval(...) の中身は括弧の対応だけ見て1トークンで抜く
 → parser     手書き再帰下降。行単位で回復して診断をまとめる
 → resolver   blueprint展開 → mixin展開 → relation展開 → 命名解決
 → ir         テーブル名・FK列名・index名が確定したスキーマ
 → codegen    ターゲットごとの DDL
```

`belongs_to` は宣言位置で FK 列の枠だけ先に確保し、参照先の主キーの型が判ってから型を埋める。これで `use` と同じく宣言順どおりの列順になる。

## crate 構成

| パス | 内容 |
|---|---|
| `crates/ddsl-core` | lexer / parser / resolver / codegen |
| `crates/ddsl-cli` | `ddsl` コマンド |
| `crates/ddsl-lsp` | language server |
| `crates/ddsl-site` | このドキュメントサイトの生成 |
| `editors/vscode` | VS Code 拡張 |
