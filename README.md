# nounsql

[![ci](https://github.com/misebox/nounsql/actions/workflows/ci.yml/badge.svg)](https://github.com/misebox/nounsql/actions/workflows/ci.yml)
[![pages](https://github.com/misebox/nounsql/actions/workflows/pages.yml/badge.svg)](https://github.com/misebox/nounsql/actions/workflows/pages.yml)
[![docs](https://img.shields.io/badge/docs-misebox.github.io%2Fnounsql-2f6f4f)](https://misebox.github.io/nounsql)
[![rust](https://img.shields.io/badge/rust-edition%202024-b7410e)](https://doc.rust-lang.org/edition-guide/)

**NounSQL is a DSL for Database Schema Design.**

A compiler from `.nsql` to SQL DDL.

[Documentation](https://misebox.github.io/nounsql)  ·  [Development](DEVELOPMENT.md)

## Install

```sh
cargo install nounsql        # the compiler
cargo install nounsql-lsp    # language server
npm install nounsql          # WebAssembly build, for JavaScript
```

```sh
nounsql sql schema.nsql -o schema.sql
```

## Contributing

Building, previewing the site and releasing are described in
[DEVELOPMENT.md](DEVELOPMENT.md).

## License

[MIT](LICENSE-MIT) or [Apache-2.0](LICENSE-APACHE), at your option.
