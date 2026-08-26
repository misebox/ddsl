# DB設計用DSL 仕様 v0.1

対象DBは PostgreSQL。型名と予約語は出力ターゲットが定める。ターゲットはコンパイラの `--dialect` で選ぶ。既定は `postgres`。

## 構文

1行1文。行頭のキーワードで文種が決まる。

### 宣言文（括弧なし）

| 文 | 記法 |
|---|---|
| カラム宣言 | `column 名前 キー=値 ...` |
| 主キー | `pk 列` / `pk [列, ...]` |
| index | `index 列 [unique]` / `index [列, ...] [unique]` |
| mixin合成 | `use mixin名` |
| 上書き | `override 列 キー=値 ...` |
| 除外 | `except 名前` / `except [名前, ...]` |
| index除外 | `except index 列` / `except index [列, ...]` |
| 従属 | `belongs_to entity` |
| 1対1従属 | `unique_belongs_to entity` |
| 局所束縛（blueprint内） | `let 名前 = 式` |

列リストは単一なら角括弧を省略できる。

### マクロ

テーブル定義に展開する。グローバルスコープに書く。

| マクロ | 記法 |
|---|---|
| 多対多 | `associate(a, b)` |
| blueprint適用 | `apply_blueprint(名前, 引数...)` |

### 関数（括弧あり）

値の位置に書く。

| 関数 | 戻り値 |
|---|---|
| `singular(x)` / `plural(x)` | 命名辞書で解決した単数形 / 複数形 |
| `name_join(a, b)` | 結合したテーブル名 |
| `eval(x)` | DB側で評価される式 |

### ブロック

```
table 名前 [comment="..."] { ... }
mixin 名前 [comment="..."] { ... }
blueprint 名前 引数... [comment="..."] { ... }
naming { ... }
constraints { ... }
entities { ... }
```

### 値の種類

| 種類 | 記法 | 例 |
|---|---|---|
| キーワード | 裸の識別子 | `true` / `serial` / `cascade` / `plural` |
| 文字列 | `"..."` | `"ユーザー"` / `"idx_${table}_${columns}"` |
| リスト | `[a, b, ...]` | `[serial, integer, text]` |
| DB側の式 | `eval(...)` | `eval(now())` / `eval(current_user)` |

## column

```
column created_at type=timestamptz default=eval(now()) on_update=eval(now())
```

| キー | 値 | 既定 |
|---|---|---|
| `type` | 出力ターゲットの型名 | 必須 |
| `null` | `true` / `false` | `constraints.null_default` |
| `default` | リテラル / `eval(...)` | 無し |
| `on_update` | リテラル / `eval(...)` | 無し |
| `comment` | 文字列 | 無し |

主キーは `pk`、一意性は `index ... unique` で表現する。

## pk

```
pk id
pk [user_id, role_id]
```

## index

```
index status
index email unique
index [status, created_at] unique
```

FKのindexは `foreign_key_index = true` により自動生成する。
index名は `naming.index` から生成する。`unique` 指定時は `naming.unique_index` から生成する。

## mixin

1テーブル内のカラム・index・制約を対象とする。`use` で他mixinを合成する。

```
mixin primary_key {
  column id type=serial
  pk id
}

mixin timestamps {
  column created_at type=timestamptz default=eval(now())
  column updated_at type=timestamptz default=eval(now()) on_update=eval(now())
}

mixin base {
  use primary_key
  use timestamps
}

mixin publishable {
  column status       type=text
  column published_at type=timestamptz null=true

  index status
  index published_at
}
```

## オーバーライド・除外

- `override 列 キー=値...`：指定したキーをmixinの定義に上書きする。指定しなかったキーはmixin側の値が残る。
- `except 名前`：mixin側のカラムを除外する。
- `except index 列`：mixin側のindexを除外する。指定は列で行う。

```
override note       type=text null=false
override created_at default=eval(now())
except  updated_at
except  index published_at
```

適用順序は `use` 展開 → `except` → `override`。

## テーブル間構造

- `belongs_to entity`：テーブル定義内に書く。FK列名・型・on_delete・indexは config に従う。参照先PKが `smallserial` / `serial` / `bigserial` の場合、FK列の型は `smallint` / `integer` / `bigint` とする。
- `unique_belongs_to entity`：FKを持つ側のテーブルに書く。one_to_one。FK列に一意制約が付く。
- `associate(a, b)`：グローバルスコープに書く。FK2列のみを持つ中間テーブルを生成し、`pk [a_id, b_id]` を付ける。中間テーブルにカラムを追加する場合は、通常のテーブルに `belongs_to` を2つ書く。

## 命名辞書

`entities` ブロックに `singular plural comment` の順で列挙する。`singular()` / `plural()` はこの辞書で解決する。辞書に無い語は規則変化にフォールバックする。

`entities` は命名辞書であり、単複を引くためだけの語を登録してよい。`table` 宣言の無いエントリはエラーにしない。

複合語の複数形は `name_join()` で組み立てる。

## comment

`entities` の第3列のほか、`table` / `blueprint` / `mixin` / `column` に `comment="..."` を書ける。

```
table user comment="ユーザー" {
  column email type=text comment="ログインID"
}
```

`entities` と `table` の両方に記述がある場合は `table` 側を採用する。

