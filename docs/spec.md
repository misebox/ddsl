# Specification

Version 0.1. Type names and reserved words come from the output target, which
the compiler picks with `--dialect`.

## Syntax

One statement per line. The keyword at the start of a line decides what the
statement is.

`#` starts a comment that runs to the end of the line. There are no comments
that span lines.

```
# what every table carries
mixin base {
  column id type=serial   # primary key
  pk id
}
```

Use `comment` for anything that should reach the DDL. `#` never leaves the
source.

### Statements

All of these go **inside** a `table` or `mixin` block. None of them can appear at
the top level. None of them take parentheses.

**They create columns.** The order they are written is the order of the columns
in the DDL.

| Statement | Form |
|---|---|
| Column | `column name key=value ...` |
| Reference | `belongs_to noun [fk="..."] [alias="..."] [comment="..."]` |
| One-to-one reference | `unique_belongs_to noun [fk="..."] [alias="..."] [comment="..."]` |

**They say something about the columns.**

| Statement | Form |
|---|---|
| Primary key | `pk column` / `pk [column, ...]` |
| Index | `index column [unique]` / `index [column, ...] [unique]` |

**They pull in a mixin and adjust it.**

| Statement | Form |
|---|---|
| Compose | `use mixin` |
| Remove | `except name` / `except [name, ...]` |
| Remove an index | `except index column` / `except index [column, ...]` |
| Replace attributes | `override column key=value ...` |

**The rest.**

| Statement | Form | Where |
|---|---|---|
| Table name | `name noun-expression` | `table` only |
| Table description | `comment "..."` | `table` only |
| Reverse, many | `has_many noun [via="..."] [alias="..."]` | `table` / `mixin` |
| Reverse, one-to-one | `has_one noun [via="..."] [alias="..."]` | `table` / `mixin` |

A list of one column does not need brackets.

### Order

Only the statements that create columns affect the result: their order is the
column order in the DDL. `use`, `except` and `override` are resolved in phases
(`use` expands, then `except`, then `override`), so where you write them makes
no difference.

Writing them in this order reads best.

```
table article {
  name noun(user, article)        # 1. what it is called, if not the identifier
  comment "A piece of writing"    # 2. what it is
  use base                        # 3. pull in
  except updated_at               # 4. adjust what was pulled in
  override id type=bigserial

  belongs_to author               # 5. create columns, in this order
  column title type=text

  pk id                           # 6. say something about them
  index title unique

  has_many review                 # 7. name the reverse side
}
```

### Macros

They expand into table definitions and are written at the top level.

| Macro | Form |
|---|---|
| Many-to-many | `associate(a, b [, name=noun] [, comment="..."])` |
| Apply a blueprint | `apply_blueprint(name, argument...)` |

### Functions

They appear where a value is expected.

| Function | Returns |
|---|---|
| `noun(a, b, ...)` | a compound noun |
| `singular(x)` / `plural(x)` | the singular or plural form |
| `desc(x)` | the third column of `nouns` |
| `eval(x)` | an expression the database evaluates |

### Blocks

```
table name { ... }
mixin name { ... }
blueprint name argument... { ... }
naming { ... }
constraints { ... }
nouns { ... }
```

### Values

| Kind | Form | Example |
|---|---|---|
| Keyword | a bare identifier | `true` / `serial` / `cascade` / `plural` |
| String | `"..."` | `"Primary key"` / `"idx_${table}_${columns}"` |
| List | `[a, b, ...]` | `[status, created_at]` |
| Database expression | `eval(...)` | `eval(now())` / `eval(gen_random_uuid())` |

`eval(...)` marks something the database evaluates, which is what separates
`default="pending"` from `default=eval(now())`.

### Names and quoting

Where a name is being declared — after `column` or `table` — write it bare.
Where a value is expected, a bare identifier is a reference to a noun, a type or
a keyword. A new name in that position is a string.

