# ddsl

DB設計用DSLのコンパイラ。`.ddsl` から PostgreSQL の DDL を生成する。

仕様は [docs/spec.md](docs/spec.md)。未決の設計判断は [docs/open-questions.md](docs/open-questions.md)。

## 構成

| パス | 内容 |
|---|---|
| `crates/ddsl-core` | lexer / parser / resolver / codegen |
| `crates/ddsl-cli` | `ddsl` コマンド |
| `crates/ddsl-lsp` | language server |
| `editors/vscode` | VS Code 拡張（syntax highlight + LSP クライアント） |
| `docs/ddsl.gbnf` | GBNF 文法（制約付きデコード用） |
| `examples/` | サンプルと生成結果 |

## 使い方

```
cargo run -p ddsl -- check examples/sample.ddsl   # 診断のみ
cargo run -p ddsl -- build examples/sample.ddsl   # DDL を出力
cargo run -p ddsl -- --dialect postgres build examples/sample.ddsl
cargo run -p ddsl -- ir    examples/sample.ddsl   # 解決済みスキーマ
cargo run -p ddsl -- ast   examples/sample.ddsl   # 構文木
```

## パイプライン

```
source
 → lexer      改行を文終端とする。eval(...) の中身は生のまま1トークンにする
 → parser     手書き再帰下降。行単位で回復して診断をまとめて出す
 → resolver   blueprint展開 → mixin展開(use/except/override) → relation展開 → 命名解決
 → ir         テーブル名・FK列名・index名が確定したスキーマ
 → codegen    PostgreSQL DDL
```

## 実装済み

- `column` / `pk` / `index` / `use` / `override` / `except` / `except index` / `belongs_to` / `unique_belongs_to`
- `mixin`（`use` の位置に展開、循環検出）
- `blueprint`（`let` + `name_join`、仮引数と entity 名の衝突検出）
- `associate` / `apply_blueprint`
- `naming` / `constraints` の既定値と上書き
- 出力ターゲットは `--dialect` で選択（既定 `postgres`）。型名・予約語・FK型の解決はターゲット側が持つ
- `entities` 辞書と規則変化フォールバック（辞書に無い語は警告）
- DDL 出力: CREATE TABLE / INDEX / FK / COMMENT、`on_update=` はトリガに落とす
# ddsl
