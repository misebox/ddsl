# nounsql-core

[NounSQL](https://github.com/misebox/nounsql) コンパイラのコア。

lexer / parser / resolver / codegen を提供する。CLI は [`nounsql`](https://crates.io/crates/nounsql)、language server は [`nounsql-lsp`](https://crates.io/crates/nounsql-lsp)。

```rust
use nounsql_core::{codegen, dialect, parse, resolve};

let (doc, diags) = parse(source);
let (schema, more) = resolve(&doc, dialect::default());
let sql = codegen::emit(dialect::default(), &schema);
```

ドキュメント: https://misebox.github.io/nounsql