```
column email type=text            # email is a declaration; text refers to a type
belongs_to user fk="sender_id"    # user refers to a noun; sender_id is a new name
```

## column

```
column created_at type=timestamptz default=eval(now()) on_update=eval(now())
```

| Key | Value | Default |
|---|---|---|
| `type` | a type name from the output target | required |
| `null` | `true` / `false` | `constraints.null_default` |
| `default` | a literal or `eval(...)` | none |
| `on_update` | a literal or `eval(...)` | none |
| `comment` | a string | none |

Primary keys are `pk`, uniqueness is `index ... unique`. Neither is a column
attribute.

## name

A table's identifier and the name it carries in the database are two different
things. `table user { ... }` uses the identifier as the noun, which is what most
tables want. `name` separates them, and does one of two things depending on what
it is given.

**A noun expression sets the noun.** The table name is derived from it, and so is
anything else built out of that noun.

```
table user_role {
  name noun(user, role)
  belongs_to user
  belongs_to role
  pk [user_id, role_id]
}
```

`user_role` never has to be registered in `nouns`: the noun is composed from two
that are. The identifier is only a handle for referring to this table elsewhere,
so it must not collide with a registered noun.

One table's name can be built from another's, which is how a `blueprint` names
the group it generates. Names that refer to each other in a circle are an error.

```
blueprint approvable target {
  table t_approval {
    name noun(target, approval)     # post_approvals
  }
  table t_step {
    name noun(t_approval, step)     # post_approval_steps
  }
}
```

**A string sets the table name and nothing else.** The noun stays the
identifier, so foreign keys pointing at the table are still built from it.

```
table customer {
  name "M_CUSTOMER"     # the table is M_CUSTOMER
  ...                   # a reference to it still gives customer_id
}
```

## Opting out of the generated names

Every generated name can be replaced, which is what makes it possible to
describe a database whose names were decided somewhere else.

| Name | How to set it |
|---|---|
| Table | `name "..."` |
| Column | written out in `column` |
| Foreign key column | `fk="..."` on the reference |
| Relation name | `alias="..."` on either side |
| Index | `naming.index` and `naming.unique_index` |

The one that has no per-item override is the index name: it comes from a
template, so an existing convention goes in `naming` and applies to the whole
schema.

```
naming {
  index = "IX_${table}_${columns}"
  unique_index = "UX_${table}_${columns}"
}

table customer {
  name "M_CUSTOMER"
  column CUST_CD   type=char
  column CUST_NAME type=text
  pk CUST_CD
}

table order {
  name "T_ORDER"
  column ORDER_NO type=char
  belongs_to customer fk="CUST_CD"
  pk ORDER_NO
}
```

Identifiers that are not plain lowercase are quoted in the output, so a schema
in upper case survives unchanged.

## pk

One per table, written once the columns exist.

```
table user {
  column id type=serial
  pk id
}
```

A composite key lists its columns. Join tables usually want one.

```
table user_role {
  belongs_to user
  belongs_to role
  pk [user_id, role_id]
}
```

## index

As many as the table needs. `unique` makes it a uniqueness constraint.

```
table post {
  column status type=text
  column title  type=text
  index title unique
  index [status, created_at]
}
```

Foreign key columns get an index automatically while
`constraints.foreign_key_index` is true. Index names come from `naming.index`,
or `naming.unique_index` when `unique` is given.

## mixin

Columns, indexes and constraints that stay inside one table. `use` composes.

```
mixin identity {
  column id type=serial
  pk id
}

mixin timestamps {
  column created_at type=timestamptz default=eval(now())
  column updated_at type=timestamptz default=eval(now()) on_update=eval(now())
}

mixin base {
  use identity
  use timestamps
}
```

## Adjusting what a mixin brought

- `override column key=value...` replaces the listed attributes. Anything not
  listed keeps the mixin's value.
- `except name` drops a column.
- `except index column` drops an index, named by its columns rather than by its
  index name — the name depends on the table and is not known where the mixin is
  written.

