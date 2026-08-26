import { For, type JSX } from "solid-js";
import { PAGES, href, type Page } from "./pages";

export function Layout(props: { page: Page; children: JSX.Element; aside?: JSX.Element }) {
  return (
    <>
      <a class="skip" href="#main">
        本文へ
      </a>
      <header class="topbar">
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
      </header>

      <div class="shell">
        <main id="main" class="content">
          {props.children}
        </main>
        <aside class="side">{props.aside}</aside>
      </div>

      <footer class="foot">
        <p>
          <strong>Data Definition Schema Language</strong> is a DSL for Database Schema Design.
        </p>
      </footer>
    </>
  );
}
