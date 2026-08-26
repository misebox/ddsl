# Development

How to build, run and release this repository. For what the language is, see the
[documentation site](https://misebox.github.io/nounsql).

## Layout

| Path | Contents |
|---|---|
| `crates/nounsql-core` | lexer, parser, resolver, code generation |
| `crates/nounsql-cli` | the `nounsql` command |
| `crates/nounsql-lsp` | language server |
| `crates/nounsql-wasm` | WebAssembly bindings for the npm package and the playground |
| `site` | documentation site — bun, Vite, SolidJS, marked |
| `editors/vscode` | VS Code extension |
| `docs` | the pages the site renders |
| `examples` | working schemas, all of which compile without warnings |
| `bin` | scripts for previewing and releasing |

## Requirements

Rust, and for the site and the npm package also `bun` and `wasm-pack`.

```sh
cargo install wasm-pack
```

## Everyday commands

```sh
cargo test
cargo clippy --all-targets -- -D warnings
cargo run -p nounsql -- sql examples/shop.nsql
```

`examples/sample.sql` is generated from `examples/sample.nsql` and checked in.
Regenerate it whenever the compiler's output changes, or the release script will
refuse to run.

```sh
cargo run -p nounsql -- sql examples/sample.nsql -o examples/sample.sql
```

## The site

```sh
bin/preview             # Vite dev server; edits to docs/ appear immediately
bin/preview --build     # build exactly what ships, then serve it
bin/preview --no-wasm   # skip rebuilding the WebAssembly; starts faster
```

`--port` changes the port, which defaults to 4321.

The site reads `docs/*.md` and `examples/*.nsql` straight from the repository
through `import.meta.glob`, so there is no copy to keep in sync. Markdown goes
through marked; code blocks are coloured by the compiler's own lexer, reached
through the WebAssembly build.

It is a multi-page Vite build rather than a single-page app: one HTML file per
page. A router would mean a 404 fallback on GitHub Pages and a site that needs
JavaScript to read.

## WebAssembly

Browsers and bundlers want different shapes, so there are two builds.

```sh
wasm-pack build crates/nounsql-wasm --target web     --out-dir pkg     --release  # site
wasm-pack build crates/nounsql-wasm --target bundler --out-dir pkg-npm --release  # npm
```

The TypeScript types for `compile()` are written by hand in
`crates/nounsql-wasm/src/lib.rs` as a `typescript_custom_section`. When the
intermediate representation changes, update them too — the
`serialized_ir_keys_are_stable` test fails when they drift apart.

## Releasing

```sh
bin/release patch           # 0.1.0 -> 0.1.1
bin/release minor           # 0.1.0 -> 0.2.0
bin/release major           # 0.1.0 -> 1.0.0
bin/release 0.2.0           # an exact version
bin/release patch --check   # run every check, change nothing
bin/release --npm-only      # publish the current version to npm only
```

The script runs formatting, lints, tests, the sample regeneration check and a
site build before it changes anything, then stops for confirmation before the
part that cannot be undone.

Publishing needs `CARGO_REGISTRY_TOKEN` — the variable cargo reads itself — and
`npm login`. `--skip-npm` leaves npm out.

crates are published in dependency order, waiting for each to appear in the
registry before the next. Versions already published are skipped, so a failed
release can be re-run.

The npm version follows crates.io. `--npm-only` does not bump the version, tag,
or push; it exists to let npm catch up when it has fallen behind.

Pushing the tag starts the release workflow, which builds binaries for macOS,
Linux and Windows and creates the GitHub release.
