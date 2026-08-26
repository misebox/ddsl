import { For, Show, createMemo } from "solid-js";
import { readDoc } from "./content";
import { render } from "./markdown";
import type { Page } from "./pages";

/** markdown のページ。WebAssembly が読み込まれると色付きで組み直る。 */
export function Doc(props: { page: Page }) {
  const rendered = createMemo(() => render(readDoc(props.page.file)));
  return <div innerHTML={rendered().html} />;
}

/** そのページの見出し。 */
export function Toc(props: { page: Page }) {
  const headings = createMemo(() =>
    render(readDoc(props.page.file)).headings.filter((h) => h.level === 2 || h.level === 3),
  );
  return (
    <Show when={headings().length > 0}>
      <nav class="toc">
        <p class="toc-title">On this page</p>
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
