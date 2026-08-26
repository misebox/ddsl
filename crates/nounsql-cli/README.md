# nounsql

**NounSQL is a DSL for Database Schema Design.**

A compiler from `.nsql` to SQL DDL.

```sh
cargo install nounsql
nounsql sql schema.nsql
```

| Command | Prints |
|---|---|
| `check` | diagnostics only |
| `sql` | the DDL |
| `ir` | the resolved schema |
| `ast` | the syntax tree |

| Option | Effect |
|---|---|
| `--dialect <name>` | output target |
| `-o, --output <path>` | write to a file instead of stdout |
| `--deny-warnings` | exit non-zero if anything was warned about |

Pass `-` as the input to read standard input.

```
nouns {
  user users "A person who signs in"
  post posts "Something a user wrote"
}

mixin base {
  column id type=serial
  pk id
}

table post {
  use base
  belongs_to user
  column title type=text
}
```

Documentation: https://misebox.github.io/nounsql
