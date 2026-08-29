use std::collections::HashMap;

use crate::ast::NounsBlock;
use crate::span::Span;

#[derive(Debug, Clone)]
pub struct Entry {
    pub singular: String,
    pub plural: String,
    pub short: String,
    pub comment: Option<String>,
    pub span: Span,
}

/// `nouns` ブロックの名詞辞書。識別子で引く。
#[derive(Debug, Default)]
pub struct Dict {
    by_id: HashMap<String, Entry>,
}

/// 複合名詞の構成要素。
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Part {
    /// 名詞。文脈に応じて屈折する。
    Noun(String),
    /// 文字列。屈折しない。
    Literal(String),
}

/// 1つ以上の要素からなる名詞。単数形と複数形を持つ。
///
/// 最後の要素だけが文脈の数に従い、それ以外は単数形になる。
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Compound {
    pub parts: Vec<Part>,
}

impl Compound {
    pub fn noun(name: impl Into<String>) -> Self {
        Compound {
            parts: vec![Part::Noun(name.into())],
        }
    }

    pub fn literal(text: impl Into<String>) -> Self {
        Compound {
            parts: vec![Part::Literal(text.into())],
        }
    }

    /// 単一の名詞ならその名前。
    pub fn as_single_noun(&self) -> Option<&str> {
        match self.parts.as_slice() {
            [Part::Noun(n)] => Some(n),
            _ => None,
        }
    }

    /// 含まれる名詞。辞書登録の検査に使う。
    pub fn nouns(&self) -> impl Iterator<Item = &str> {
        self.parts.iter().filter_map(|p| match p {
            Part::Noun(n) => Some(n.as_str()),
            Part::Literal(_) => None,
        })
    }

    pub fn singular(&self, dict: &Dict, separator: &str) -> String {
        self.render(dict, separator, false)
    }

    pub fn plural(&self, dict: &Dict, separator: &str) -> String {
        self.render(dict, separator, true)
    }

    /// 略語。数は変えず、名詞の要素をすべて略語にする。
    pub fn short(&self, dict: &Dict, separator: &str) -> String {
        self.parts
            .iter()
            .map(|part| match part {
                Part::Literal(text) => text.clone(),
                Part::Noun(id) => dict.short(id).into_value(),
            })
            .collect::<Vec<_>>()
            .join(separator)
    }

    /// 屈折させず、書かれたまま連結する。
    pub fn as_written(&self, separator: &str) -> String {
        self.parts
            .iter()
            .map(|p| match p {
                Part::Noun(n) | Part::Literal(n) => n.as_str(),
            })
            .collect::<Vec<_>>()
            .join(separator)
    }

    fn render(&self, dict: &Dict, separator: &str, last_plural: bool) -> String {
        let last = self.parts.len().saturating_sub(1);
        self.parts
            .iter()
            .enumerate()
            .map(|(i, part)| match part {
                Part::Literal(text) => text.clone(),
                Part::Noun(name) if i == last && last_plural => dict.plural(name).into_value(),
                Part::Noun(name) => dict.singular(name).into_value(),
            })
            .collect::<Vec<_>>()
            .join(separator)
    }
}

/// 辞書で解決できたか、規則変化に落ちたか。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Resolved {
    FromDict(String),
    FromRule(String),
}

impl Resolved {
    pub fn value(&self) -> &str {
        match self {
            Resolved::FromDict(s) | Resolved::FromRule(s) => s,
        }
    }

    pub fn into_value(self) -> String {
        match self {
            Resolved::FromDict(s) | Resolved::FromRule(s) => s,
        }
    }
}

impl Dict {
    /// 語形は書かれていなければ順に埋める。
    /// 識別子 → 単数形 → 複数形 / 略語、の一方通行。
    pub fn from_block(block: Option<&NounsBlock>) -> Dict {
        let mut dict = Dict::default();
        let Some(block) = block else { return dict };
        for e in &block.entries {
            let word = |w: &Option<crate::ast::Name>| w.as_ref().map(|w| w.value.clone());
            let singular = word(&e.singular).unwrap_or_else(|| e.id.value.clone());
            let plural = word(&e.plural).unwrap_or_else(|| pluralize(&singular));
            let short = word(&e.short).unwrap_or_else(|| singular.clone());
            dict.by_id.insert(
                e.id.value.clone(),
                Entry {
                    singular,
                    plural,
                    short,
                    comment: e.comment.as_ref().map(|c| c.value.clone()),
                    span: e.id.span,
                },
            );
        }
        dict
    }

