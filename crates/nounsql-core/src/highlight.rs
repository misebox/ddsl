//! NounSQL のシンタックスハイライト。ドキュメントサイトと WASM 版で共有する。

use crate::lexer::{Tok, lex_all};

const BLOCK_KEYWORDS: &[&str] = &[
    "table",
    "mixin",
    "blueprint",
    "naming",
    "constraints",
    "nouns",
];

const STATEMENT_KEYWORDS: &[&str] = &[
    "column",
    "pk",
    "index",
    "use",
    "override",
    "except",
    "belongs_to",
    "unique_belongs_to",
    "has_many",
    "has_one",
    "let",
    "unique",
    "comment",
];

/// 宣言文のうち、直後の識別子が「型・テーブル」を指すもの。
const NAMES_A_TYPE: &[&str] = &[
    "table",
    "mixin",
    "blueprint",
    "use",
    "belongs_to",
    "unique_belongs_to",
    "has_many",
    "has_one",
];

const MACROS: &[&str] = &["associate", "apply_blueprint"];
const FUNCTIONS: &[&str] = &["eval", "noun", "singular", "plural", "desc"];
const CONSTANTS: &[&str] = &[
    "true",
    "false",
    "cascade",
    "restrict",
    "set_null",
    "no_action",
    "plural",
    "singular",
];

/// NounSQL のソースを、実際の lexer を使って HTML に色付けする。
///
/// CSS クラスだけを付けるので、配色は利用側の stylesheet が決める。
pub fn to_html(src: &str) -> String {
    let (tokens, _) = lex_all(src);
    let mut out = String::new();
    let mut cursor = 0usize;
    let mut prev_ident: Option<String> = None;
    // 行頭のキーワード。1行1文なので、これで文の種類が判る。
    let mut line_head: Option<String> = None;

    for (i, tok) in tokens.iter().enumerate() {
        if tok.value == Tok::Eof {
            break;
        }
        // トークン間の空白をそのまま出す。
        if tok.span.start > cursor {
            out.push_str(&escape(&src[cursor..tok.span.start]));
        }
        let text = &src[tok.span.start..tok.span.end];
        let next_is_eq = matches!(tokens.get(i + 1).map(|t| &t.value), Some(Tok::Eq));
        let next_is_paren = matches!(tokens.get(i + 1).map(|t| &t.value), Some(Tok::LParen));

        let class = match &tok.value {
            Tok::Comment(_) => Some("c"),
            Tok::Str(_) => Some("s"),
            Tok::Num(_) => Some("n"),
            Tok::Eval(_) => Some("fn"),
            Tok::Ident(name) => classify(name, &prev_ident, &line_head, next_is_eq, next_is_paren),
            _ => None,
        };

        match &tok.value {
            Tok::Eval(body) => {
                out.push_str("<span class=\"fn\">eval</span>(");
                out.push_str(&format!("<span class=\"raw\">{}</span>", escape(body)));
                out.push(')');
            }
            Tok::Str(_) => out.push_str(&string_html(text)),
            _ => match class {
                Some(c) => out.push_str(&format!("<span class=\"{c}\">{}</span>", escape(text))),
                None => out.push_str(&escape(text)),
            },
        }

        match &tok.value {
            Tok::Ident(name) => {
                if line_head.is_none() {
                    line_head = Some(name.clone());
                }
                prev_ident = Some(name.clone());
            }
            Tok::Newline | Tok::LBrace | Tok::RBrace => {
                prev_ident = None;
                line_head = None;
            }
            // `=` を挟んでも直前の識別子を覚えておく（`type=text` の型名を拾うため）。
            Tok::Eq | Tok::Comment(_) => {}
            _ => prev_ident = None,
        }
        cursor = tok.span.end;
    }
    if cursor < src.len() {
        out.push_str(&escape(&src[cursor..]));
    }
    out
}

fn classify(
    name: &str,
    prev: &Option<String>,
    line_head: &Option<String>,
    next_is_eq: bool,
    next_is_paren: bool,
) -> Option<&'static str> {
    if next_is_paren {
        if MACROS.contains(&name) {
            return Some("macro");
        }
        if FUNCTIONS.contains(&name) {
            return Some("fn");
        }
    }
    if next_is_eq {
        return Some("attr");
    }
    // 直前の語で決まるもの。
    if let Some(p) = prev.as_deref() {
        if NAMES_A_TYPE.contains(&p) {
            return Some("ty");
        }
        match p {
            "column" | "override" => return Some("member"),
            "let" => return Some("var"),
            "type" => return Some("t"),
            _ => {}
        }
    }
    if BLOCK_KEYWORDS.contains(&name) || STATEMENT_KEYWORDS.contains(&name) {
        return Some("k");
    }
    if CONSTANTS.contains(&name) {
        return Some("cn");
    }
    // 文の種類で決まるもの。列の並びや引数の並び。
    match line_head.as_deref() {
        Some("pk" | "index" | "except") => Some("member"),
        Some(head) if MACROS.contains(&head) => Some("ty"),
        Some("blueprint") => Some("var"),
        _ => None,
    }
}

/// 文字列リテラル内の `${...}` を別色にする。
fn string_html(text: &str) -> String {
    let mut out = String::from("<span class=\"s\">");
    let mut rest = text;
    while let Some(i) = rest.find("${") {
        let Some(close) = rest[i..].find('}') else {
            break;
        };
        out.push_str(&escape(&rest[..i]));
        out.push_str(&format!(
            "<span class=\"tpl\">{}</span>",
            escape(&rest[i..i + close + 1])
        ));
        rest = &rest[i + close + 1..];
    }
    out.push_str(&escape(rest));
    out.push_str("</span>");
    out
}

pub fn escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}

const SQL_KEYWORDS: &[&str] = &[
    "CREATE",
    "TABLE",
    "ALTER",
    "ADD",
    "CONSTRAINT",
    "PRIMARY",
    "KEY",
    "FOREIGN",
    "REFERENCES",
    "ON",
    "DELETE",
    "UPDATE",
    "CASCADE",
    "RESTRICT",
    "SET",
    "NULL",
    "NOT",
    "DEFAULT",
    "UNIQUE",
    "INDEX",
    "COMMENT",
    "IS",
    "OR",
    "REPLACE",
    "FUNCTION",
    "RETURNS",
    "TRIGGER",
    "BEFORE",
    "FOR",
    "EACH",
    "ROW",
    "EXECUTE",
    "LANGUAGE",
    "BEGIN",
    "END",
    "RETURN",
    "AS",
];

/// 生成した DDL を色付けする。
pub fn sql_to_html(src: &str) -> String {
    let mut out = String::new();
    for token in split_keep(src) {
        if SQL_KEYWORDS.contains(&token.to_ascii_uppercase().as_str()) {
            out.push_str(&format!("<span class=\"k\">{}</span>", escape(token)));
        } else if token.starts_with('\'') {
            out.push_str(&format!("<span class=\"s\">{}</span>", escape(token)));
        } else {
            out.push_str(&escape(token));
        }
    }
    out
}

/// 単語と区切りを保ったまま分割する。
fn split_keep(src: &str) -> Vec<&str> {
    let mut parts = Vec::new();
    let mut start = 0;
    let mut in_word = false;
    for (i, ch) in src.char_indices() {
        let word = ch.is_alphanumeric() || ch == '_';
        if word != in_word {
            if i > start {
                parts.push(&src[start..i]);
            }
            start = i;
            in_word = word;
        }
    }
    if start < src.len() {
        parts.push(&src[start..]);
    }
    parts
}
