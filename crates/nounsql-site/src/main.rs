use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use pulldown_cmark::{CodeBlockKind, CowStr, Event, HeadingLevel, Options, Parser, Tag, TagEnd};

use nounsql_core::highlight::{escape, sql_to_html, to_html as nounsql_to_html};

struct Page {
    source: &'static str,
    output: &'static str,
    title: &'static str,
    nav: &'static str,
}

const PAGES: &[Page] = &[
    Page {
        source: "index.md",
        output: "index.html",
        title: "NounSQL",
        nav: "概要",
    },
    Page {
        source: "spec.md",
        output: "spec.html",
        title: "仕様",
        nav: "仕様",
    },
    Page {
        source: "tooling.md",
        output: "tooling.html",
        title: "ツール",
        nav: "ツール",
    },
    // markdown ではなく、直接書いた HTML から作る。
    Page {
        source: "",
        output: "playground.html",
        title: "プレイグラウンド",
        nav: "プレイグラウンド",
    },
];

struct Heading {
    level: u8,
    id: String,
    text: String,
}

fn main() -> Result<()> {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
    let docs = root.join("docs");
    let out = root.join("dist");
    fs::create_dir_all(&out).context("dist を作れない")?;

    for page in PAGES {
        let (body, headings) = if page.source.is_empty() {
            (include_str!("playground.html").to_string(), Vec::new())
        } else {
            let md = fs::read_to_string(docs.join(page.source))
                .with_context(|| format!("読めない: docs/{}", page.source))?;
            render_markdown(&md)
        };
        let html = layout(page, &body, &headings);
        fs::write(out.join(page.output), html)
            .with_context(|| format!("書けない: dist/{}", page.output))?;
        println!("dist/{}", page.output);
    }

    fs::write(out.join("style.css"), include_str!("style.css"))?;
    // Jekyll に処理させない。
    fs::write(out.join(".nojekyll"), "")?;
    copy_if_exists(&root.join("docs/nounsql.gbnf"), &out.join("nounsql.gbnf"))?;
    copy_if_exists(&root.join("examples/sample.nsql"), &out.join("sample.nsql"))?;

    // プレイグラウンド用の WebAssembly。無ければページ側で読み込み失敗を表示する。
    let pkg = root.join("crates/nounsql-wasm/pkg");
    for name in ["nounsql_wasm.js", "nounsql_wasm_bg.wasm"] {
        if !copy_if_exists(&pkg.join(name), &out.join(name))? {
            eprintln!("警告: {name} が無い。wasm-pack build を先に実行する");
        }
    }
    Ok(())
}

fn copy_if_exists(from: &Path, to: &Path) -> Result<bool> {
    if !from.exists() {
        return Ok(false);
    }
    fs::copy(from, to)?;
    Ok(true)
}

fn render_markdown(md: &str) -> (String, Vec<Heading>) {
    let mut options = Options::empty();
    options.insert(Options::ENABLE_TABLES);
    options.insert(Options::ENABLE_STRIKETHROUGH);

    let mut headings: Vec<Heading> = Vec::new();
    let mut events: Vec<Event> = Vec::new();

    let mut code: Option<(String, String)> = None;
    let mut heading: Option<(u8, String)> = None;

    for event in Parser::new_ext(md, options) {
        // コードブロックは中身を貯めてから自前で描画する。
        if let Some((lang, buf)) = &mut code {
            match event {
                Event::Text(t) => {
                    buf.push_str(&t);
                    continue;
                }
                Event::End(TagEnd::CodeBlock) => {
                    events.push(Event::Html(CowStr::Boxed(
                        code_block(lang, buf).into_boxed_str(),
                    )));
                    code = None;
                    continue;
                }
                _ => continue,
            }
        }
        // 見出しは目次のためにテキストを集める。
        if let Some((_, text)) = &mut heading {
            match &event {
                Event::Text(t) => text.push_str(t),
                Event::Code(t) => text.push_str(t),
                Event::End(TagEnd::Heading(_)) => {
                    let (level, text) = heading.take().unwrap_or((2, String::new()));
                    let id = slug(&text);
                    events.push(Event::Html(CowStr::Boxed(
                        format!(
                            "<h{level} id=\"{id}\"><a class=\"anchor\" href=\"#{id}\">#</a>{}</h{level}>",
                            escape(&text)
                        )
                        .into_boxed_str(),
                    )));
                    headings.push(Heading { level, id, text });
                    continue;
                }
                _ => {}
            }
            continue;
        }

        match event {
            Event::Start(Tag::CodeBlock(kind)) => {
                let lang = match kind {
                    CodeBlockKind::Fenced(l) => l.to_string(),
                    CodeBlockKind::Indented => String::new(),
                };
                code = Some((lang, String::new()));
            }
            Event::Start(Tag::Heading { level, .. }) => {
                heading = Some((heading_level(level), String::new()));
            }
            Event::Start(Tag::Link {
                link_type,
                dest_url,
                title,
                id,
            }) => events.push(Event::Start(Tag::Link {
                link_type,
                dest_url: CowStr::Boxed(rewrite_link(&dest_url).into_boxed_str()),
                title,
                id,
            })),
            other => events.push(other),
        }
    }

    let mut body = String::new();
    pulldown_cmark::html::push_html(&mut body, events.into_iter());
    (body, headings)
}

