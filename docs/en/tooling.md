# How it works

What the compiler does between your source and the DDL it prints. The
[guide](guide.md) covers running it; this page is about the machinery.

## The pipeline

```text
source
 → lexer      newlines terminate statements; eval(...) is lifted whole by
              matching parentheses, so SQL never reaches the DSL lexer
 → parser     hand-written recursive descent; recovers at line boundaries
              so one run reports every error it can find
 → resolver   expand blueprints → expand mixins → resolve relations → derive names
 → ir         mixins and blueprints are gone; every name is settled
 → codegen    DDL for the selected target
```

Each stage hands the next one a smaller problem. By the time codegen runs there
are no conventions left to apply — only tables, columns and constraints with
final names.

## Resolution order

`use` expands where it is written, then `except` removes, then `override`
replaces. Fixing the order means the result does not depend on where in the
block you wrote those statements.

Column order is the exception, and deliberately so: columns appear in the DDL in
the order they were declared, because that order is something you chose.

`belongs_to` reserves its foreign key column at the point of declaration but
leaves the type blank. The type is filled in once the referenced table's primary
key is known. Without that two-step, foreign keys would pile up at the end of
every table instead of sitting where you wrote them.

## Names

Every generated name goes through one of the `naming` templates, and every
template resolves nouns through the `nouns` dictionary. There is no second path:
if a name appears in the output, it came from a template and a noun.

`noun(a, b, ...)` builds a compound noun rather than a string. Only the last
element takes the number the context asks for, so the same expression yields
`sent_message` under `has_one` and `sent_messages` under `has_many`.

## Diagnostics

Errors do not stop the run. Because the grammar is one statement per line, the
parser can skip a failed line and pick up at the next one, so a single run
reports everything it can reach.

Warnings cover the cases where the compiler can produce something but probably
should not: a noun that fell back to regular inflection, a table with no primary
key, a foreign key column removed by `except`.

## WebAssembly

`nounsql-core` depends on `indexmap` and nothing else, and performs no I/O, so it
compiles to `wasm32` unchanged. The [playground](playground.html) and the
[npm package](https://www.npmjs.com/package/nounsql) both run that build — the
same lexer, resolver and code generator as the CLI.

The syntax highlighting on this site comes from the same place. Code blocks are
coloured by the compiler's own lexer rather than by a second implementation in
JavaScript, so the highlighting cannot drift from the grammar.

## Grammar for constrained decoding

[`nounsql.gbnf`](nounsql.gbnf) is a GBNF grammar for llama.cpp-style constrained
decoding. It keeps a language model inside the syntax when generating NounSQL.
