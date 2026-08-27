import type { JSX } from "solid-js";
import type { Lang } from "./i18n";
import type { PageId } from "./pages";

/** 概要ページに出る構文。並び順は表の行の順。 */
export const CONSTRUCTS = ["mixin", "belongs_to", "associate", "blueprint"] as const;
export type Construct = (typeof CONSTRUCTS)[number];

export type OverviewCopy = {
  readonly heading: string;
  readonly lede: string;
  readonly tryIt: string;
  readonly install: string;
  readonly removes: { readonly heading: string; readonly body: () => JSX.Element };
  readonly conventions: { readonly heading: string; readonly body: () => JSX.Element };
  readonly names: { readonly heading: string; readonly body: () => JSX.Element };
  readonly constructs: {
    readonly heading: string;
    readonly body: () => JSX.Element;
    readonly columns: readonly [string, string, string];
    /** [生成するもの, 何をまたいで使い回すか] */
    readonly rows: Record<Construct, readonly [string, string]>;
  };
  readonly nouns: { readonly heading: string; readonly body: () => JSX.Element };
  readonly next: {
    readonly heading: string;
    readonly items: readonly { readonly id: PageId; readonly label: string; readonly note: string }[];
  };
};

export const OVERVIEW: Record<Lang, OverviewCopy> = {
  en: {
    heading: "Write the conventions once.",
    lede: "Mixins, naming rules and a dictionary of nouns settle the repetitive parts of a schema. What stays in a table definition is what makes that table different.",
    tryIt: "Try it in the browser",
    install: "Install",
    removes: {
      heading: "What it removes",
      body: () => (
        <>
          Writing DDL by hand spreads the same column definitions, the same constraints and the same
          naming habits across every table. NounSQL folds that in four places.
        </>
      ),
    },
    conventions: {
      heading: "Conventions live in one place",
      body: () => (
        <>
          Primary keys, timestamps, anything every table carries — put it in a <code>mixin</code>.
          What stays in a table definition is what makes that table different. Changing a convention
          means editing one block.
        </>
      ),
    },
    names: {
      heading: "Names stop drifting",
      body: () => (
        <>
          <code>naming</code> holds the rules for table names, foreign key columns and index names.
          The difference between <code>idx_</code> and <code>ix_</code>, between{" "}
          <code>user_id</code> and <code>userId</code>, is settled by a definition instead of by
          review comments.
        </>
      ),
    },
    constructs: {
      heading: "Each construct produces one thing",
      body: () => (
        <>What a name generates is decided by which construct it is, so you can tell without opening it.</>
      ),
      columns: ["Construct", "Produces", "Reused across"],
      rows: {
        mixin: ["columns and indexes inside a table", "tables"],
        belongs_to: ["a foreign key column", "—"],
        associate: ["a table between two others", "—"],
        blueprint: ["whole tables", "nouns"],
      },
    },
    nouns: {
      heading: "What nouns declares",
      body: () => (
        <>
          <p>Each entry gives a singular, a plural, and a description.</p>
          <p>
            The plural is written, not guessed. <code>person</code> → <code>people</code>,{" "}
            <code>child</code> → <code>children</code>: no rule produces those.
          </p>
          <p>
            The description says what the noun means to the business, and reaches the DDL as a{" "}
            <code>COMMENT</code> on every table and column named after it.
          </p>
        </>
      ),
    },
    next: {
      heading: "Next",
      items: [
        { id: "guide", label: "Guide", note: "install and run the compiler" },
        { id: "samples", label: "Samples", note: "working schemas and the DDL they produce" },
        { id: "playground", label: "Playground", note: "edit and compile in the browser" },
        { id: "spec", label: "Specification", note: "syntax and resolution rules" },
      ],
    },
  },

  ja: {
    heading: "規約は一度だけ書く。",
    lede: "mixin と命名規則、そして名詞の辞書が、スキーマの繰り返しの部分を引き受けます。テーブルの定義に残るのは、そのテーブルを他と違うものにしている部分だけです。",
    tryIt: "ブラウザで試す",
    install: "インストール",
    removes: {
      heading: "何が消えるか",
      body: () => (
        <>
          DDL を手で書くと、同じカラム定義、同じ制約、同じ命名の癖が、全テーブルに散らばります。
          NounSQL はそれを4か所に畳みます。
        </>
      ),
    },
    conventions: {
      heading: "規約が1か所に集まる",
      body: () => (
        <>
          主キー、タイムスタンプ、どのテーブルも持つもの——それは <code>mixin</code> に置きます。
          テーブルの定義に残るのは、そのテーブルを他と違うものにしている部分だけです。規約を変えるとは、
          1つのブロックを直すことになります。
        </>
      ),
    },
    names: {
      heading: "名前がぶれなくなる",
      body: () => (
        <>
          <code>naming</code> がテーブル名・外部キー列・インデックス名の規則を持ちます。
          <code>idx_</code> と <code>ix_</code> のどちらか、<code>user_id</code> と{" "}
          <code>userId</code> のどちらかは、レビューのコメントではなく定義で決まります。
        </>
      ),
    },
    constructs: {
      heading: "構文ごとに作るものが1つ決まっている",
      body: () => <>その名前が何を生成するかは構文で決まるので、中を開かなくても分かります。</>,
      columns: ["構文", "作るもの", "使い回す単位"],
      rows: {
        mixin: ["テーブルの中のカラムとインデックス", "テーブル"],
        belongs_to: ["外部キー列", "—"],
        associate: ["2つのテーブルの間のテーブル", "—"],
        blueprint: ["テーブルそのもの", "名詞"],
      },
    },
    nouns: {
      heading: "nouns が宣言するもの",
      body: () => (
        <>
          <p>1行につき、単数形・複数形・説明の3つを書きます。</p>
          <p>
            複数形は推測ではなく、書きます。<code>person</code> → <code>people</code>、
            <code>child</code> → <code>children</code>。これを作る規則はありません。
          </p>
          <p>
            説明はその名詞が業務で何を指すかで、その名詞から名前が付いた全テーブル・全カラムに{" "}
            <code>COMMENT</code> として DDL に出ます。
          </p>
        </>
      ),
    },
    next: {
      heading: "次に読む",
      items: [
        { id: "guide", label: "ガイド", note: "インストールしてコンパイラを動かす" },
        { id: "samples", label: "サンプル", note: "動くスキーマと、そこから出る DDL" },
        { id: "playground", label: "プレイグラウンド", note: "ブラウザで編集してコンパイルする" },
        { id: "spec", label: "仕様", note: "構文と解決の規則" },
      ],
    },
  },
};
