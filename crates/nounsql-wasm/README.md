# nounsql-wasm

[NounSQL](https://github.com/misebox/nounsql) コンパイラの WebAssembly バインディング。

ドキュメントサイトのプレイグラウンドと、npm の [`nounsql`](https://www.npmjs.com/package/nounsql) で使う。crates.io には公開しない。

```
wasm-pack build crates/nounsql-wasm --target web     # サイト用
wasm-pack build crates/nounsql-wasm --target bundler # npm 用
```

TypeScript の型は `src/lib.rs` の `typescript_custom_section` で定義している。
`nounsql-core` の `ir` を変えたらこちらも直す。`serialized_ir_keys_are_stable`
テストがずれを検出する。