```
except  updated_at
except  index published_at
override note       type=text null=false
override created_at default=eval(now())
```

Resolution runs `use` expansion, then `except`, then `override`.

## Relations

The side holding the foreign key writes `belongs_to` or `unique_belongs_to`. The
side being pointed at writes `has_many` or `has_one`.

- `belongs_to noun` creates a foreign key column. Its name, referential actions
  and index follow the configuration. **The type comes from the referenced
  primary key**, so it is never written: a `smallserial`, `serial` or `bigserial`
  key gives a `smallint`, `integer` or `bigint` column.
- `unique_belongs_to noun` does the same and adds a uniqueness constraint.
- `has_many noun` and `has_one noun` name the reverse side. They create no
  column and no constraint, so they never appear in the DDL; ORM generation uses
  them.
- `associate(a, b)` is written at the top level. It names the table with
  `noun(a, b)`, gives it two foreign key columns and a `pk [a_id, b_id]`. `name=`
  sets the noun and `comment=` the description. Anything more — a comment on one
  of the foreign keys, an extra column — means writing an ordinary table with two
  `belongs_to`.

```
associate(user, post)                                     # user_posts
associate(user, post, name=favorite, comment="A like")    # favorites
```

### Attributes

| Key | On | Meaning | Default |
|---|---|---|---|
| `fk` | `belongs_to` / `unique_belongs_to` | the foreign key column name | `naming.foreign_key` |
| `comment` | `belongs_to` / `unique_belongs_to` | that column's comment | none |
| `alias` | all four | the relation name on this side; a string or `noun(...)` | `naming.belongs_to` / `naming.has_many` / `naming.has_one` |
| `via` | `has_many` / `has_one` | which foreign key this is the reverse of | the only one, if there is only one |

`has_many` and `has_one` create no column, so they take no `comment=`.

```
table message {
  belongs_to user fk="sender_id"   alias="sender"   comment="Who sent it"
  belongs_to user fk="receiver_id" alias="receiver" comment="Who received it"
}

table user {
  has_many message via="sender_id"   alias="sent_messages"
  has_many message via="receiver_id" alias="received_messages"
}
```

Both `has_many` and `has_one` can be left out; the reverse relation exists under
its default name either way.

Foreign key columns are `NOT NULL`. An optional reference says so.

```
belongs_to category comment="Where it is filed"
override category_id null=true
```

### Matching

`has_many` and `has_one` attach to a foreign key that points at this table from
the named noun's table. When more than one does, `via=` picks which.

## nouns

The dictionary. Each line is a singular form, a plural form and a description.

```
nouns {
  user     users      "A person who signs in"
  category categories "A grouping for products"
  person   people     "Someone responsible"
  child    children   "A nested element"
}
```

Table names, foreign key columns and relation names are all built from these.
`singular()` and `plural()` resolve here; a noun that is missing falls back to
regular inflection and produces a warning.

The block carries two jobs.

**Irregular forms become part of the definition.** Regular inflection covers a
lot, but not everything.

| Noun | Regular rule | Correct |
|---|---|---|
| `person` | `persons` | `people` |
| `child` | `childs` | `children` |
| `analysis` | `analysises` | `analyses` |
| `datum` | `datums` | `data` |

Without an entry those names go through, warned about but generated. With one,
the conversion lives in the source rather than in a comment or a commit message.

**The glossary comes along for free.** The third column becomes the DDL
`COMMENT`. What a table means to the business ends up in the same file as the
schema, so writing the schema is what keeps the glossary current.

An entry with no `table` of its own is fine. Nouns that only ever appear as part
of a name — `item`, `history` — are registered too.

Anything without singular and plural forms — a modifier like `sent` — does not
belong here. Write it as a string.

## Compound nouns

`noun(a, b, ...)` builds a noun out of several parts. It returns a noun, not a
string, so it has both a singular and a plural form.

- only the last part takes the number the context asks for
- everything before it is singular
- string parts never inflect

