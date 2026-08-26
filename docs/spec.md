# DB設計用DSL 仕様 v0.1

対象DBは PostgreSQL。型名と予約語は出力ターゲットが定める。ターゲットはコンパイラの `--dialect` で選ぶ。既定は `postgres`。

## 構文

1行1文。行頭のキーワードで文種が決まる。

`#` から行末まではコメント。行をまたぐコメントは無い。

```
# 共通の列をまとめる
mixin base {
  column id type=serial   # 主キー
  pk id
}
```

DDL に残したい説明は `comment` で書く。`#` はソースにしか残らない。

### 宣言文

すべて `table` または `mixin` ブロックの**中**に書く。グローバルスコープには書けない。括弧は付けない。

**列を作る。** 書いた順がそのまま DDL の列順になる。

| 文 | 記法 |
|---|---|
| カラム | `column 名前 キー=値 ...` |
| 従属 | `belongs_to 名詞 [fk="..."] [alias="..."] [comment="..."]` |
| 1対1従属 | `unique_belongs_to 名詞 [fk="..."] [alias="..."] [comment="..."]` |

**揃った列について宣言する。**

| 文 | 記法 |
|---|---|
| 主キー | `pk 列` / `pk [列, ...]` |
| index | `index 列 [unique]` / `index [列, ...] [unique]` |

**mixin を取り込み、調整する。**

| 文 | 記法 |
|---|---|
| 合成 | `use mixin名` |
| 除外 | `except 名前` / `except [名前, ...]` |
| index除外 | `except index 列` / `except index [列, ...]` |
| 上書き | `override 列 キー=値 ...` |

**その他。**

| 文 | 記法 | 書ける場所 |
|---|---|---|
| テーブルの説明 | `comment "..."` | `table` のみ |
| 逆参照（多） | `has_many 名詞 [via="..."] [alias="..."]` | `table` / `mixin` |
| 逆参照（1対1） | `has_one 名詞 [via="..."] [alias="..."]` | `table` / `mixin` |
| 局所束縛 | `let 名前 = 式` | `blueprint` の直下 |

列リストは単一なら角括弧を省略できる。

### 書く順序

列を作る文の並びだけが結果に効く。書いた順がそのまま DDL の列順になる。`use` / `except` / `override` はフェーズ順（`use` 展開 → `except` → `override`）に解決されるので、書いた位置に依らない。

読みやすさのために次の順を推奨する。

```
table article {
  comment "記事"              # 1. このテーブルが何か
  use base                    # 2. 取り込む
  except updated_at           # 3. 取り込んだものを調整する
  override id type=bigserial

  belongs_to author           # 4. 列を作る。この順が列順になる
  column title type=text

  pk id                       # 5. 揃った列について宣言する
  index title unique

  has_many review             # 6. 逆参照に名前を付ける
}
```

### 名前とクォート

宣言位置（`column` や `table` の直後）には新しい名前を裸で書く。値位置（`キー=値` の右辺）の裸の識別子は参照（名詞・型名・キーワード）を表す。値位置に新しい名前を書くときは文字列にする。

```
column email type=text            # email は宣言位置。text は型名の参照
belongs_to user fk="sender_id"    # user は名詞の参照。sender_id は新しい名前
```

### マクロ

テーブル定義に展開する。グローバルスコープに書く。

| マクロ | 記法 |
|---|---|
| 多対多 | `associate(a, b [, name=名詞] [, comment="..."])` |
| blueprint適用 | `apply_blueprint(名前, 引数...)` |

### 関数（括弧あり）

値の位置に書く。

| 関数 | 戻り値 |
|---|---|
| `noun(a, b, ...)` | 複合名詞 |
| `singular(x)` / `plural(x)` | 単数形 / 複数形の文字列 |
| `desc(x)` | `nouns` の第3列（説明） |
| `eval(x)` | DB側で評価される式 |

### ブロック

