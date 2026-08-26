# Guide

## Install

```sh
cargo install nounsql        # the compiler
cargo install nounsql-lsp    # language server
```

Binaries are also attached to each [release](https://github.com/misebox/nounsql/releases).
To compile from JavaScript, there is an npm package.

```sh
npm install nounsql
```

## Compile

```text
nounsql <command> [options] <input>
```

| Command | Prints |
|---|---|
| `check` | diagnostics only, then a count if nothing failed |
| `sql` | the DDL |
| `ir` | the resolved schema, with mixins and blueprints expanded and every name settled |
| `ast` | the syntax tree |

| Option | Effect |
|---|---|
| `--dialect <name>` | output target; defaults to `postgres` |
| `-o, --output <path>` | write to a file instead of stdout; parent directories are created |
| `--deny-warnings` | exit non-zero if anything was warned about |
| `--json` | `ir` only; print the intermediate representation as JSON |

Pass `-` as the input to read standard input. Diagnostics always go to standard
error, so piping the output never mixes the two.

```sh
nounsql sql schema.nsql -o schema.sql
cat schema.nsql | nounsql sql - > schema.sql
nounsql check schema.nsql --deny-warnings   # for CI
```

## Read the diagnostics

Errors arrive together. Because the grammar is one statement per line, a failed
line is skipped and parsing continues, so one run tells you everything it can.

```text
error: unknown attribute `foo`; expected type / null / default / on_update / comment
  --> schema.nsql:2:26
   2 |   column email type=text foo=1
     |                          ^^^
```

Warnings do not fail the run by default. A noun missing from the dictionary, or
a table that ended up without a primary key, shows up here. Add
`--deny-warnings` to make them fatal.

## In an editor

`nounsql-lsp` speaks the Language Server Protocol over stdio.

| Feature | What it does |
|---|---|
| Diagnostics | syntax, resolution and warnings, before you save |
| Go to definition | `use` to its mixin, `belongs_to` to its table, `apply_blueprint` to its blueprint |
| Hover | the table a noun resolves to |
| Outline | tables, mixins and blueprints |
| Completion | keywords, attributes, type names, mixin names and nouns |

The VS Code extension lives in `editors/vscode` and bundles syntax highlighting
with the language server client.

```sh
cd editors/vscode
npm install
npm run compile
```

If `nounsql-lsp` is not on your `PATH`, point `nounsql.server.path` at it.

## From JavaScript

The npm package [`nounsql`](https://www.npmjs.com/package/nounsql) is the
compiler built to WebAssembly. It runs in the browser and in Node, with no Rust
toolchain involved.

```ts
import init, { compile } from "nounsql";

await init();
const { sql, ir, diagnostics } = compile(source, "postgres");
```

`ir` is the resolved schema, typed. Generating ORM models means reading it.

```ts
for (const table of ir.tables) {
  // table.singular  "user"   for the model name
  // table.name      "users"  for the table
  // table.columns   in declaration order
  // table.foreignKeys / table.reverses  both directions of every relation
}
```

`nounsql ir --json` prints the same thing.
