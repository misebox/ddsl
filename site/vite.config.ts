import { resolve } from "node:path";
import { defineConfig } from "vite";
import solid from "vite-plugin-solid";
import { writeEntries } from "./src/entries.ts";
import { llmsTxt } from "./src/llms.ts";

// ドキュメントは静的なページの方が向いているので SPA にせず、
// HTML を1ページ1ファイル出す。ページ間の移動は素の <a> で足りる。
// エントリはページ × 言語の分だけ要るので、雛形から書き出す。
const root = import.meta.dirname;
const input = writeEntries(root);

export default defineConfig({
  base: "./",
  plugins: [solid(), llmsTxt()],
  build: {
    outDir: resolve(root, "../dist"),
    emptyOutDir: true,
    rollupOptions: { input },
  },
  server: {
    // docs/ と examples/ はサイトの外にあるが、内容はここから読む。
    fs: { allow: [resolve(root, "..")] },
  },
});
