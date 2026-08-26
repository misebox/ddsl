# 未決の設計判断

`docs/spec.md` に書けていない、決めるべき事項の一覧。

## dialect

- 型のマッピング（抽象型 → ターゲット型）を持つか、ターゲット型名を直接書かせるか。現状は後者。
- パラメータ付き型（`varchar(255)` / `decimal(10,2)`）を許すか。
- PostgreSQL 以外のターゲットを足すときに `naming` / `constraints` の既定値も変えるか。

## column

- **`override` / `except` の対象範囲**：mixin由来のみか、`belongs_to` 生成列も対象にできるか。
- **mixinカラムの列順**：`use()` の位置に展開するか、先頭に集めるか。DDLの列順に直結する。
- **同名カラムの衝突**：mixin同士 / mixinと自テーブル。エラーか後勝ちか。

## blueprint

- **適用範囲**：新規テーブルの生成のみか、既存テーブルへのカラム・制約の追加も許すか。後者は適用順序と衝突解決が難しくなる。
- **共有テーブル**：`taggable(target)` のように「`tags` は一度だけ生成、`${target}_tags` は適用ごとに生成」を書けない。「一度だけ」「適用ごと」の区別が必要。
- **`associate` を blueprint 内に書けるか**（現状は global スコープのディレクティブ）。
- **呼び出し記法**：`apply_blueprint(audited, post)` は第1引数だけが名前で残りが引数という非対称。`audited(post)` にすれば通常のディレクティブ呼び出しに揃うが、組み込みディレクティブと名前空間を共有する。
- **`comment` のテンプレート**：生成テーブルの comment に `${comment(target)}` のような補間を許すか。

## relation / index

- **index種別の構文**：バリデーション章が `gin` / `gist` に言及しているが構文が無い。`index(data) type=gin` が形としては揃う。
- **`belongs_to` の同一テーブル複数参照**：FK列名が衝突する。エイリアス構文が無いため自己参照テーブルも書けない。
- **FKの `on_delete` 個別指定**：config既定値のみでテーブル個別に変えられない。

## その他

- **コメント構文**（`--` / `//` / `#`）。ソース上の注釈であって `comment=` とは別。
- **`join_table` と `name_join` の単複不一致**：`join_table` は `${singular(a)}_${singular(b)}`（`user_post`）、`name_join` は `${singular(a)}_${plural(b)}`（`post_approvals`）。
- **予約語チェックの責務**：PostgreSQL / MySQL で異なるため、コア検証ではなく出力ターゲット側に置く。
- **出力ターゲットごとの codegen 仕様**（SQL / Prisma / Drizzle 等）。