```
table 名前 { ... }
mixin 名前 { ... }
blueprint 名前 引数... { ... }
naming { ... }
constraints { ... }
nouns { ... }
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

`pk` は1テーブルに1つ。列が揃ってから書く。

```
table user {
  column id type=serial
  pk id
}
```

複合主キーは列を並べる。中間テーブルでよく使う。

```
table user_role {
  belongs_to user
  belongs_to role
  pk [user_id, role_id]
}
```

## index

1テーブルにいくつでも書ける。`unique` を付けると一意制約になる。

```
table post {
  column status type=text
  column title  type=text
  index title unique
  index [status, created_at]
}
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

FKを持つ側に `belongs_to` / `unique_belongs_to` を書く。参照される側に `has_many` / `has_one` を書く。

- `belongs_to 名詞`：FK列を生成する。列名・on_delete・indexは config に従う。**型は参照先の主キーから決まる**ので書かない。参照先PKが `smallserial` / `serial` / `bigserial` の場合、FK列の型は `smallint` / `integer` / `bigint` になる。
- `unique_belongs_to 名詞`：`belongs_to` と同じくFK列を生成し、一意制約を付ける。one_to_one。
- `has_many 名詞` / `has_one 名詞`：逆参照に名前を付ける。カラムも制約も生成しないため DDL には現れない。ORM のモデル生成でのみ使う。
- `associate(a, b)`：グローバルスコープに書く。`noun(a, b)` から名前を決め、FK2列のみを持つ中間テーブルを生成し、`pk [a_id, b_id]` を付ける。`name=` でテーブルの名詞を、`comment=` でコメントを指定できる。FK列のコメントなど、これ以上のものが要る場合は、通常のテーブルに `belongs_to` を2つ書く。

```
associate(user, post)                                  # user_posts
associate(user, post, name=favorite, comment="いいね")  # favorites
```

### 属性

| キー | 書ける文 | 意味 | 既定値 |
|---|---|---|---|
| `fk` | `belongs_to` / `unique_belongs_to` | FK列名 | `naming.foreign_key` |
| `comment` | `belongs_to` / `unique_belongs_to` | FK列のコメント | 無し |
| `alias` | 4文すべて | その側の関連名。文字列か `noun(...)` | `naming.belongs_to` / `naming.has_many` / `naming.has_one` |
| `via` | `has_many` / `has_one` | 対応するFK列名 | 参照が1本ならその列 |

`has_many` / `has_one` は列を作らないので `comment=` は書けない。

```
table message {
  belongs_to user fk="sender_id"   alias="sender"
  belongs_to user fk="receiver_id" alias="receiver"
}

table user {
  has_many message via="sender_id"   alias="sent_messages"
  has_many message via="receiver_id" alias="received_messages"
}
```

`has_many` / `has_one` は省略できる。省略した場合も逆参照は既定の名前で存在する。

FK列は既定で `NOT NULL` になる。任意の参照にしたいときは `override` で緩める。

```
belongs_to category comment="所属カテゴリ"
override category_id null=true
```

### 対応付け

`has_many` / `has_one` は、その名詞のテーブルからこのテーブルへ向かうFKに対応する。FKが複数ある場合は `via=` で選ぶ。

## nouns

名詞の辞書。`singular plural comment` の順で列挙する。

```
nouns {
  user     users      "ユーザー"
  category categories "商品カテゴリ"
  person   people     "担当者"
  child    children   "子要素"
}
```

テーブル名・FK列名・関連名はすべてここの名詞から組み立てる。`singular()` / `plural()` はこの辞書で解決し、無い名詞は規則変化にフォールバックして警告を出す。

このブロックは2つのことを引き受けている。

**不規則変化を仕様の一部にする。** 規則変化で出せる語は多いが、出せない語がある。

| 語 | 規則変化 | 正しくは |
|---|---|---|
| `person` | `persons` | `people` |
| `child` | `childs` | `children` |
| `analysis` | `analysises` | `analyses` |
| `datum` | `datums` | `data` |

