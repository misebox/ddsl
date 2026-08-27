import { readFileSync } from "node:fs";
import { resolve } from "node:path";
import type { Plugin } from "vite";
import { LLMS_INTRO } from "./llmsIntro";
import { SAMPLES } from "./sampleList";

const SITE = "https://misebox.github.io/nounsql/";

/** llms.txt から辿れるドキュメント。並び順は読ませたい順。 */
const DOCS = [
  { file: "spec.md", title: "Specification", summary: "Every statement and attribute, the resolution order, and the validations." },
  { file: "guide.md", title: "Guide", summary: "Installing the compiler, the CLI, diagnostics, and the JavaScript API." },
  { file: "tooling.md", title: "How it works", summary: "The compilation pipeline, where generated names come from, and the WebAssembly build." },
];

const root = (...parts: string[]) => resolve(import.meta.dirname, "../..", ...parts);
const read = (...parts: string[]) => readFileSync(root(...parts), "utf8");

function index(): string {
  const docs = DOCS.map((d) => `- [${d.title}](${SITE}${d.file}): ${d.summary}`);
  const samples = SAMPLES.map(
    (s) => `- [${s.label}](${SITE}examples/${s.file}): ${s.summary}`,
  );
  return [
    LLMS_INTRO,
    "## Docs",
    "",
    docs.join("\n"),
    "",
    "## Examples",
    "",
    "Each one compiles as it stands.",
    "",
    samples.join("\n"),
    "",
    "## Optional",
    "",
    `- [Everything above in one file](${SITE}llms-full.txt): the docs and the examples inlined.`,
    "",
  ].join("\n");
}

function full(): string {
  const docs = DOCS.map((d) => `${read("docs", d.file)}\n`);
  const samples = SAMPLES.map(
    (s) => `## examples/${s.file}\n\n${s.summary}\n\n\`\`\`\n${read("examples", s.file)}\`\`\`\n`,
  );
  return [
    LLMS_INTRO,
    "---",
    "",
    docs.join("\n---\n\n"),
    "---",
    "",
    "# Examples",
    "",
    samples.join("\n"),
  ].join("\n");
}

/**
 * llms.txt / llms-full.txt と、そこから辿る素の Markdown と例を出す。
 * どれも docs/ と examples/ の実物から作るので、写しが古くなることがない。
 */
export function llmsTxt(): Plugin {
  return {
    name: "nounsql-llms-txt",
    apply: "build",
    generateBundle() {
      const emit = (fileName: string, source: string) =>
        this.emitFile({ type: "asset", fileName, source });

      emit("llms.txt", index());
      emit("llms-full.txt", full());
      for (const doc of DOCS) emit(doc.file, read("docs", doc.file));
      for (const sample of SAMPLES) emit(`examples/${sample.file}`, read("examples", sample.file));
    },
  };
}