```
noun("sent", message)   # sent_message  / sent_messages
noun(order, item)       # order_item    / order_items
```

The context decides.

```
has_many message alias=noun("sent", message)   # sent_messages
has_one  profile alias=noun("main", profile)   # main_profile
```

Parts are joined with `naming.noun_separator`.

## comment

Things with a block use a statement; things without use an attribute.

| On | How |
|---|---|
| A table | `comment "..."` inside `table` |
| A column | `column ... comment="..."` |
| A foreign key column | `belongs_to ... comment="..."` |
| A table from `associate` | `associate(a, b, comment="...")` |

```
table user {
  comment "A person who signs in"
  column email type=text comment="Used to sign in"
}
```

The third column of `nouns` also becomes a table comment. A `comment` inside the
`table` wins.

`mixin` and `blueprint` never become tables, so they take no `comment`. Use `#`
for notes that belong to the source.

### Interpolation

Comment strings take `${...}`. What can be referenced is nouns, including a
blueprint's parameters and the tables it declares.

```
nouns {
  post    posts     "Something a user wrote"
  history histories "A record of a past state"
}

blueprint audited target {
  table t_history {
    name noun(target, history)
    comment "Past states of a ${target}"
    column id type=serial
    pk id
  }
}

apply_blueprint(audited, post)
```

`post_histories` is generated, and its comment reads "Past states of a post".

## Configuration

Two blocks, `naming` and `constraints`. Both may be left out; the keys that are
written override the defaults.

`naming` holds string-building rules and `constraints` holds defaults for
constraints. How a mixin's own columns behave belongs in the mixin.

### naming

Leaving it out is the same as writing this.

```
naming {
  table_name = plural
  primary_key = "id"
  foreign_key = "${singular(table)}_id"
  index = "idx_${table}_${columns}"
  unique_index = "uq_${table}_${columns}"
  column_separator = "_"
  noun_separator = "_"
  belongs_to = "${singular(table)}"
  has_many = "${plural(table)}"
  has_one = "${singular(table)}"
}
```

`${columns}` joins several columns with `column_separator`; compound nouns join
with `noun_separator`.

For `belongs_to` the `table` variable is the referenced noun; for `has_many` and
`has_one` it is the noun on the side that holds the foreign key.

### constraints

Leaving it out is the same as writing this.

```
constraints {
  null_default = false
  on_delete_default = cascade
  on_update_default = cascade
  foreign_key_index = true
}
```

`null_default` is what a `column` means when `null=` is left out.
`on_delete_default` and `on_update_default` take `cascade`, `restrict`,
`set_null` or `no_action`.

## blueprint

It gives one name to **a structure that only exists once several tables point at
each other**.

An approval flow is a request, its stages and their notes: three tables that mean
nothing apart. Written as three separate `table` blocks, nothing in the source
says they are one mechanism, and attaching the same mechanism to another noun
means copying them by hand.

`mixin` stays inside one table. `belongs_to` joins two. `blueprint` covers a
wider group, and keeping the three as separate constructs is what makes the reach
of a name readable.

```
blueprint approvable target {
  table t_approval {
    name noun(target, approval)
    comment "A request to publish a ${target}"
    use primary_key
    belongs_to target
    column status type=text
  }

  table t_step {
    name noun(t_approval, step)
    comment "One stage of approving a ${target}"
    use primary_key
    belongs_to t_approval
    column position type=integer
  }
}

apply_blueprint(approvable, post)
```

### Arguments

Every argument is a noun, and no type is written. A noun passed in must be
registered in `nouns`, so that `desc()` can reach its description.

### Resolving names

References are written bare and resolved in this order.

1. a parameter
2. a table declared in this blueprint
3. a noun

A parameter that collides with a noun is an error.

## Validation

