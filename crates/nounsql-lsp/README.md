# nounsql-lsp

A language server for [NounSQL](https://github.com/misebox/nounsql), speaking the
Language Server Protocol over stdio.

```sh
cargo install nounsql-lsp
```

| Feature | What it does |
|---|---|
| Diagnostics | syntax, resolution and warnings, before you save |
| Go to definition | `use` to its mixin, `belongs_to` to its table, `apply_blueprint` to its blueprint |
| Hover | the table a noun resolves to |
| Outline | tables, mixins and blueprints |
| Completion | keywords, attributes, type names, mixin names and nouns |

The VS Code extension lives in `editors/vscode` in the repository.

Documentation: https://misebox.github.io/nounsql
