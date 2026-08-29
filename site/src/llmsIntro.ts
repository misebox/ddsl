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
  account "A person who signs in"
  post    "An article written by an account"
}

mixin timestamps {
  column created_at type=timestamptz default=eval(now())
  column updated_at type=timestamptz default=eval(now())
}

table account {
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
  belongs_to account
  use timestamps
  pk id
}
\`\`\`

That produces \`CREATE TABLE account\`, \`CREATE TABLE post\` with an
\`account_id\` foreign key and an index on it, \`uq_account_email\`, and a
\`COMMENT\` on each table taken from the noun's description.

Rules a generator needs to know:

- One statement per line. There are no semicolons and no line continuations.
- A \`table\` is named after a noun. Table names are singular by default;
  \`naming { table_name = plural }\` switches the whole schema. Write \`name\`
  inside the block to set one table's name directly.
- Every noun used anywhere must be registered in \`nouns\`. A line is an
  identifier, then any of \`singular=\` / \`plural=\` / \`short=\`, then the
  description. Left out, \`singular\` is the identifier, \`plural\` is its
  regular inflection, and \`short\` is the singular. Write \`plural=\` when the
  regular rule is wrong for the word or for the domain.
- The identifier never appears in the output. It only seeds the word forms.
- \`belongs_to\` makes the foreign key column; \`has_many\` / \`has_one\` only
  declare the reverse direction and emit nothing.
- Avoid nouns that are reserved words (\`user\`, \`order\`, \`group\`); they come
  out quoted.
- Comments start with \`#\` and run to the end of the line.
`;
