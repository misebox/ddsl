use std::collections::HashMap;

use crate::ast::NounsBlock;
use crate::span::Span;

#[derive(Debug, Clone)]
pub struct Entry {
    pub singular: String,
    pub plural: String,
    pub comment: Option<String>,
    pub span: Span,
}

/// `nouns` ブロックの名詞辞書。
#[derive(Debug, Default)]
pub struct Dict {
    by_singular: HashMap<String, Entry>,
    by_plural: HashMap<String, String>,
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
    pub fn from_block(block: Option<&NounsBlock>) -> Dict {
        let mut dict = Dict::default();
        let Some(block) = block else { return dict };
        for e in &block.entries {
            let entry = Entry {
                singular: e.singular.value.clone(),
                plural: e.plural.value.clone(),
                comment: e.comment.as_ref().map(|c| c.value.clone()),
                span: e.singular.span,
            };
            dict.by_plural
                .insert(entry.plural.clone(), entry.singular.clone());
            dict.by_singular.insert(entry.singular.clone(), entry);
        }
        dict
    }

    pub fn get(&self, singular: &str) -> Option<&Entry> {
        self.by_singular.get(singular)
    }

    pub fn entries(&self) -> impl Iterator<Item = &Entry> {
        self.by_singular.values()
    }

    pub fn plural(&self, name: &str) -> Resolved {
        match self.by_singular.get(name) {
            Some(e) => Resolved::FromDict(e.plural.clone()),
            None => Resolved::FromRule(pluralize(name)),
        }
    }

    pub fn singular(&self, name: &str) -> Resolved {
        if self.by_singular.contains_key(name) {
            return Resolved::FromDict(name.to_string());
        }
        match self.by_plural.get(name) {
            Some(s) => Resolved::FromDict(s.clone()),
            None => Resolved::FromRule(singularize(name)),
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

pub fn singularize(word: &str) -> String {
    let lower = word.to_ascii_lowercase();
    if lower.ends_with("ies") && word.len() > 3 {
        return format!("{}y", &word[..word.len() - 3]);
    }
    for suffix in ["ses", "xes", "zes", "ches", "shes"] {
        if lower.ends_with(suffix) {
            return word[..word.len() - 2].to_string();
        }
    }
    word.strip_suffix('s').unwrap_or(word).to_string()
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

    fn dict_with(entries: &[(&str, &str)]) -> Dict {
        let mut dict = Dict::default();
        for (singular, plural) in entries {
            dict.by_plural
                .insert(plural.to_string(), singular.to_string());
            dict.by_singular.insert(
                singular.to_string(),
                Entry {
                    singular: singular.to_string(),
                    plural: plural.to_string(),
                    comment: None,
                    span: Span::new(0, 0),
                },
            );
        }
        dict
    }

    #[test]
    fn compound_inflects_only_the_last_noun() {
        let dict = dict_with(&[("message", "messages"), ("history", "histories")]);
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
        assert_eq!(c.as_written("_"), "users");
    }

    #[test]
    fn regular_singulars() {
        assert_eq!(singularize("posts"), "post");
        assert_eq!(singularize("categories"), "category");
        assert_eq!(singularize("boxes"), "box");
    }
}
