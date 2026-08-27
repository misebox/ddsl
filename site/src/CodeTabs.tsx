import { For, Show, createMemo, createSignal } from "solid-js";
import { t } from "./i18n";
import { wasm } from "./wasm";

/**
 * ソースと生成された DDL をタブで見せる。
 *
 * コンパイルはブラウザの中で行うので、載せた例が必ず本物と一致する。
 */
export function CodeTabs(props: { source: string; dialect?: string }) {
  const [pane, setPane] = createSignal<"source" | "output">("source");

  const result = createMemo(() => {
    const ready = wasm();
    if (!ready) return undefined;
    try {
      return ready.compile(props.source, props.dialect ?? "postgres") as {
        sql: string;
        tables: number;
        columns: number;
        errors: number;
      };
    } catch {
      return undefined;
    }
  });

  const sourceHtml = () => wasm()?.highlightSource(props.source) ?? escapeHtml(props.source);
  const sqlHtml = () => {
    const sql = result()?.sql ?? "";
    return sql ? (wasm()?.highlightSql(sql) ?? escapeHtml(sql)) : "";
  };

  return (
    <div class="tabs">
      <div class="tabs-bar" role="tablist">
        <button
          type="button"
          role="tab"
          class="tabs-tab"
          classList={{ "is-on": pane() === "source" }}
          aria-selected={pane() === "source"}
          onClick={() => setPane("source")}
        >
          {t().code.source}
        </button>
        <button
          type="button"
          role="tab"
          class="tabs-tab"
          classList={{ "is-on": pane() === "output" }}
          aria-selected={pane() === "output"}
          onClick={() => setPane("output")}
        >
          {t().code.output}
        </button>
        <Show when={result()}>
          {(r) => (
            <span class="tabs-note">
              {r().errors > 0
                ? t().count.errors(r().errors)
                : t().count.size(r().tables, r().columns)}
            </span>
          )}
        </Show>
      </div>

      <pre class="tabs-body" hidden={pane() !== "source"}>
        <code innerHTML={sourceHtml()} />
      </pre>
      <pre class="tabs-body" hidden={pane() !== "output"}>
        <code innerHTML={sqlHtml()} />
      </pre>
    </div>
  );
}

function escapeHtml(text: string): string {
  return text.replace(/&/g, "&amp;").replace(/</g, "&lt;").replace(/>/g, "&gt;");
}
