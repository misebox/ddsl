# nounsql-lsp

[NounSQL](https://github.com/misebox/nounsql) の language server。stdio で話す。

```
cargo install nounsql-lsp
```

| 機能 | 内容 |
|---|---|
| 診断 | 構文エラー・解決エラー・警告 |
| 定義へ移動 | `use` → mixin、`belongs_to` → table、`apply_blueprint` → blueprint |
| ホバー | 名詞から解決されるテーブル名 |
| アウトライン | table / mixin / blueprint |
| 補完 | キーワード・属性キー・型名・mixin 名・名詞 |

VS Code 拡張はリポジトリの `editors/vscode` にある。

ドキュメント: https://misebox.github.io/nounsql
