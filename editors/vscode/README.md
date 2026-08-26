# NounSQL for VS Code

Syntax highlighting and a language server client for `.nsql` files.

```sh
cargo install nounsql-lsp
cd editors/vscode && npm install && npm run compile
```

If `nounsql-lsp` is not on your `PATH`, point `nounsql.server.path` at it.

| Feature | What it does |
|---|---|
| Diagnostics | syntax, resolution and warnings, before you save |
| Go to definition | `use` to its mixin, `belongs_to` to its table, `apply_blueprint` to its blueprint |
| Hover | the table a noun resolves to |
| Outline | tables, mixins and blueprints |
| Completion | keywords, attributes, type names, mixin names and nouns |
