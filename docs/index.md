# DDSL

**Data Definition Schema Language is a DSL for Database Schema Design.**

スキーマ定義を書くと PostgreSQL の DDL が出る。

## 何を解くか

DDL を直接書くと、同じ列定義・同じ制約・同じ命名規則が全テーブルに散らばる。DDSL はそれを `mixin` と命名規則にまとめ、テーブルごとの差分だけを書けるようにする。

```
mixin base {
  column id type=serial
  pk id
  column created_at type=timestamptz default=eval(now())
}

table user {
  comment "ユーザー"
  use base
  column email type=text
  index email unique
}

table post {
  use base
  belongs_to user
  column title type=text
}
```

これが次の DDL になる。FK 列 `user_id`、その型、index 名は書いていない。命名規則から決まる。

```sql
CREATE TABLE users (
  id serial NOT NULL,
  created_at timestamptz NOT NULL DEFAULT now(),
  email text NOT NULL,
  CONSTRAINT users_pkey PRIMARY KEY (id)
);

CREATE TABLE posts (
  id serial NOT NULL,
  created_at timestamptz NOT NULL DEFAULT now(),
  user_id integer NOT NULL,
  title text NOT NULL,
  CONSTRAINT posts_pkey PRIMARY KEY (id)
);

CREATE UNIQUE INDEX uq_users_email ON users (email);
CREATE INDEX idx_posts_user_id ON posts (user_id);

ALTER TABLE posts ADD CONSTRAINT posts_user_id_fkey
  FOREIGN KEY (user_id) REFERENCES users (id) ON DELETE CASCADE ON UPDATE CASCADE;

COMMENT ON TABLE users IS 'ユーザー';
```

## 言語の要素

| 要素 | 役割 |
|---|---|
| `mixin` | 1テーブル内のカラム・index・制約をまとめる。`use` の位置に展開される |
| `override` / `except` | mixin の一部だけを差し替える・外す |
| `blueprint` | 複数テーブルにまたがる構造を定義する。`let` でテーブル名を束縛する |
| `belongs_to` / `associate` | FK と中間テーブルを生成する |
| `has_many` / `has_one` | 逆参照に名前を付ける。DDL には出ない |
| `nouns` | 単数形・複数形・説明の辞書。テーブル名・FK列名・関連名・コメントの元になる |
| `noun(a, b, ...)` | 複合名詞を作る。数は使う側の文脈が決める |
| `naming` / `constraints` | 命名規則と制約の既定値 |

## 構文の原則

- 1行1文。行頭のキーワードで文種が決まる
- 属性は `キー=値`。bare keyword は使わない
- テーブルの一部を設定する文は括弧なし（`use base` / `belongs_to user` / `index email unique`）
- 引数リストを取るものだけが関数・マクロ（`associate(a, b)` / `noun(a, b)` / `eval(now())`）

詳細は [仕様](spec.md)。

## 使う

```
git clone https://github.com/misebox/ddsl
cd ddsl
cargo run -p ddsl -- sql examples/sample.ddsl
```

エディタ連携・LSP・GBNF については [ツール](tooling.md)。

## 状態

v0.1。PostgreSQL 対応。出力ターゲットは `--dialect` で選ぶ（既定 `postgres`）。
