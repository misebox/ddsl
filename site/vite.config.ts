import { resolve } from "node:path";
import { defineConfig } from "vite";
import solid from "vite-plugin-solid";

// ドキュメントは静的なページの方が向いているので SPA にせず、
// HTML を1ページ1ファイル出す。ページ間の移動は素の <a> で足りる。
const pages = ["index", "spec", "tooling", "playground"];

export default defineConfig({
  base: "./",
  plugins: [solid()],
  build: {
    outDir: resolve(import.meta.dirname, "../dist"),
    emptyOutDir: true,
    rollupOptions: {
      input: Object.fromEntries(
        pages.map((name) => [name, resolve(import.meta.dirname, `${name}.html`)]),
      ),
    },
  },
  server: {
    // docs/ と examples/ はサイトの外にあるが、内容はここから読む。
    fs: { allow: [resolve(import.meta.dirname, "..")] },
  },
});