- `type=` names a type the output target has
- `override` names a column the mixin defined
- foreign key column types match the referenced primary key
- no duplicate table or column names
- a name that collides with a reserved word of the output target (warning)
- `use` does not form a cycle
- a blueprint parameter, or the identifier of a table that writes `name`, does not collide with a noun
- table names do not refer to each other in a circle
- a noun fell back to regular inflection (warning); as a blueprint argument, an error
- a `${...}` in a comment refers to a noun that has a description
- `has_many` and `has_one` have a foreign key to attach to
- `via=` is given when more than one foreign key would match
- `has_many` is not used where the reference is one-to-one, or `has_one` where it is not
- a relation name does not collide with a column or another relation on the same table
- a table has a primary key (warning); removing one with `except` shows up here
- `except` did not remove a foreign key column and silently drop the relation (warning)

### Indexes

- the column types suit the index kind (`gin` and `gist` are limited; everything else is btree)
- no two indexes cover the same columns
- no index duplicates the uniqueness constraint `unique_belongs_to` creates
- a foreign key column has an index when `foreign_key_index = false` turned the automatic one off (warning)

## Every construct

```
naming {
  table_name = plural
  primary_key = "id"
  foreign_key = "${singular(table)}_id"
  index = "idx_${table}_${columns}"
  unique_index = "uq_${table}_${columns}"
  column_separator = "_"
  noun_separator = "_"
}

constraints {
  null_default = false
  on_delete_default = cascade
  on_update_default = cascade
  foreign_key_index = true
}

nouns {
  user       users       "A person who signs in"
  category   categories  "A grouping for products"
  product    products    "Something for sale"
  order      orders      "A purchase a customer placed"
  item       items       "One line of something"
  order_item order_items "One line of an order"
  post       posts       "Something a user wrote"
  profile    profiles    "Extra details about one user"
  history    histories   "A record of a past state"
  favorite   favorites   "A user liking a post"
}

mixin primary_key {
  column id type=serial comment="Primary key"
  pk id
}

mixin timestamps {
  column created_at type=timestamptz default=eval(now()) comment="When the row was created"
  column updated_at type=timestamptz default=eval(now()) on_update=eval(now()) comment="When it last changed"
}

mixin describable {
  column note type=varchar null=true comment="Anything worth noting"
}

mixin base {
  use primary_key
  use timestamps
}

mixin publishable {
  column status       type=text                 comment="Draft, scheduled or published"
  column published_at type=timestamptz null=true comment="When it went public"
  index status
  index published_at
}

blueprint audited target {
  table t_history {
    name noun(target, history)
    comment "Past states of a ${target}"
    use primary_key
    belongs_to target comment="What changed"
    belongs_to user   comment="Who changed it"
    column version     type=integer                        comment="Which revision"
    column snapshot    type=jsonb                          comment="The row as it stood"
    column recorded_at type=timestamptz default=eval(now()) comment="When it was recorded"
  }
}

table user {
  comment "A person who signs in"
  use base
  column email type=text comment="Used to sign in"
  column name  type=text comment="Shown to other users"
  index email unique
  has_many post
  has_one profile
}

table category {
  use base
  use describable
  except updated_at
  override note       type=text null=false
  override created_at default=eval(now())
  column name type=text comment="Shown in navigation"
  index name unique
}

table product {
  use base
  column name  type=text    comment="Shown to shoppers"
  column price type=integer comment="Excluding tax, in yen"
}

table order {
  use base
  belongs_to user comment="Who placed it"
}

table order_item {
  use base
  belongs_to order   comment="Which order"
  belongs_to product comment="What was bought"
  column quantity type=integer comment="How many"
}

table post {
  use base
  use publishable
  except index published_at
  belongs_to user     comment="Who wrote it"
  belongs_to category comment="Where it is filed"
  column title type=text comment="Shown in listings"
  index [status, created_at] unique
}

table profile {
  use base
  unique_belongs_to user comment="Whose profile this is"
  column bio type=text null=true comment="Free text the user wrote"
}

associate(user, post, name=favorite, comment="A user liking a post")
apply_blueprint(audited, post)
```
