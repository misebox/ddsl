/** Shared by the samples page and the playground. The files live in examples/. */
import type { Lang } from "./lang.ts";

export type Sample = {
  readonly file: string;
  readonly label: string;
  readonly summary: string;
};

const SAMPLES_BY_LANG: Record<Lang, readonly Sample[]> = {
  en: [
    {
      file: "minimal.nsql",
      label: "Minimal",
      summary: "Register the nouns, put shared columns in a mixin, connect tables with belongs_to.",
    },
    {
      file: "mixin.nsql",
      label: "Mixins",
      summary: "Keep conventions in one place and write only what makes each table different.",
    },
    {
      file: "relations.nsql",
      label: "Relations",
      summary: "Two references to the same table, a self reference, and a join table with columns.",
    },
    {
      file: "blueprint.nsql",
      label: "Blueprints",
      summary: "Generating tables from a noun, so the same structure can be attached to another.",
    },
    {
      file: "naming.nsql",
      label: "Naming",
      summary: "Where table names, foreign key columns and index names come from.",
    },
    {
      file: "identifier.nsql",
      label: "Identifiers",
      summary: "Short identifiers in the file, long nouns in the database. Every generated name comes from the noun.",
    },
    {
      file: "shop.nsql",
      label: "A real schema",
      summary: "Ten tables of an online shop. On top of conventions, only the differences remain.",
    },
    {
      file: "legacy.nsql",
      label: "An existing database",
      summary: "Describing tables and columns whose names were decided somewhere else.",
    },
    {
      file: "sample.nsql",
      label: "Everything",
      summary: "Every construct that appears in the specification.",
    },
  ],
  ja: [
    {
      file: "minimal.nsql",
      label: "最小",
      summary: "名詞を登録し、共通のカラムを mixin に置き、belongs_to でテーブルをつなぐ。",
    },
    {
      file: "mixin.nsql",
      label: "mixin",
      summary: "規約を1か所に置き、テーブルごとに違うところだけを書く。",
    },
    {
      file: "relations.nsql",
      label: "関連",
      summary: "同じテーブルへの2つの参照、自己参照、カラムを持つ中間テーブル。",
    },
    {
      file: "blueprint.nsql",
      label: "blueprint",
      summary: "名詞からテーブルを生成し、同じ構造を別の名詞にも付ける。",
    },
    {
      file: "naming.nsql",
      label: "命名",
      summary: "テーブル名・外部キー列・インデックス名がどこから来るか。",
    },
    {
      file: "identifier.nsql",
      label: "識別子",
      summary: "ファイルの中は短い識別子、データベースには長い名詞。生成される名前は全部名詞から来る。",
    },
    {
      file: "shop.nsql",
      label: "実際のスキーマ",
      summary: "オンラインショップの10テーブル。規約の上に、違いだけが残る。",
    },
    {
      file: "legacy.nsql",
      label: "既存のデータベース",
      summary: "名前が別のところで決まっているテーブルとカラムを書く。",
    },
    {
      file: "sample.nsql",
      label: "全構文",
      summary: "仕様に出てくる構文を全部使ったもの。",
    },
  ],
};

export function samples(lang: Lang): readonly Sample[] {
  return SAMPLES_BY_LANG[lang];
}