辞書に書かないと、こうした名前が**警告付きではあるが黙って**通る。書けば、変換規則がコメントやコミットログではなく**定義そのもの**として残る。

**用語集がスキーマ定義の副産物になる。** 第3列の説明は DDL の `COMMENT` になる。「このテーブルは業務上何を指すか」を別のドキュメントに書くと必ず陳腐化するが、ここに書けば**スキーマを書く行為そのものに用語の定義が含まれる**。

`table` 宣言の無いエントリはエラーにしない。名前の部品としてしか使わない名詞（`item` / `history` など）も登録する。

単数形と複数形を持たないもの（`sent` のような修飾語）はここに書かない。文字列として直接書く。

## 複合名詞

`noun(a, b, ...)` は複数の要素から名詞を作る。戻り値は文字列ではなく**名詞**であり、単数形と複数形の両方を持つ。

- 最後の要素だけが文脈の数に従う
- それ以外の要素は単数形になる
- 文字列の要素は屈折しない

```
noun("sent", message)   # 単数形 sent_message / 複数形 sent_messages
noun(order, item)       # 単数形 order_item   / 複数形 order_items
```

数は使う側が決める。

```
has_many message alias=noun("sent", message)   # sent_messages
has_one  profile alias=noun("main", profile)   # main_profile
```

連結には `naming.noun_separator` を使う。

## comment

ブロックを持つものは**文**で、持たないものは**属性**で書く。

| 対象 | 書き方 |
|---|---|
| テーブル | `table` の中に `comment "..."` |
| カラム | `column ... comment="..."` |
| FK列 | `belongs_to ... comment="..."` |
| `associate` が作るテーブル | `associate(a, b, comment="...")` |

```
table user {
  comment "ユーザー"
  column email type=text comment="ログインID"
}
```

`nouns` の第3列もテーブルのコメントになる。`table` の中に `comment` があればそちらを採用する。

`mixin` と `blueprint` はテーブルにならないので `comment` を書けない。ソース上の注釈は `#` を使う。

### 展開

コメントの文字列では `${...}` が使える。参照できるのは名詞で、blueprint の仮引数と `let` 束縛もそのまま書ける。

```
nouns {
  post    posts     "投稿"
  history histories "履歴"
}

blueprint audited target {
  let t_history = noun(target, history)

  table t_history {
    comment "${desc(target)}の変更履歴"
    column id type=serial
    pk id
  }
}

apply_blueprint(audited, post)
```

