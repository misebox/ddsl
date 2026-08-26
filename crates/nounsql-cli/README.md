# nounsql

**NounSQL is a DSL for Database Schema Design.**

`.nsql` から PostgreSQL の DDL を生成するコンパイラ。

```
cargo install nounsql
nounsql sql schema.nsql
```

| サブコマンド | 出力 |
|---|---|
| `check` | 診断のみ |
| `sql` | DDL |
| `ir` | 中間表現（解決済みスキーマ） |
| `ast` | 構文木 |

| オプション | 内容 |
|---|---|
| `--dialect <名前>` | 出力ターゲット。既定 `postgres` |
| `-o, --output <PATH>` | 出力先。省略すると標準出力 |
| `--deny-warnings` | 警告があっても失敗させる |

入力に `-` を渡すと標準入力から読む。

```
mixin base {
  column id type=serial
  pk id
}

table post {
  use base
  belongs_to user
  column title type=text
}
```

ドキュメント: https://misebox.github.io/nounsql
