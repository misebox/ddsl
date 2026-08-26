# ddsl

**Data Definition Schema Language is a DSL for Database Schema Design.**

`.ddsl` から PostgreSQL の DDL を生成するコンパイラ。

```
cargo install ddsl
ddsl sql schema.ddsl
```

| サブコマンド | 出力 |
|---|---|
| `check` | 診断のみ |
| `sql` | DDL |
| `ir` | 中間表現（解決済みスキーマ） |
| `ast` | 構文木 |

出力ターゲットは `--dialect` で選ぶ（既定 `postgres`）。

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

ドキュメント: https://misebox.github.io/ddsl