## config

設定ブロックは `naming` / `constraints`。各ブロックは省略可。書かれたキーが既定値を上書きする。

`naming` は文字列組み立て規則、`constraints` は制約の既定値を持つ。mixin自身のカラムの挙動は mixin 定義内に書く。

### `naming`

省略時は以下を書いたものとして扱う。

```
naming {
  table_name = plural
  primary_key = "id"
  foreign_key = "${singular(table)}_id"
  join_table = "${singular(a)}_${singular(b)}"
  name_join = "${singular(a)}_${plural(b)}"
  index = "idx_${table}_${columns}"
  unique_index = "uq_${table}_${columns}"
  column_separator = "_"
}
```

`${columns}` は複数列のとき `column_separator` で連結する。

### `constraints`

省略時は以下を書いたものとして扱う。

```
constraints {
  null_default = false
  on_delete_default = cascade
  on_update_default = cascade
  foreign_key_index = true
}
```

`null_default` は `column` の `null=` を省略したときの値。
`on_delete_default` / `on_update_default` に書ける値は `cascade` / `restrict` / `set_null` / `no_action`。

## blueprint

複数テーブルにまたがる構造を定義する。

```
blueprint approvable target comment="承認フロー" {
  let t_approval = name_join(target, approval)
  let t_step     = name_join(t_approval, step)

  table t_approval {
    use primary_key
    belongs_to target
    column status type=text
  }

  table t_step {
    use primary_key
    belongs_to t_approval
    column step type=integer
  }
}

apply_blueprint(approvable, post)
```

### 引数

引数はすべて entity。型注釈は書かない。渡す語は `entities` に登録されていること。

### 局所束縛 `let`

`let 名前 = 式` でテーブル名を束縛する。再代入は不可。

### 名前解決

参照は裸の識別子で書く。次の順で解決する。

1. 仮引数
2. `let` 束縛
3. 実エンティティ

仮引数名・`let` 束縛名が実エンティティ名と衝突した場合はエラー。

### `name_join`

`naming.name_join` の規則で最終テーブル名を返す。`table_name` は適用しない。

`name_join(post, approval)` → `post_approvals`

## バリデーション

- `type=` の値が出力ターゲットの型名にあるか
- `override` 対象のカラムがmixin側に存在するか
- `except` でprimary keyやFK等の必須カラムを除外していないか
- FK列の型が参照先PKの型と一致しているか
- テーブル名・列名の重複
- 出力ターゲットの予約語との衝突（警告）
- `use` の循環参照
- blueprint の仮引数名・`let` 束縛名と実エンティティ名の衝突
- 単複が辞書に無く規則変化で解決された場合は警告

## indexのバリデーション

- 列の型とindex種別の整合性（gin/gistは対象型限定、それ以外はbtree）
- 同一列組み合わせのindexの重複
- `unique_belongs_to` が生成する一意制約と同じ列組み合わせのindexの重複
- `foreign_key_index = false` にしたFK列にindexが無い場合は警告

## 全構文を使ったサンプル

```
naming {
  table_name = plural
  primary_key = "id"
  foreign_key = "${singular(table)}_id"
  join_table = "${singular(a)}_${singular(b)}"
  name_join = "${singular(a)}_${plural(b)}"
  index = "idx_${table}_${columns}"
  unique_index = "uq_${table}_${columns}"
  column_separator = "_"
}

constraints {
  null_default = false
  on_delete_default = cascade
  on_update_default = cascade
  foreign_key_index = true
}

entities {
  user       users       "ユーザー"
  category   categories  "商品カテゴリ"
  product    products    "商品"
  order      orders      "注文"
  item       items       "明細"
  order_item order_items "注文明細"
  post       posts       "投稿"
  profile    profiles    "プロフィール"
  history    histories   "履歴"
}

mixin primary_key {
  column id type=serial
  pk id
}

mixin timestamps {
  column created_at type=timestamptz default=eval(now())
  column updated_at type=timestamptz default=eval(now()) on_update=eval(now())
}

mixin describable {
  column note type=varchar null=true
}

mixin base {
  use primary_key
  use timestamps
}

mixin publishable {
  column status       type=text
  column published_at type=timestamptz null=true

  index status
  index published_at
}

blueprint audited target comment="変更履歴" {
  let t_history = name_join(target, history)

  table t_history {
    use primary_key
    belongs_to target
    belongs_to user
    column version     type=integer
    column snapshot    type=jsonb
    column recorded_at type=timestamptz default=eval(now())
  }
}

table user comment="ユーザー" {
  use base
  column email type=text comment="ログインID"
  column name  type=text

  index email unique
}

table category {
  use base
  use describable
  override note       type=text null=false
  override created_at default=eval(now())
  except updated_at
  column name type=text

  index name unique
}

table product {
  use base
  column name  type=text
  column price type=integer
}

table order {
  use base
  belongs_to user
}

table order_item {
  use base
  belongs_to order
  belongs_to product
  column quantity type=integer
}

table post {
  use base
  use publishable
  except index published_at
  belongs_to user
  belongs_to category
  column title type=text

  index [status, created_at] unique
}

table profile {
  use base
  unique_belongs_to user
  column bio type=text null=true
}

associate(user, post)
apply_blueprint(audited, post)
```
