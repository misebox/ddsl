import { For, createSignal } from "solid-js";
import { CodeTabs } from "./CodeTabs";
import { readExample } from "./content";
import { lang, t } from "./i18n";
import { samples } from "./sampleList";

/** 左に一覧、右に中身。一覧は畳まない。 */
export function Samples() {
  const list = samples(lang());
  const [current, setCurrent] = createSignal(list[0]!);

  return (
    <>
      <h1>{t().samples.heading}</h1>
      <p class="page-intro">{t().samples.intro()}</p>

      <div class="samples">
        <nav class="samples-list" aria-label={t().samples.listLabel}>
          <For each={list}>
            {(sample) => (
              <button
                type="button"
                class="samples-item"
                classList={{ "is-on": current().file === sample.file }}
                onClick={() => setCurrent(sample)}
              >
                <span class="samples-item-label">{sample.label}</span>
                <span class="samples-item-file">{sample.file}</span>
              </button>
            )}
          </For>
        </nav>

        <section class="samples-body">
          <p class="samples-summary">{current().summary}</p>
          <CodeTabs source={readExample(current().file)} />
        </section>
      </div>
    </>
  );
}
