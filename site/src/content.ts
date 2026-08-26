/**
 * ドキュメントと例の中身。
 *
 * どちらもサイトの外（`docs/` と `examples/`）にある実物を読む。
 * ここで写しを持つと、仕様と説明がずれる。
 */
const docs = import.meta.glob("../../docs/*.md", {
  query: "?raw",
  import: "default",
  eager: true,
}) as Record<string, string>;

const examples = import.meta.glob("../../examples/*.nsql", {
  query: "?raw",
  import: "default",
  eager: true,
}) as Record<string, string>;

function byBasename(files: Record<string, string>): Map<string, string> {
  return new Map(
    Object.entries(files).map(([path, text]) => [path.split("/").pop() ?? path, text]),
  );
}

const docByName = byBasename(docs);
const exampleByName = byBasename(examples);

export function readDoc(file: string): string {
  const text = docByName.get(file);
  if (text === undefined) {
    throw new Error(`docs/${file} が無い`);
  }
  return text;
}

export function readExample(file: string): string {
  return exampleByName.get(file) ?? `# ${file} を読めなかった\n`;
}
