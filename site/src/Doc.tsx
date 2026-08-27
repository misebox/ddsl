import { For, Show, createMemo } from "solid-js";
import { readDoc } from "./content";
import { t } from "./i18n";
import { render } from "./markdown";
import { langFor, type Page } from "./pages";
import type { Lang } from "./i18n";

function source(page: Page, lang: Lang): string {
  if (!page.doc) throw new Error(`${page.id} は markdown のページではない`);
  return readDoc(langFor(page, lang), page.doc);
}

/** markdown のページ。WebAssembly が読み込まれると色付きで組み直る。 */
export function Doc(props: { page: Page; lang: Lang }) {
  const rendered = createMemo(() => render(source(props.page, props.lang)));
  return <div innerHTML={rendered().html} />;
}

/** そのページの見出し。 */
export function Toc(props: { page: Page; lang: Lang }) {
  const headings = createMemo(() =>
    render(source(props.page, props.lang)).headings.filter((h) => h.level === 2 || h.level === 3),
  );
  return (
    <Show when={headings().length > 0}>
      <nav class="toc">
        <p class="toc-title">{t().tocTitle}</p>
        <For each={headings()}>
          {(heading) => (
            <a class={`h${heading.level}`} href={`#${heading.id}`}>
              {heading.text}
            </a>
          )}
        </For>
      </nav>
    </Show>
  );
}
