import { For, createSignal } from "solid-js";
import { CodeTabs } from "./CodeTabs";
import { readExample } from "./content";
import { SAMPLES } from "./sampleList";

/** 左に一覧、右に中身。一覧は畳まない。 */
export function Samples() {
  const [current, setCurrent] = createSignal(SAMPLES[0]!);

  return (
    <>
      <h1>サンプル</h1>
      <p>
        すべて警告なしで通る。同じものが <code>examples/</code> にあり、
        <a href="./playground.html">プレイグラウンド</a>で書き換えて試せる。
      </p>

      <div class="samples">
        <nav class="samples-list" aria-label="サンプル">
          <For each={SAMPLES}>
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
