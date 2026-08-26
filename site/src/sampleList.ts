/** プレイグラウンドとサンプルページが共有する一覧。実物は examples/ にある。 */
export type Sample = {
  readonly file: string;
  readonly label: string;
  readonly summary: string;
};

export const SAMPLES: readonly Sample[] = [
  { file: "minimal.nsql", label: "最小", summary: "名詞を登録し、共通の列を mixin にまとめ、belongs_to で繋ぐ。" },
  { file: "mixin.nsql", label: "mixin", summary: "規約を1箇所に集め、テーブルごとの差分だけを書く。" },
  { file: "relations.nsql", label: "関連", summary: "同じテーブルを2回参照する、自分自身を参照する、中間テーブルを持つ。" },
  { file: "blueprint.nsql", label: "blueprint", summary: "3テーブルで1つの仕組みになる構造を、1つの名前で持つ。" },
  { file: "naming.nsql", label: "命名規則", summary: "テーブル名・FK 列名・索引名の作り方を一元化する。" },
  { file: "shop.nsql", label: "実例", summary: "10テーブルの EC スキーマ。規約に載せると差分だけが残る。" },
  { file: "sample.nsql", label: "全構文", summary: "仕様に出てくる構文をひととおり含む。" },
];