fn heading_level(level: HeadingLevel) -> u8 {
    match level {
        HeadingLevel::H1 => 1,
        HeadingLevel::H2 => 2,
        HeadingLevel::H3 => 3,
        HeadingLevel::H4 => 4,
        HeadingLevel::H5 => 5,
        HeadingLevel::H6 => 6,
    }
}

/// リポジトリ内の相対リンクをページのURLに直す。
fn rewrite_link(dest: &str) -> String {
    if dest.starts_with("http") || dest.starts_with('#') {
        return dest.to_string();
    }
    let file = dest.rsplit('/').next().unwrap_or(dest);
    match file.strip_suffix(".md") {
        Some(stem) if PAGES.iter().any(|p| p.source == file) => format!("{stem}.html"),
        _ => dest.to_string(),
    }
}

fn slug(text: &str) -> String {
    let mut out = String::new();
    let mut dash = false;
    for ch in text.chars() {
        if ch.is_alphanumeric() {
            out.extend(ch.to_lowercase());
            dash = false;
        } else if !dash && !out.is_empty() {
            out.push('-');
            dash = true;
        }
    }
    out.trim_end_matches('-').to_string()
}

fn code_block(lang: &str, code: &str) -> String {
    let inner = match lang {
        "" | "nounsql" => nounsql_to_html(code),
        "sql" => sql_to_html(code),
        _ => escape(code),
    };
    let label = if lang.is_empty() { "nounsql" } else { lang };
    format!(
        "<figure class=\"code\" data-lang=\"{}\"><pre><code>{inner}</code></pre></figure>",
        escape(label)
    )
}

fn layout(page: &Page, body: &str, headings: &[Heading]) -> String {
    let nav = PAGES
        .iter()
        .map(|p| {
            let current = if p.output == page.output {
                " class=\"current\""
            } else {
                ""
            };
            format!("<a href=\"{}\"{current}>{}</a>", p.output, p.nav)
        })
        .collect::<Vec<_>>()
        .join("\n        ");

    let toc = headings
        .iter()
        .filter(|h| h.level == 2 || h.level == 3)
        .map(|h| {
            format!(
                "<a class=\"h{}\" href=\"#{}\">{}</a>",
                h.level,
                h.id,
                escape(&h.text)
            )
        })
        .collect::<Vec<_>>()
        .join("\n        ");

    let toc_block = if toc.is_empty() {
        String::new()
    } else {
        format!(
            "<nav class=\"toc\"><p class=\"toc-title\">このページ</p>\n        {toc}\n      </nav>"
        )
    };

    format!(
        r##"<!doctype html>
<html lang="ja">
<head>
<meta charset="utf-8">
<meta name="viewport" content="width=device-width, initial-scale=1">
<title>{title}</title>
<meta name="description" content="NounSQL is a DSL for Database Schema Design. スキーマ定義から PostgreSQL の DDL を生成する DSL とそのコンパイラ。">
<link rel="stylesheet" href="style.css">
<link rel="icon" href="data:image/svg+xml,<svg xmlns='http://www.w3.org/2000/svg' viewBox='0 0 16 16'><text y='13' font-size='14'>&#128190;</text></svg>">
</head>
<body>
<a class="skip" href="#main">本文へ</a>
<header class="topbar">
  <a class="brand" href="index.html"><span class="brand-mark">NounSQL</span></a>
  <nav class="topnav">
        {nav}
  </nav>
  <a class="repo" href="https://github.com/misebox/nounsql">GitHub</a>
</header>
<div class="shell">
  <main id="main" class="content">
{body}
  </main>
  <aside class="side">
      {toc_block}
  </aside>
</div>
<footer class="foot">
  <p><strong>NounSQL</strong> is a DSL for Database Schema Design.</p>
</footer>
</body>
</html>
"##,
        title = if page.output == "index.html" {
            "NounSQL — a DSL for Database Schema Design".to_string()
        } else {
            format!("{} — NounSQL", escape(page.title))
        },
    )
}
