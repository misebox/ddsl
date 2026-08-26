/**
 * markdown を HTML にする。
 *
 * コードブロックはコンパイラ本体の字句解析で色付けするので、
 * WebAssembly が読み込まれるまでは色が付かない。読み込み後に組み直す。
 */
import { Marked } from "marked";
import { wasm } from "./wasm";

export type Heading = { readonly level: number; readonly id: string; readonly text: string };
export type Rendered = { readonly html: string; readonly headings: readonly Heading[] };

function escapeHtml(text: string): string {
  return text.replace(/&/g, "&amp;").replace(/</g, "&lt;").replace(/>/g, "&gt;");
}

/** 見出しの id。日本語はそのまま残す。 */
function slug(text: string): string {
  const out = text
    .toLowerCase()
    .replace(/[^\p{L}\p{N}]+/gu, "-")
    .replace(/^-+|-+$/g, "");
  return out || "section";
}

/** リポジトリ内の相対リンクをページの URL に直す。 */
function rewriteLink(dest: string): string {
  if (/^[a-z]+:/i.test(dest) || dest.startsWith("#")) return dest;
  const file = dest.split("/").pop() ?? dest;
  return file.endsWith(".md") ? `./${file.slice(0, -3)}.html` : dest;
}

export function render(markdown: string): Rendered {
  const headings: Heading[] = [];
  const engine = new Marked({ gfm: true });

  engine.use({
    renderer: {
      heading({ tokens, depth }) {
        const text = this.parser.parseInline(tokens).replace(/<[^>]+>/g, "");
        const id = slug(text);
        headings.push({ level: depth, id, text });
        return `<h${depth} id="${id}"><a class="anchor" href="#${id}">#</a>${text}</h${depth}>`;
      },
      code({ text, lang }) {
        const ready = wasm();
        const label = lang || "nounsql";
        let inner: string;
        if (!ready) {
          inner = escapeHtml(text);
        } else if (lang === "sql") {
          inner = ready.highlightSql(text);
        } else if (!lang || lang === "nounsql" || lang === "nsql") {
          inner = ready.highlightSource(text);
        } else {
          inner = escapeHtml(text);
        }
        return `<figure class="code" data-lang="${escapeHtml(label)}"><pre><code>${inner}</code></pre></figure>`;
      },
      link({ href, title, tokens }) {
        const text = this.parser.parseInline(tokens);
        const attr = title ? ` title="${escapeHtml(title)}"` : "";
        return `<a href="${escapeHtml(rewriteLink(href))}"${attr}>${text}</a>`;
      },
    },
  });

  return { html: engine.parse(markdown, { async: false }), headings };
}
