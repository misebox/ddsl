import type { JSX } from "solid-js";
import { lang as currentLang, type Lang } from "./lang";
import { TITLE } from "./meta";
import type { PageId } from "./pages";

export { DEFAULT_LANG, LANGS, LANG_LABEL, isLang, lang, setLang, type Lang } from "./lang";

export function t(): Strings {
  return STRINGS[currentLang()];
}

/** そのページの <title>。 */
export function title(lang: Lang, page: PageId): string {
  return TITLE[lang][page];
}

type Strings = {
  readonly skip: string;
  readonly tagline: string;
  readonly tocTitle: string;
  readonly nav: Record<PageId, string>;
  /** 英語版しか無いページに付ける印。 */
  readonly englishOnly: string;
  readonly samples: {
    readonly heading: string;
    readonly intro: () => JSX.Element;
    readonly listLabel: string;
  };
  readonly playground: {
    readonly heading: string;
    readonly intro: () => JSX.Element;
    readonly source: string;
    readonly output: string;
    readonly example: string;
    readonly target: string;
    readonly copy: string;
    readonly copied: string;
    readonly diagnostics: string;
    readonly editorLabel: string;
    readonly loading: string;
    readonly loadFailed: string;
    readonly clipboardFailed: string;
  };
  readonly code: {
    readonly source: string;
    readonly output: string;
  };
  /** 件数の言い回し。日本語には複数形が無いので言語ごとに持つ。 */
  readonly count: {
    readonly errors: (n: number) => string;
    readonly warnings: (n: number) => string;
    readonly size: (tables: number, columns: number) => string;
  };
};

const STRINGS: Record<Lang, Strings> = {
  en: {
    skip: "Skip to content",
    tagline: "NounSQL is a DSL for Database Schema Design.",
    tocTitle: "On this page",
    nav: {
      index: "overview",
      guide: "guide",
      spec: "spec",
      samples: "samples",
      playground: "playground",
      tooling: "how it works",
    },
    englishOnly: "English only",
    samples: {
      heading: "Samples",
      intro: () => (
        <>
          Every one compiles without warnings. The same files live in <code>examples/</code>, and
          the <a href="./playground.html">playground</a> loads them for editing.
        </>
      ),
      listLabel: "Samples",
    },
    playground: {
      heading: "Playground",
      intro: () => (
        <>
          Compiles in the browser. Nothing you type is sent anywhere. It runs the{" "}
          <a href="https://github.com/misebox/nounsql/tree/main/crates/nounsql-wasm">
            compiler itself, built to WebAssembly
          </a>
          — the same lexer, resolver and code generator the CLI uses. The examples are described on
          the <a href="./samples.html">samples</a> page.
        </>
      ),
      source: "source",
      output: "output",
      example: "example",
      target: "target",
      copy: "Copy DDL",
      copied: "Copied",
      diagnostics: "Diagnostics",
      editorLabel: "NounSQL source",
      loading: "loading",
      loadFailed: "compiler failed to load",
      clipboardFailed: "could not write to the clipboard",
    },
    code: {
      source: "NounSQL",
      output: "Generated DDL",
    },
    count: {
      errors: (n) => `${n} error${n > 1 ? "s" : ""}`,
      warnings: (n) => `${n} warning${n > 1 ? "s" : ""}`,
      size: (tables, columns) => `${tables} tables · ${columns} columns`,
    },
  },

  ja: {
    skip: "本文へ",
    tagline: "NounSQL はデータベーススキーマ設計のための DSL です。",
    tocTitle: "このページの見出し",
    nav: {
      index: "概要",
      guide: "ガイド",
      spec: "仕様",
      samples: "サンプル",
      playground: "プレイグラウンド",
      tooling: "仕組み",
    },
    englishOnly: "英語のみ",
    samples: {
      heading: "サンプル",
      intro: () => (
        <>
          どれも警告なしでコンパイルできます。同じファイルが <code>examples/</code> にあり、
          <a href="./playground.html">プレイグラウンド</a>で編集できます。
        </>
      ),
      listLabel: "サンプル",
    },
    playground: {
      heading: "プレイグラウンド",
      intro: () => (
        <>
          ブラウザの中でコンパイルします。入力したものはどこにも送られません。動いているのは
          <a href="https://github.com/misebox/nounsql/tree/main/crates/nounsql-wasm">
            コンパイラ自身を WebAssembly にしたもの
          </a>
          で、CLI と同じ字句解析器・解決器・コード生成器です。例の説明は
          <a href="./samples.html">サンプル</a>のページにあります。
        </>
      ),
      source: "ソース",
      output: "出力",
      example: "例",
      target: "出力先",
      copy: "DDL をコピー",
      copied: "コピーしました",
      diagnostics: "診断",
      editorLabel: "NounSQL のソース",
      loading: "読み込み中",
      loadFailed: "コンパイラを読み込めませんでした",
      clipboardFailed: "クリップボードに書けませんでした",
    },
    code: {
      source: "NounSQL",
      output: "生成された DDL",
    },
    count: {
      errors: (n) => `エラー ${n} 件`,
      warnings: (n) => `警告 ${n} 件`,
      size: (tables, columns) => `${tables} テーブル · ${columns} カラム`,
    },
  },
};
