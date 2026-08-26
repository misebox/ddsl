# NounSQL for VS Code

`.nsql` ファイルの syntax highlight と language server。

## 使い方

```
cargo install --path ../../crates/nounsql-lsp
cd editors/vscode && npm install && npm run compile
```

`nounsql-lsp` が PATH に無い場合は設定 `nounsql.server.path` で場所を指定する。

## 機能

| 機能 | 内容 |
|---|---|
| 診断 | 保存前でも構文エラー・解決エラーを表示 |
| 定義へ移動 | `use` → mixin、`belongs_to` → table、`apply_blueprint` → blueprint |
| ホバー | 名詞から解決されるテーブル名を表示 |
| アウトライン | table / mixin / blueprint |
| 補完 | キーワード・属性キー・型名・mixin名・名詞 |
