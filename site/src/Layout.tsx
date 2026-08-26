import { For, Show, type JSX } from "solid-js";
import { PAGES, href, type Page } from "./pages";

export function Layout(props: {
  page: Page;
  children: JSX.Element;
  aside?: JSX.Element;
  wide?: boolean;
}) {
  return (
    <>
      <a class="skip" href="#main">
        本文へ
      </a>
      <header class="topbar">
        <div class="topbar-inner">
          <a class="brand" href="./index.html">
            <span class="brand-mark">NounSQL</span>
          </a>
          <nav class="topnav">
            <For each={PAGES}>
              {(page) => (
                <a href={href(page)} class={page.id === props.page.id ? "current" : undefined}>
                  {page.nav}
                </a>
              )}
            </For>
          </nav>
          <a class="repo" href="https://github.com/misebox/nounsql">
            GitHub
          </a>
        </div>
      </header>

      <div class="shell" classList={{ "is-wide": props.wide }}>
        <main id="main" class="content">
          {props.children}
        </main>
        <Show when={props.aside}>
          <aside class="side">{props.aside}</aside>
        </Show>
      </div>

      <footer class="foot">
        <p>Data Definition Schema Language is a DSL for Database Schema Design.</p>
      </footer>
    </>
  );
}
