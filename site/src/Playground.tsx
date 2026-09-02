import { For, Show, batch, createEffect, createSignal, on, onMount } from "solid-js";
import { readExample } from "./content";
import { lang, t } from "./i18n";
import { samples } from "./sampleList";
import { failed, wasm } from "./wasm";

type Diagnostic = {
  severity: "error" | "warning";
  message: string;
  line: number;
  column: number;
};

type Result = {
  sql: string;
  diagnostics: Diagnostic[];
  tables: number;
  columns: number;
  errors: number;
  warnings: number;
};

export function Playground() {
  const list = samples(lang());
  const [source, setSource] = createSignal("");
  const [current, setCurrent] = createSignal(list[0]!.file);
  const [dialect, setDialect] = createSignal("postgres");
  const [pane, setPane] = createSignal<"source" | "output">("source");
  const [result, setResult] = createSignal<Result | undefined>();
  const [error, setError] = createSignal("");
  const [copied, setCopied] = createSignal(false);
  // 前回組んでからソースが変わったか。
  const [stale, setStale] = createSignal(false);

  let editor!: HTMLTextAreaElement;
  let highlighted!: HTMLPreElement;

  onMount(() => select(list[0]!.file));

  function select(file: string) {
    batch(() => {
      setCurrent(file);
      setSource(readExample(file));
    });
    editor?.scrollTo({ top: 0 });
    compile();
  }

  // 組むのは頼まれたときだけ。入力途中のソースはたいてい通らない。
  function compile() {
    const ready = wasm();
    if (!ready) return;
    try {
      setResult(ready.compile(source(), dialect()) as Result);
      setError("");
    } catch (e) {
      setError(String(e));
    }
    setStale(false);
  }

  // WebAssembly が届いたら、選んであるサンプルを一度組む。
  createEffect(on(wasm, () => compile()));

  const sourceHtml = () => {
    const text = source();
    const html = wasm()?.highlightSource(text) ?? escapeHtml(text);
    // A textarea draws an empty line after a trailing newline and the <pre>
    // behind it does not, which leaves the two one line out of step once the
    // editor is scrolled. Give the pre that line back.
    return text.endsWith("\n") ? `${html} ` : html;
  };
  const sqlHtml = () => {
    const sql = result()?.sql ?? "";
    return sql ? (wasm()?.highlightSql(sql) ?? escapeHtml(sql)) : "";
  };

  function status(): { text: string; kind: string } {
    if (failed()) return { text: t().playground.loadFailed, kind: "is-error" };
    if (error()) return { text: error(), kind: "is-error" };
    const r = result();
    if (!r) return { text: t().playground.loading, kind: "" };
    if (r.errors > 0) return { text: t().count.errors(r.errors), kind: "is-error" };
    const size = t().count.size(r.tables, r.columns);
    if (r.warnings > 0)
      return { text: `${size} · ${t().count.warnings(r.warnings)}`, kind: "is-warning" };
    return { text: size, kind: "is-ok" };
  }

  function jumpTo(line: number, column: number) {
    setPane("source");
    const lines = source().split("\n");
    let offset = 0;
    for (let i = 0; i < line - 1 && i < lines.length; i++) offset += lines[i]!.length + 1;
    editor.focus();
    editor.setSelectionRange(offset + column - 1, offset + column - 1);
  }

  async function copySql() {
    try {
      await navigator.clipboard.writeText(result()?.sql ?? "");
      setCopied(true);
      setTimeout(() => setCopied(false), 1500);
    } catch {
      setError(t().playground.clipboardFailed);
    }
  }

  function syncScroll() {
    highlighted.scrollTop = editor.scrollTop;
    highlighted.scrollLeft = editor.scrollLeft;
  }

  return (
    <>
      <h1>{t().playground.heading}</h1>
      <p class="page-intro">{t().playground.intro()}</p>

      <div class="pg" data-state={failed() ? "failed" : wasm() ? "ready" : "loading"}>
        <div class="pg-bar">
          <div class="pg-tabs" role="tablist">
            <button
              type="button"
              role="tab"
              class="pg-tab"
              classList={{ "is-on": pane() === "source" }}
              aria-selected={pane() === "source"}
              onClick={() => setPane("source")}
            >
              {t().playground.source}
            </button>
            <button
              type="button"
              role="tab"
              class="pg-tab"
              classList={{ "is-on": pane() === "output" }}
              aria-selected={pane() === "output"}
              onClick={() => setPane("output")}
            >
              {t().playground.output}
              <Show when={(result()?.errors ?? 0) > 0}>
                <span class="pg-badge">{result()?.errors}</span>
              </Show>
            </button>
          </div>

          <label class="pg-field">
            <span>{t().playground.example}</span>
            <select value={current()} onChange={(e) => select(e.currentTarget.value)}>
              <For each={list}>
                {(sample) => <option value={sample.file}>{sample.label}</option>}
              </For>
            </select>
          </label>

          <label class="pg-field">
            <span>{t().playground.target}</span>
            <select
              value={dialect()}
              onChange={(e) => {
                setDialect(e.currentTarget.value);
                compile();
              }}
            >
              <For each={wasm()?.dialects() ?? ["postgres"]}>
                {(name) => <option value={name}>{name}</option>}
              </For>
            </select>
          </label>

          <button type="button" class="pg-button" onClick={() => compile()}>
            {t().playground.compile}
          </button>

          <Show when={pane() === "output"}>
            <button type="button" class="pg-button" onClick={copySql}>
              {copied() ? t().playground.copied : t().playground.copy}
            </button>
          </Show>

          <span class={`pg-status ${status().kind}`} classList={{ "is-stale": stale() }}>
            {status().text}
          </span>
        </div>

        <div class="pg-pane" hidden={pane() !== "source"}>
          <div class="pg-editor">
            <pre class="pg-hl" ref={highlighted} aria-hidden="true">
              <code innerHTML={sourceHtml()} />
            </pre>
            <textarea
              ref={editor}
              spellcheck={false}
              autocapitalize="off"
              autocorrect="off"
              aria-label={t().playground.editorLabel}
              value={source()}
              onInput={(e) => {
                setSource(e.currentTarget.value);
                setStale(true);
              }}
              onKeyDown={(e) => {
                if (e.key === "Enter" && (e.metaKey || e.ctrlKey)) compile();
              }}
              onScroll={syncScroll}
            />
          </div>
        </div>

        <div class="pg-pane" hidden={pane() !== "output"}>
          <pre class="pg-out">
            <code innerHTML={sqlHtml()} />
          </pre>
        </div>

        <Show when={(result()?.diagnostics.length ?? 0) > 0}>
          <section class="pg-diags">
            <header class="pg-head">{t().playground.diagnostics}</header>
            <ul>
              <For each={result()!.diagnostics}>
                {(d) => (
                  <li class={d.severity}>
                    <button type="button" class="pg-where" onClick={() => jumpTo(d.line, d.column)}>
                      {d.line}:{d.column}
                    </button>
                    <span>{d.message}</span>
                  </li>
                )}
              </For>
            </ul>
          </section>
        </Show>
      </div>
    </>
  );
}

function escapeHtml(text: string): string {
  return text.replace(/&/g, "&amp;").replace(/</g, "&lt;").replace(/>/g, "&gt;");
}
