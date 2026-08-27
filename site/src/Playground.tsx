import { For, Show, batch, createEffect, createSignal, onMount } from "solid-js";
import { readExample } from "./content";
import { SAMPLES } from "./sampleList";
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
  const [source, setSource] = createSignal("");
  const [current, setCurrent] = createSignal(SAMPLES[0]!.file);
  const [dialect, setDialect] = createSignal("postgres");
  const [pane, setPane] = createSignal<"source" | "output">("source");
  const [result, setResult] = createSignal<Result | undefined>();
  const [error, setError] = createSignal("");
  const [copied, setCopied] = createSignal(false);

  let editor!: HTMLTextAreaElement;
  let highlighted!: HTMLPreElement;

  onMount(() => select(SAMPLES[0]!.file));

  function select(file: string) {
    batch(() => {
      setCurrent(file);
      setSource(readExample(file));
    });
    editor?.scrollTo({ top: 0 });
  }

  // 入力・ターゲット・WebAssembly の読み込みのいずれかが変わったら組み直す。
  createEffect(() => {
    const ready = wasm();
    const text = source();
    if (!ready) return;
    try {
      setResult(ready.compile(text, dialect()) as Result);
      setError("");
    } catch (e) {
      setError(String(e));
    }
  });

  const sourceHtml = () => wasm()?.highlightSource(source()) ?? escapeHtml(source());
  const sqlHtml = () => {
    const sql = result()?.sql ?? "";
    return sql ? (wasm()?.highlightSql(sql) ?? escapeHtml(sql)) : "";
  };

  function status(): { text: string; kind: string } {
    if (failed()) return { text: "compiler failed to load", kind: "is-error" };
    if (error()) return { text: error(), kind: "is-error" };
    const r = result();
    if (!r) return { text: "loading", kind: "" };
    if (r.errors > 0) return { text: `${r.errors} error${r.errors > 1 ? "s" : ""}`, kind: "is-error" };
    const size = `${r.tables} tables · ${r.columns} columns`;
    if (r.warnings > 0)
      return { text: `${size} · ${r.warnings} warning${r.warnings > 1 ? "s" : ""}`, kind: "is-warning" };
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
      setError("could not write to the clipboard");
    }
  }

  function syncScroll() {
    highlighted.scrollTop = editor.scrollTop;
    highlighted.scrollLeft = editor.scrollLeft;
  }

  return (
    <>
      <h1>Playground</h1>
      <p class="page-intro">
        Compiles in the browser. Nothing you type is sent anywhere. It runs the{" "}
        <a href="https://github.com/misebox/nounsql/tree/main/crates/nounsql-wasm">
          compiler itself, built to WebAssembly
        </a>
        — the same lexer, resolver and code generator the CLI uses. The examples are described on
        the <a href="./samples.html">samples</a> page.
      </p>

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
              source
            </button>
            <button
              type="button"
              role="tab"
              class="pg-tab"
              classList={{ "is-on": pane() === "output" }}
              aria-selected={pane() === "output"}
              onClick={() => setPane("output")}
            >
              output
              <Show when={(result()?.errors ?? 0) > 0}>
                <span class="pg-badge">{result()?.errors}</span>
              </Show>
            </button>
          </div>

          <label class="pg-field">
            <span>example</span>
            <select value={current()} onChange={(e) => select(e.currentTarget.value)}>
              <For each={SAMPLES}>
                {(sample) => <option value={sample.file}>{sample.label}</option>}
              </For>
            </select>
          </label>

          <label class="pg-field">
            <span>target</span>
            <select value={dialect()} onChange={(e) => setDialect(e.currentTarget.value)}>
              <For each={wasm()?.dialects() ?? ["postgres"]}>
                {(name) => <option value={name}>{name}</option>}
              </For>
            </select>
          </label>

          <Show when={pane() === "output"}>
            <button type="button" class="pg-copy" onClick={copySql}>
              {copied() ? "Copied" : "Copy DDL"}
            </button>
          </Show>

          <span class={`pg-status ${status().kind}`}>{status().text}</span>
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
              aria-label="NounSQL source"
              value={source()}
              onInput={(e) => setSource(e.currentTarget.value)}
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
            <header class="pg-head">Diagnostics</header>
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
