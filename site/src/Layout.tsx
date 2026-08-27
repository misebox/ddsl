import { For, Show, type JSX } from "solid-js";
import { LANGS, LANG_LABEL, lang, t } from "./i18n";
import { PAGES, hasLang, href, langFor, type Page } from "./pages";

export function Layout(props: {
  page: Page;
  children: JSX.Element;
  aside?: JSX.Element;
  wide?: boolean;
}) {
  const here = lang();

  return (
    <>
      <a class="skip" href="#main">
        {t().skip}
      </a>
      <header class="topbar">
        <div class="topbar-inner">
          <a class="brand" href={href("index", here, here)}>
            <span class="brand-mark">NounSQL</span>
          </a>
          <nav class="topnav">
            <For each={PAGES}>
              {(page) => {
                // 訳の無いページは、読める言語の版へ送る。
                const to = langFor(page, here);
                return (
                  <a
                    href={href(page.id, here, to)}
                    class={page.id === props.page.id ? "current" : undefined}
                  >
                    {t().nav[page.id]}
                    <Show when={!hasLang(page, here)}>
                      <span class="nav-lang" title={t().englishOnly}>
                        {to}
                      </span>
                    </Show>
                  </a>
                );
              }}
            </For>
          </nav>
          <nav class="langs" aria-label={LANG_LABEL[here]}>
            <For each={LANGS}>
              {(to) => (
                <a
                  href={href(langFor(props.page, to) === to ? props.page.id : "index", here, to)}
                  class={to === here ? "current" : undefined}
                  lang={to}
                >
                  {LANG_LABEL[to]}
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
        <p>{t().tagline}</p>
      </footer>
    </>
  );
}
