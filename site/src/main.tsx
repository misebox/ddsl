import { render } from "solid-js/web";
import { Doc, Toc } from "./Doc";
import { Layout } from "./Layout";
import { Overview } from "./Overview";
import { Playground } from "./Playground";
import { Samples } from "./Samples";
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

const current = page;
const body = () => {
  if (current.file) return <Doc page={current} />;
  if (current.id === "index") return <Overview />;
  return current.id === "samples" ? <Samples /> : <Playground />;
};

render(
  () => (
    <Layout
      page={current}
      aside={current.file ? <Toc page={current} /> : undefined}
      wide={current.id === "samples" || current.id === "playground"}
    >
      {body()}
    </Layout>
  ),
  root,
);
