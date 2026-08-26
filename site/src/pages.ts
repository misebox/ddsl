/** サイトのページ。並び順がそのままナビゲーションの並びになる。 */
export type Page = {
  readonly id: string;
  readonly file: string;
  readonly nav: string;
  readonly title: string;
};

export const PAGES: readonly Page[] = [
  { id: "index", file: "", nav: "overview", title: "NounSQL — a DSL for Database Schema Design" },
  { id: "guide", file: "guide.md", nav: "guide", title: "Guide — NounSQL" },
  { id: "spec", file: "spec.md", nav: "spec", title: "Specification — NounSQL" },
  { id: "samples", file: "", nav: "samples", title: "Samples — NounSQL" },
  { id: "playground", file: "", nav: "playground", title: "Playground — NounSQL" },
  { id: "tooling", file: "tooling.md", nav: "internals", title: "Internals — NounSQL" },
];

export function pageById(id: string): Page | undefined {
  return PAGES.find((page) => page.id === id);
}

/** ページの URL。MPA なので実ファイルを指す。 */
export function href(page: Page): string {
  return `./${page.id}.html`;
}
