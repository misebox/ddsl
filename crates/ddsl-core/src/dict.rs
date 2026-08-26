use std::collections::HashMap;

use crate::ast::EntitiesBlock;
use crate::span::Span;

#[derive(Debug, Clone)]
pub struct Entry {
    pub singular: String,
    pub plural: String,
    pub comment: Option<String>,
    pub span: Span,
}

/// `entities` ブロックの命名辞書。
#[derive(Debug, Default)]
pub struct Dict {
    by_singular: HashMap<String, Entry>,
    by_plural: HashMap<String, String>,
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
    pub fn from_block(block: Option<&EntitiesBlock>) -> Dict {
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

    #[test]
    fn regular_plurals() {
        assert_eq!(pluralize("post"), "posts");
        assert_eq!(pluralize("category"), "categories");
        assert_eq!(pluralize("history"), "histories");
        assert_eq!(pluralize("box"), "boxes");
        assert_eq!(pluralize("day"), "days");
    }

    #[test]
    fn regular_singulars() {
        assert_eq!(singularize("posts"), "post");
        assert_eq!(singularize("categories"), "category");
        assert_eq!(singularize("boxes"), "box");
    }
}
