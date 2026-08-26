# ddsl-core

[DDSL](https://github.com/misebox/ddsl) コンパイラのコア。

lexer / parser / resolver / codegen を提供する。CLI は [`ddsl`](https://crates.io/crates/ddsl)、language server は [`ddsl-lsp`](https://crates.io/crates/ddsl-lsp)。

```rust
use ddsl_core::{codegen, dialect, parse, resolve};

let (doc, diags) = parse(source);
let (schema, more) = resolve(&doc, dialect::default());
let sql = codegen::emit(dialect::default(), &schema);
```

ドキュメント: https://misebox.github.io/ddsl
