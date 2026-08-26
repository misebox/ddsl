# nounsql-core

The core of the [NounSQL](https://github.com/misebox/nounsql) compiler: lexer,
parser, resolver and code generation.

The CLI is [`nounsql`](https://crates.io/crates/nounsql) and the language server
is [`nounsql-lsp`](https://crates.io/crates/nounsql-lsp).

```rust
use nounsql_core::{codegen, dialect, parse, resolve};

let (doc, diagnostics) = parse(source);
let (schema, more) = resolve(&doc, dialect::default());
let sql = codegen::emit(dialect::default(), &schema);
```

The `serde` feature makes the resolved schema serialisable, which is what
`nounsql ir --json` and the WebAssembly build use.

Documentation: https://misebox.github.io/nounsql