`post_histories` が生成され、そのコメントは `desc(post)` すなわち `nouns` の第3列から「投稿の変更履歴」になる。

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
  index = "idx_${table}_${columns}"
  unique_index = "uq_${table}_${columns}"
  column_separator = "_"
  noun_separator = "_"
  belongs_to = "${singular(table)}"
  has_many = "${plural(table)}"
  has_one = "${singular(table)}"
}
```

`${columns}` は複数列のとき `column_separator` で連結する。複合名詞は `noun_separator` で連結する。

`belongs_to` は参照先の名詞、`has_many` / `has_one` はFKを持つ側の名詞を `table` として展開する。

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

**複数のテーブルが互いを参照して初めて成立する構造**に、1つの名前を与える。

「承認フロー」は申請・段階・コメントの3テーブルが揃って初めて意味を持つ。これを個別の `table` として並べて書くと、どのテーブル群が1つの仕組みなのかがソースから読み取れず、別の名詞に同じ仕組みを付けるたびに手で写すことになる。

`mixin` は1テーブルの中に閉じる。`belongs_to` は2テーブルの間を繋ぐ。それより広い範囲をまとめるのが `blueprint` で、この3つを最初から別の構文にしてあるので、名前を見れば何が生成されるのかが判る。

```
blueprint approvable target comment="承認フロー" {
  let t_approval = noun(target, approval)
  let t_step     = noun(t_approval, step)

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

引数はすべて名詞。型注釈は書かない。渡す名詞は `nouns` に登録されていなければエラーとする。`desc()` で説明を引けるようにするため。

### 局所束縛 `let`

`let 名前 = 式` で名詞を束縛する。再代入は不可。

### 名前解決

参照は裸の識別子で書く。次の順で解決する。

1. 仮引数
2. `let` 束縛
3. 名詞

仮引数名・`let` 束縛名が名詞と衝突した場合はエラー。

### `let` が束縛するもの

`let` は名詞を束縛する。テーブル名はそこから `table_name` で決まる。

```
let t_approval = noun(target, approval)   # 名詞 post_approval / post_approvals
table t_approval { ... }                  # table_name=plural なので post_approvals
```

## バリデーション

- `type=` の値が出力ターゲットの型名にあるか
- `override` 対象のカラムがmixin側に存在するか
- テーブル名・列名の重複
- 出力ターゲットの予約語との衝突（警告）
- `use` の循環参照
- blueprint の仮引数名・`let` 束縛名と名詞の衝突
- 名詞が `nouns` に無く規則変化で解決された場合は警告。ただし blueprint の引数はエラー
- `comment` の `${...}` が参照する名詞に説明が無い
- `has_many` / `has_one` に対応するFKが無い
- FKが複数あるのに `via=` が無い
- `has_many` を `unique_belongs_to` に、`has_one` を `belongs_to` に対して書いている
- 関連名が同じテーブルのカラム名や他の関連名と衝突している
- 主キーが無い（警告）。`except` で主キーの列を消した場合もここで出る
- `except` でFK列を消して関連が消えた（警告）

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
  user       users       "ユーザー"
  category   categories  "商品カテゴリ"
  product    products    "商品"
  order      orders      "注文"
  item       items       "明細"
  order_item order_items "注文明細"
  post       posts       "投稿"
  profile    profiles    "プロフィール"
  history    histories   "履歴"
  favorite   favorites   "いいね"
}

mixin primary_key {
  column id type=serial comment="主キー"
  pk id
}

mixin timestamps {
  column created_at type=timestamptz default=eval(now()) comment="作成日時"
  column updated_at type=timestamptz default=eval(now()) on_update=eval(now()) comment="更新日時"
}

mixin describable {
  column note type=varchar null=true comment="備考"
}

mixin base {
  use primary_key
  use timestamps
}

mixin publishable {
  column status       type=text                 comment="公開状態"
  column published_at type=timestamptz null=true comment="公開日時"
  index status
  index published_at
}

blueprint audited target {
  let t_history = noun(target, history)

  table t_history {
    comment "${desc(target)}の変更履歴"
    use primary_key
    belongs_to target comment="変更対象"
    belongs_to user   comment="変更した人"
    column version     type=integer                        comment="版番号"
    column snapshot    type=jsonb                          comment="変更時点の内容"
    column recorded_at type=timestamptz default=eval(now()) comment="記録日時"
  }
}

table user {
  comment "ユーザー"
  use base
  column email type=text comment="ログインID"
  column name  type=text comment="表示名"
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
  column name type=text comment="カテゴリ名"
  index name unique
}

table product {
  use base
  column name  type=text    comment="商品名"
  column price type=integer comment="税抜価格（円）"
}

table order {
  use base
  belongs_to user comment="注文したユーザー"
}

table order_item {
  use base
  belongs_to order   comment="親の注文"
  belongs_to product comment="対象の商品"
  column quantity type=integer comment="数量"
}

table post {
  use base
  use publishable
  except index published_at
  belongs_to user     comment="投稿者"
  belongs_to category comment="所属カテゴリ"
  column title type=text comment="表題"
  index [status, created_at] unique
}

table profile {
  use base
  unique_belongs_to user comment="持ち主"
  column bio type=text null=true comment="自己紹介"
}

associate(user, post, name=favorite, comment="いいね")
apply_blueprint(audited, post)
```
