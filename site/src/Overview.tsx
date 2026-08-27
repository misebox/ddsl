import { For, Show } from "solid-js";
import { CodeTabs } from "./CodeTabs";
import { readExample } from "./content";
import { lang, t } from "./i18n";
import { CONSTRUCTS, OVERVIEW, type Links } from "./overviewCopy";
import { DEFAULT_LANG } from "./lang";
import { href, hasLang, langFor, pageById } from "./pages";

export function Overview() {
  const here = lang();
  const copy = OVERVIEW[here];
  const to = (id: Parameters<typeof href>[0]) => {
    const page = pageById(id);
    return href(id, here, page ? langFor(page, here) : here);
  };
  const links: Links = {
    spec: to("spec"),
    // 文法は言語ごとの場所にも書き出している。llms.txt はルートに1つだけ。
    gbnf: "./nounsql.gbnf",
    llms: here === DEFAULT_LANG ? "./llms.txt" : "../llms.txt",
  };

  return (
    <>
      <section class="hero">
        <h1>{copy.heading}</h1>

        <p class="hero-lede">{copy.lede}</p>

        <div class="hero-actions">
          <a class="button" href={to("playground")}>
            {copy.tryIt}
          </a>
          <a class="button is-quiet" href={to("guide")}>
            {copy.install}
          </a>
        </div>
      </section>

      <CodeTabs source={readExample("minimal.nsql")} />

      <h2 id="where-the-repetition-goes">{copy.repetition.heading}</h2>
      <p>{copy.repetition.body()}</p>

      <h3 id="mixin">{copy.mixin.heading}</h3>
      <p>{copy.mixin.body()}</p>

      <h3 id="naming">{copy.naming.heading}</h3>
      <p>{copy.naming.body()}</p>

      <h3 id="nouns">{copy.nouns.heading}</h3>
      {copy.nouns.body()}

      <h2 id="each-construct-produces-one-thing">{copy.constructs.heading}</h2>
      <p>{copy.constructs.body()}</p>
      <table>
        <thead>
          <tr>
            <For each={copy.constructs.columns}>{(head) => <th>{head}</th>}</For>
          </tr>
        </thead>
        <tbody>
          <For each={CONSTRUCTS}>
            {(name) => (
              <tr>
                <td>
                  <code>{name}</code>
                </td>
                <For each={copy.constructs.rows[name]}>{(cell) => <td>{cell}</td>}</For>
              </tr>
            )}
          </For>
        </tbody>
      </table>

      <h2 id="the-output-is-just-sql">{copy.output.heading}</h2>
      <p>{copy.output.body()}</p>

      <h2 id="writing-it-with-a-model">{copy.generated.heading}</h2>
      <p>{copy.generated.body(links)}</p>

      <h2 id="next">{copy.next.heading}</h2>
      <ul>
        <For each={copy.next.items}>
          {(item) => {
            const page = pageById(item.id);
            return (
              <li>
                <a href={to(item.id)}>{item.label}</a> — {item.note}
                <Show when={page && !hasLang(page, here)}>
                  {" "}
                  <span class="nav-lang">{t().englishOnly}</span>
                </Show>
              </li>
            );
          }}
        </For>
      </ul>
    </>
  );
}
