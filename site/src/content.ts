/**
 * ドキュメントと例の中身。
 *
 * どちらもサイトの外（`docs/` と `examples/`）にある実物を読む。
 * ここで写しを持つと、仕様と説明がずれる。
 */
import type { Lang } from "./i18n";

const docs = import.meta.glob("../../docs/*/*.md", {
  query: "?raw",
  import: "default",
  eager: true,
}) as Record<string, string>;

const examples = import.meta.glob("../../examples/*.nsql", {
  query: "?raw",
  import: "default",
  eager: true,
}) as Record<string, string>;

/** `../../docs/ja/guide.md` を `ja/guide.md` にする。 */
const docByPath = new Map(
  Object.entries(docs).map(([path, text]) => [path.split("/").slice(-2).join("/"), text]),
);

const exampleByName = new Map(
  Object.entries(examples).map(([path, text]) => [path.split("/").pop() ?? path, text]),
);

export function readDoc(lang: Lang, file: string): string {
  const text = docByPath.get(`${lang}/${file}`);
  if (text === undefined) {
    throw new Error(`docs/${lang}/${file} が無い`);
  }
  return text;
}

export function readExample(file: string): string {
  return exampleByName.get(file) ?? `# ${file} を読めなかった\n`;
}
