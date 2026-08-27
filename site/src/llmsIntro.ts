/**
 * llms.txt の頭に置く紹介文。llms.txt と llms-full.txt が共有する。
 *
 * 相手は人ではなく、これだけを読んで NounSQL を書こうとするモデルなので、
 * 何ができるかではなく、何がどう書けるかを先に出す。
 */
export const LLMS_INTRO = `# NounSQL

> NounSQL is a schema definition language that compiles to SQL DDL. A schema is
> written in terms of nouns; table names, foreign key columns, index names and
> \`COMMENT\`s are derived from those nouns instead of being typed out on every
> table. PostgreSQL is the only dialect emitted today.

- Repository: https://github.com/misebox/nounsql
- Rust crate: https://crates.io/crates/nounsql
- npm package (WebAssembly): https://www.npmjs.com/package/nounsql
- Licence: MIT OR Apache-2.0

## At a glance

\`\`\`
nouns {
  user users "A person who signs in"
  post posts "An article written by a user"
}

mixin timestamps {
  column created_at type=timestamptz default=eval(now())
  column updated_at type=timestamptz default=eval(now())
}

table user {
  column id type=serial
  column email type=text
  use timestamps
  pk id
  index email unique
  has_many post
}

table post {
  column id type=serial
  column title type=text
  belongs_to user
  use timestamps
  pk id
}
\`\`\`

That produces \`CREATE TABLE users\`, \`CREATE TABLE posts\` with a
\`user_id\` foreign key and an index on it, \`uq_users_email\`, and a
\`COMMENT\` on each table taken from the third column of \`nouns\`.

Rules a generator needs to know:

- One statement per line. There are no semicolons and no line continuations.
- A \`table\` is named after a noun, not after the table. \`table user\` produces
  \`users\`. Write \`name\` inside the block to set the table name directly.
- Nouns used anywhere must be registered in \`nouns\` with singular, plural and
  description. Plurals are written, never guessed.
- \`belongs_to\` makes the foreign key column; \`has_many\` / \`has_one\` only
  declare the reverse direction and emit nothing.
- Comments start with \`#\` and run to the end of the line.
`;
