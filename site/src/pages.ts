/** サイトのページ。並び順がそのままナビゲーションの並びになる。 */
export type Page = {
  readonly id: string;
  readonly file: string;
  readonly nav: string;
  readonly title: string;
};

export const PAGES: readonly Page[] = [
  { id: "index", file: "", nav: "概要", title: "NounSQL — a DSL for Database Schema Design" },
  { id: "guide", file: "guide.md", nav: "使い方", title: "使い方 — NounSQL" },
  { id: "spec", file: "spec.md", nav: "仕様", title: "仕様 — NounSQL" },
  { id: "samples", file: "", nav: "サンプル", title: "サンプル — NounSQL" },
  { id: "playground", file: "", nav: "プレイグラウンド", title: "プレイグラウンド — NounSQL" },
  { id: "tooling", file: "tooling.md", nav: "ツール", title: "ツール — NounSQL" },
];

export function pageById(id: string): Page | undefined {
  return PAGES.find((page) => page.id === id);
}

/** ページの URL。MPA なので実ファイルを指す。 */
export function href(page: Page): string {
  return `./${page.id}.html`;
}