    pub fn get(&self, id: &str) -> Option<&Entry> {
        self.by_id.get(id)
    }

    pub fn entries(&self) -> impl Iterator<Item = &Entry> {
        self.by_id.values()
    }

    pub fn plural(&self, id: &str) -> Resolved {
        match self.by_id.get(id) {
            Some(e) => Resolved::FromDict(e.plural.clone()),
            None => Resolved::FromRule(pluralize(id)),
        }
    }

    pub fn singular(&self, id: &str) -> Resolved {
        match self.by_id.get(id) {
            Some(e) => Resolved::FromDict(e.singular.clone()),
            None => Resolved::FromRule(id.to_string()),
        }
    }

    pub fn short(&self, id: &str) -> Resolved {
        match self.by_id.get(id) {
            Some(e) => Resolved::FromDict(e.short.clone()),
            None => Resolved::FromRule(id.to_string()),
        }
    }
}

fn is_vowel(c: char) -> bool {
    matches!(c, 'a' | 'e' | 'i' | 'o' | 'u')
}

/// 規則変化。辞書に無い語のフォールバック。
pub fn pluralize(word: &str) -> String {
    let lower = word.to_ascii_lowercase();
    if lower.ends_with("s")
        || lower.ends_with("x")
        || lower.ends_with("z")
        || lower.ends_with("ch")
        || lower.ends_with("sh")
    {
        return format!("{word}es");
    }
    if lower.ends_with('y') {
        let before = lower.chars().rev().nth(1);
        if before.is_some_and(|c| !is_vowel(c)) {
            return format!("{}ies", &word[..word.len() - 1]);
        }
    }
    format!("{word}s")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::span::Span;

    #[test]
    fn regular_plurals() {
        assert_eq!(pluralize("post"), "posts");
        assert_eq!(pluralize("category"), "categories");
        assert_eq!(pluralize("history"), "histories");
        assert_eq!(pluralize("box"), "boxes");
        assert_eq!(pluralize("day"), "days");
    }

    /// `(識別子, 単数形, 複数形, 略語)`
    fn dict_with(entries: &[(&str, &str, &str, &str)]) -> Dict {
        let mut dict = Dict::default();
        for (id, singular, plural, short) in entries {
            dict.by_id.insert(
                id.to_string(),
                Entry {
                    singular: singular.to_string(),
                    plural: plural.to_string(),
                    short: short.to_string(),
                    comment: None,
                    span: Span::new(0, 0),
                },
            );
        }
        dict
    }

    #[test]
    fn compound_inflects_only_the_last_noun() {
        let dict = dict_with(&[
            ("message", "message", "messages", "msg"),
            ("history", "history", "histories", "hist"),
        ]);
        let c = Compound {
            parts: vec![Part::Literal("sent".into()), Part::Noun("message".into())],
        };
        assert_eq!(c.singular(&dict, "_"), "sent_message");
        assert_eq!(c.plural(&dict, "_"), "sent_messages");

        let c = Compound {
            parts: vec![Part::Noun("post".into()), Part::Noun("history".into())],
        };
        assert_eq!(c.singular(&dict, "_"), "post_history");
        assert_eq!(c.plural(&dict, "_"), "post_histories");
    }

    #[test]
    fn literal_never_inflects() {
        let dict = dict_with(&[]);
        let c = Compound::literal("users");
        assert_eq!(c.plural(&dict, "_"), "users");
        assert_eq!(c.short(&dict, "_"), "users");
    }

    #[test]
    fn short_replaces_every_noun_and_keeps_the_number() {
        let dict = dict_with(&[
            ("message", "message", "messages", "msg"),
            ("history", "history", "histories", "hist"),
        ]);
        let c = Compound {
            parts: vec![Part::Noun("message".into()), Part::Noun("history".into())],
        };
        assert_eq!(c.short(&dict, "_"), "msg_hist");

        let c = Compound {
            parts: vec![Part::Literal("sent".into()), Part::Noun("message".into())],
        };
        assert_eq!(c.short(&dict, "_"), "sent_msg");
    }

    #[test]
    fn unregistered_nouns_keep_the_identifier() {
        let dict = dict_with(&[]);
        assert_eq!(dict.singular("widget").value(), "widget");
        assert_eq!(dict.short("widget").value(), "widget");
        assert_eq!(dict.plural("widget").value(), "widgets");
    }
}
