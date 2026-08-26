/** プレイグラウンドとサンプルページが共有する一覧。実物は examples/ にある。 */
export type Sample = {
  readonly file: string;
  readonly label: string;
  readonly summary: string;
  readonly points: readonly string[];
};

export const SAMPLES: readonly Sample[] = [
  {
    file: "minimal.nsql",
    label: "最小",
    summary: "名詞を登録し、共通の列を mixin にまとめ、belongs_to で繋ぐ。",
    points: ["nouns への語の登録", "mixin による共通列", "belongs_to が生む FK 列と索引"],
  },
  {
    file: "mixin.nsql",
    label: "mixin",
    summary: "規約を1箇所に集め、テーブルごとの差分だけを書く。",
    points: ["mixin から mixin を use する", "override で属性だけ差し替える", "except で列や索引を外す"],
  },
  {
    file: "relations.nsql",
    label: "関連",
    summary: "同じテーブルを2回参照する、自分自身を参照する、中間テーブルを持つ。",
    points: [
      "fk= と alias= で FK 列名と関連名を分ける",
      "via= でどちらの逆側かを選ぶ",
      "associate と中間テーブルの使い分け",
    ],
  },
  {
    file: "blueprint.nsql",
    label: "blueprint",
    summary: "3テーブルで1つの仕組みになる構造を、1つの名前で持つ。",
    points: [
      "let で名詞を束縛し、次の let がそれを合成する",
      "生成したテーブル同士を belongs_to で繋ぐ",
      "同じ仕組みを別の名詞へ適用する",
    ],
  },
  {
    file: "naming.nsql",
    label: "命名規則",
    summary: "テーブル名・FK 列名・索引名の作り方を一元化する。",
    points: ["table_name で単複を決める", "索引名の型を決める", "null と参照動作の既定値"],
  },
  {
    file: "shop.nsql",
    label: "実例",
    summary: "10テーブルの EC スキーマ。規約に載せると差分だけが残る。",
    points: ["金額の持ち方を mixin に切り出す", "自己参照によるカテゴリ階層", "複合索引と一意制約"],
  },
  {
    file: "sample.nsql",
    label: "全構文",
    summary: "仕様に出てくる構文をひととおり含む。",
    points: ["blueprint と associate", "comment の展開", "index の除外"],
  },
];
