import { render } from "solid-js/web";
import { Doc, Toc } from "./Doc";
import { Layout } from "./Layout";
import { Overview } from "./Overview";
import { Playground } from "./Playground";
import { Samples } from "./Samples";
import { isLang, setLang, title } from "./i18n";
import { pageById } from "./pages";
import { loadWasm } from "./wasm";
import "./styles.css";

const root = document.getElementById("app");
if (!root) throw new Error("#app が無い");

const page = pageById(root.dataset.page);
if (!page) throw new Error(`知らないページ: ${root.dataset.page}`);

const found = root.dataset.lang;
if (!isLang(found)) throw new Error(`知らない言語: ${found}`);
setLang(found);
const here = found;

document.title = title(here, page.id);

// ドキュメントの色付けにも使うので、どのページでも読み込む。
void loadWasm();

const current = page;
const body = () => {
  if (current.doc) return <Doc page={current} lang={here} />;
  if (current.id === "index") return <Overview />;
  return current.id === "samples" ? <Samples /> : <Playground />;
};

render(
  () => (
    <Layout
      page={current}
      aside={current.doc ? <Toc page={current} lang={here} /> : undefined}
      wide={current.id === "samples" || current.id === "playground"}
    >
      {body()}
    </Layout>
  ),
  root,
);
