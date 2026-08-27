/**
 * HTML のエントリを書き出す。
 *
 * ページ × 言語の分だけ必要だが、中身は `data-page` と `data-lang` と
 * <head> しか違わない。手で並べると必ずどれかが古くなるので、
 * ここで1つの雛形から作る。書き出したファイルは git に入れない。
 */
import { existsSync, mkdirSync, rmSync, writeFileSync } from "node:fs";
import { dirname, resolve } from "node:path";
import { LANGS, type Lang } from "./lang.ts";
import { DESCRIPTION, TITLE } from "./meta.ts";
import { PAGES, outPath, type PageId } from "./pages.ts";

const FAVICON =
  "data:image/svg+xml,<svg xmlns='http://www.w3.org/2000/svg' viewBox='0 0 16 16'><text y='13' font-size='14'>&#128190;</text></svg>";

const SITE = "https://misebox.github.io/nounsql/";

function escape(text: string): string {
  return text.replace(/&/g, "&amp;").replace(/</g, "&lt;").replace(/"/g, "&quot;");
}

/** 同じページの他の言語版。検索エンジンに対応関係を伝える。 */
function alternates(id: PageId, langs: readonly Lang[]): string {
  return langs
    .map(
      (lang) =>
        `    <link rel="alternate" hreflang="${lang}" href="${SITE}${outPath(id, lang)}" />`,
    )
    .join("\n");
}

function html(id: PageId, lang: Lang, langs: readonly Lang[]): string {
  return `<!doctype html>
<html lang="${lang}">
  <head>
    <meta charset="utf-8" />
    <meta name="viewport" content="width=device-width, initial-scale=1" />
    <meta name="description" content="${escape(DESCRIPTION[lang])}" />
    <link rel="icon" href="${FAVICON}" />
${alternates(id, langs)}
    <title>${escape(TITLE[lang][id])}</title>
  </head>
  <body>
    <div id="app" data-page="${id}" data-lang="${lang}"></div>
    <script type="module" src="/src/main.tsx"></script>
  </body>
</html>
`;
}

/**
 * エントリを書き出し、rollup の input を返す。
 * 既定の言語がサイトのルート、他が `/<lang>/` に出る。
 */
export function writeEntries(root: string): Record<string, string> {
  const input: Record<string, string> = {};

  for (const page of PAGES) {
    for (const lang of page.langs) {
      const rel = outPath(page.id, lang);
      const file = resolve(root, rel);
      const dir = dirname(file);
      if (!existsSync(dir)) mkdirSync(dir, { recursive: true });
      writeFileSync(file, html(page.id, lang, page.langs));
      input[rel.replace(/\.html$/, "")] = file;
    }
  }

  // ページの言語を減らしたときに、前回の書き出しが残って dev サーバに
  // 出てしまわないようにする。消すのは自分が作る形のファイルだけ。
  for (const page of PAGES) {
    for (const lang of LANGS) {
      const rel = outPath(page.id, lang);
      if (rel.replace(/\.html$/, "") in input || !existsSync(resolve(root, rel))) continue;
      rmSync(resolve(root, rel));
    }
  }

  return input;
}
