import { render } from "solid-js/web";
import { Doc, Toc } from "./Doc";
import { Layout } from "./Layout";
import { Playground } from "./Playground";
import { pageById } from "./pages";
import { loadWasm } from "./wasm";
import "./styles.css";

const root = document.getElementById("app");
if (!root) throw new Error("#app が無い");

const page = pageById(root.dataset.page ?? "index");
if (!page) throw new Error(`知らないページ: ${root.dataset.page}`);

document.title = page.title;

// ドキュメントの色付けにも使うので、どのページでも読み込む。
void loadWasm();

render(
  () => (
    <Layout page={page} aside={page.file ? <Toc page={page} /> : undefined}>
      {page.file ? <Doc page={page} /> : <Playground />}
    </Layout>
  ),
  root,
);
