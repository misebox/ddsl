import { For, Show, batch, createEffect, createSignal, onMount } from "solid-js";
import { readExample } from "./content";
import { failed, wasm } from "./wasm";

type Sample = { readonly file: string; readonly label: string; readonly note: string };

/** examples/ にある実物を読む。ここには説明だけを持つ。 */
const SAMPLES: readonly Sample[] = [
  { file: "minimal.nsql", label: "最小", note: "nouns / mixin / belongs_to だけの構成" },
  { file: "relations.nsql", label: "関連", note: "同一テーブルへの複数参照、自己参照、複合主キー" },
  { file: "blueprint.nsql", label: "blueprint", note: "1回の適用で3テーブルを作る" },
  { file: "config.nsql", label: "config", note: "命名規則と制約の既定値を変えると出力がどう変わるか" },
  { file: "sample.nsql", label: "全構文", note: "仕様に出てくる構文をすべて含む" },
];

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
    if (failed()) return { text: "コンパイラを読み込めなかった", kind: "is-error" };
    if (error()) return { text: error(), kind: "is-error" };
    const r = result();
    if (!r) return { text: "読み込み中…", kind: "" };
    if (r.errors > 0) return { text: `エラー ${r.errors} 件`, kind: "is-error" };
    const size = `${r.tables} テーブル / ${r.columns} 列`;
    if (r.warnings > 0) return { text: `${size} · 警告 ${r.warnings} 件`, kind: "is-warning" };
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
      setError("クリップボードに書けなかった");
    }
  }

  function syncScroll() {
    highlighted.scrollTop = editor.scrollTop;
    highlighted.scrollLeft = editor.scrollLeft;
  }

  return (
    <>
      <h1>プレイグラウンド</h1>
      <p>
        ブラウザの中でコンパイルする。入力はどこにも送られない。コンパイラは{" "}
        <a href="https://github.com/misebox/nounsql/tree/main/crates/nounsql-wasm">
          WebAssembly に落とした本体
        </a>
        で、CLI と同じ字句解析・解決・コード生成を使う。
      </p>

      <div class="pg" data-state={failed() ? "failed" : wasm() ? "ready" : "loading"}>
        <nav class="pg-samples" aria-label="サンプル">
          <For each={SAMPLES}>
            {(sample) => (
              <button
                type="button"
                class="pg-sample"
                classList={{ "is-on": current() === sample.file }}
                onClick={() => select(sample.file)}
              >
                <span class="pg-sample-label">{sample.label}</span>
                <span class="pg-sample-note">{sample.note}</span>
              </button>
            )}
          </For>
        </nav>

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
              ソース
            </button>
            <button
              type="button"
              role="tab"
              class="pg-tab"
              classList={{ "is-on": pane() === "output" }}
              aria-selected={pane() === "output"}
              onClick={() => setPane("output")}
            >
              出力
              <Show when={(result()?.errors ?? 0) > 0}>
                <span class="pg-badge">{result()?.errors}</span>
              </Show>
            </button>
          </div>

          <label class="pg-field">
            <span>ターゲット</span>
            <select value={dialect()} onChange={(e) => setDialect(e.currentTarget.value)}>
              <For each={wasm()?.dialects() ?? ["postgres"]}>
                {(name) => <option value={name}>{name}</option>}
              </For>
            </select>
          </label>

          <Show when={pane() === "output"}>
            <button type="button" class="pg-copy" onClick={copySql}>
              {copied() ? "コピーした" : "DDL をコピー"}
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
              aria-label="NounSQL のソース"
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
            <header class="pg-head">診断</header>
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
