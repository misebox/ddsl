import { DEFAULT_LANG, LANGS, type Lang } from "./lang.ts";

export type PageId = "index" | "guide" | "spec" | "samples" | "playground" | "tooling";

export type Page = {
  readonly id: PageId;
  /** 本文が markdown のページ。`docs/<lang>/<doc>` を読む。 */
  readonly doc?: string;
  /** 用意がある言語。ここに無い言語では、既定の言語のページへ送る。 */
  readonly langs: readonly Lang[];
};

/** 並び順がそのままナビゲーションの並びになる。 */
export const PAGES: readonly Page[] = [
  { id: "index", langs: LANGS },
  { id: "guide", doc: "guide.md", langs: LANGS },
  // 仕様は動きが速いので原本を1つに保つ。訳を足したらここに "ja" を書く。
  { id: "spec", doc: "spec.md", langs: ["en"] },
  { id: "samples", langs: LANGS },
  { id: "playground", langs: LANGS },
  { id: "tooling", doc: "tooling.md", langs: LANGS },
];

export function pageById(id: string | undefined): Page | undefined {
  return PAGES.find((page) => page.id === id);
}

export function hasLang(page: Page, lang: Lang): boolean {
  return page.langs.includes(lang);
}

/** そのページを読める言語のうち、希望に一番近いもの。 */
export function langFor(page: Page, wanted: Lang): Lang {
  return hasLang(page, wanted) ? wanted : DEFAULT_LANG;
}

/**
 * ページの URL。MPA なので実ファイルを指す。
 * 既定の言語がルート、それ以外が `/<lang>/` にあるという配置に依存している。
 */
export function href(id: PageId, from: Lang, to: Lang): string {
  const file = `${id}.html`;
  if (from === to) return `./${file}`;
  return to === DEFAULT_LANG ? `../${file}` : `./${to}/${file}`;
}

/** dist の中でのそのページの位置。 */
export function outPath(id: PageId, lang: Lang): string {
  return lang === DEFAULT_LANG ? `${id}.html` : `${lang}/${id}.html`;
}
