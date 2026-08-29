import type { JSX } from "solid-js";
import type { Lang } from "./i18n";
import type { PageId } from "./pages";

/** 概要ページの表に出る構文。並び順は行の順。 */
export const CONSTRUCTS = ["mixin", "belongs_to", "associate", "blueprint"] as const;
export type Construct = (typeof CONSTRUCTS)[number];

type Section = { readonly heading: string; readonly body: () => JSX.Element };

/** 本文からリンクする資材の場所。ページの言語で変わるので描画時に渡す。 */
export type Links = { readonly spec: string; readonly gbnf: string; readonly llms: string };
type LinkedSection = { readonly heading: string; readonly body: (to: Links) => JSX.Element };

export type OverviewCopy = {
  readonly heading: string;
  readonly lede: string;
  readonly tryIt: string;
  readonly install: string;
  readonly repetition: Section;
  readonly mixin: Section;
  readonly naming: Section;
  readonly nouns: Section;
  readonly constructs: Section & {
    readonly columns: readonly [string, string, string];
    /** [生成するもの, 何をまたいで使い回すか] */
    readonly rows: Record<Construct, readonly [string, string]>;
  };
  readonly output: Section;
  readonly generated: LinkedSection;
  readonly next: {
    readonly heading: string;
    readonly items: readonly { readonly id: PageId; readonly label: string; readonly note: string }[];
  };
};

export const OVERVIEW: Record<Lang, OverviewCopy> = {
  en: {
    heading: "Write a schema. Get the DDL.",
    lede: "NounSQL is a small language for defining database schemas. It compiles to CREATE TABLE. The columns every table carries, the rules that form names, and what your nouns mean are each written once.",
    tryIt: "Try it in the browser",
    install: "Install",
    repetition: {
      heading: "Where the repetition goes",
      body: () => (
        <>
          Writing DDL by hand spreads the same column definitions, the same constraints and the same
          naming habits across every table. NounSQL collects them into three declarations.
        </>
      ),
    },
    mixin: {
      heading: "mixin — what every table carries",
      body: () => (
        <>
          Primary keys, timestamps, audit columns. What stays in a table definition is what makes
          that table different, and changing a convention means editing one block.
        </>
      ),
    },
    naming: {
      heading: "naming — how names are formed",
      body: () => (
        <>
          Table names, foreign key columns, index names. Whether it is <code>idx_</code> or{" "}
          <code>ix_</code>, <code>user_id</code> or <code>userId</code>, is settled by a definition
          instead of by review comments.
        </>
      ),
    },
    nouns: {
      heading: "nouns — the dictionary",
      body: () => (
        <>
          <p>
            An identifier and a description per entry. The identifier is what the rest of the file
            writes; it never reaches the DDL.
          </p>
          <p>
            <code>singular=</code>, <code>plural=</code> and <code>short=</code> set the word forms
            that do. Left out, each follows from the one before it, so a regular noun is one line.
          </p>
          <p>
            The description says what the noun means to the business, and reaches the DDL as a{" "}
            <code>COMMENT</code> on every table and column named after it.
          </p>
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
    output: {
      heading: "The output is just SQL",
      body: () => (
        <>
          NounSQL is neither an ORM nor a migration tool. It prints DDL: run it into{" "}
          <code>psql</code>, paste it into a migration, diff it against what the database already
          has. A project that writes its SQL by hand still gets one place to define the schema.
        </>
      ),
    },
    generated: {
      heading: "Writing it with a model",
      body: (to) => (
        <>
          The grammar ships as a <a href={to.gbnf}>GBNF file</a>, so constrained decoding cannot
          leave the syntax. The <a href={to.spec}>specification</a> is one page, and{" "}
          <a href={to.llms}>llms.txt</a> collects the documentation for a model to read.
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
    heading: "スキーマを書けば、DDL が出る。",
    lede: "NounSQL はデータベースのスキーマを書くための小さな言語です。コンパイルすると CREATE TABLE が出ます。どのテーブルも持つ列、名前の作り方、名詞の意味は、それぞれ一度だけ書きます。",
    tryIt: "ブラウザで試す",
    install: "インストール",
    repetition: {
      heading: "繰り返しの行き先",
      body: () => (
        <>
          DDL を手で書くと、同じ列定義、同じ制約、同じ命名の癖が、全テーブルに散らばります。
          NounSQL はそれを3つの宣言に集めます。
        </>
      ),
    },
    mixin: {
      heading: "mixin — どのテーブルも持つもの",
      body: () => (
        <>
          主キー、タイムスタンプ、監査用の列。テーブルの定義に残るのは、そのテーブルを他と違うものにしている部分だけで、規約を変えるとは1つのブロックを直すことになります。
        </>
      ),
    },
    naming: {
      heading: "naming — 名前の作り方",
      body: () => (
        <>
          テーブル名、外部キー列、インデックス名。<code>idx_</code> と <code>ix_</code> のどちらか、
          <code>user_id</code> と <code>userId</code> のどちらかは、レビューのコメントではなく定義で決まります。
        </>
      ),
    },
    nouns: {
      heading: "nouns — 名詞の辞書",
      body: () => (
        <>
          <p>1行につき、識別子と説明を書きます。識別子はファイルの他の場所が書く名前で、DDL には出ません。</p>
          <p>
            DDL に出る語形は <code>singular=</code>、<code>plural=</code>、<code>short=</code> で決めます。
            省略すると1つ前から順に埋まるので、規則どおりの名詞は1行で済みます。
          </p>
          <p>
            説明はその名詞が業務で何を指すかで、その名詞から名前が付いた全テーブル・全カラムに{" "}
            <code>COMMENT</code> として DDL に出ます。
          </p>
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
    output: {
      heading: "出力はただの SQL",
      body: () => (
        <>
          NounSQL は ORM でもマイグレーションツールでもありません。出るのは DDL です。
          <code>psql</code> に流しても、既存のマイグレーションに貼っても、今の DB との差分を取っても構いません。SQL を手で書いているプロジェクトでも、スキーマの定義は1か所にまとまります。
        </>
      ),
    },
    generated: {
      heading: "モデルに書かせる",
      body: (to) => (
        <>
          文法は <a href={to.gbnf}>GBNF</a> で配っているので、constrained decoding に通せば構文から外れた出力は出ません。<a href={to.spec}>仕様</a>は1ページに収まっていて、
          <a href={to.llms}>llms.txt</a> にモデルが読むドキュメント一式をまとめてあります。
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
