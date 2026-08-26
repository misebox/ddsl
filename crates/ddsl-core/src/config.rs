use std::collections::HashMap;

use crate::ast::{ConfigBlock, Document, Value};
use crate::diag::Diagnostic;
use crate::span::Span;
use crate::template::Template;

/// `naming` の既定値。仕様の `### naming` と一致させる。
const DEFAULT_NAMING: &[(&str, &str)] = &[
    ("primary_key", "id"),
    ("foreign_key", "${singular(table)}_id"),
    ("join_table", "${singular(a)}_${singular(b)}"),
    ("name_join", "${singular(a)}_${plural(b)}"),
    ("index", "idx_${table}_${columns}"),
    ("unique_index", "uq_${table}_${columns}"),
    ("column_separator", "_"),
];

const NAMING_KEYS: &[&str] = &[
    "table_name",
    "primary_key",
    "foreign_key",
    "join_table",
    "name_join",
    "index",
    "unique_index",
    "column_separator",
];

const CONSTRAINT_KEYS: &[&str] = &[
    "null_default",
    "on_delete_default",
    "on_update_default",
    "foreign_key_index",
];

const REFERENTIAL_ACTIONS: &[&str] = &["cascade", "restrict", "set_null", "no_action"];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TableNameStyle {
    Plural,
    Singular,
}

#[derive(Debug, Clone)]
pub struct Naming {
    pub table_name: TableNameStyle,
    pub primary_key: String,
    pub foreign_key: Template,
    pub join_table: Template,
    pub name_join: Template,
    pub index: Template,
    pub unique_index: Template,
    pub column_separator: String,
}

#[derive(Debug, Clone)]
pub struct Constraints {
    pub null_default: bool,
    pub on_delete_default: String,
    pub on_update_default: String,
    pub foreign_key_index: bool,
}

#[derive(Debug, Clone)]
pub struct Config {
    pub naming: Naming,
    pub constraints: Constraints,
}

impl Default for Config {
    fn default() -> Self {
        let t = |key: &str| {
            let src = DEFAULT_NAMING
                .iter()
                .find(|(k, _)| *k == key)
                .map(|(_, v)| *v)
                .unwrap_or_default();
            Template::parse(src).unwrap_or(Template { segments: vec![] })
        };
        Config {
            naming: Naming {
                table_name: TableNameStyle::Plural,
                primary_key: "id".into(),
                foreign_key: t("foreign_key"),
                join_table: t("join_table"),
                name_join: t("name_join"),
                index: t("index"),
                unique_index: t("unique_index"),
                column_separator: "_".into(),
            },
            constraints: Constraints {
                null_default: false,
                on_delete_default: "cascade".into(),
                on_update_default: "cascade".into(),
                foreign_key_index: true,
            },
        }
    }
}

impl Config {
    pub fn from_document(doc: &Document, diags: &mut Vec<Diagnostic>) -> Config {
        let mut config = Config::default();
        if let Some(block) = &doc.naming {
            config.apply_naming(block, diags);
        }
        if let Some(block) = &doc.constraints {
            config.apply_constraints(block, diags);
        }
        config
    }

    fn apply_naming(&mut self, block: &ConfigBlock, diags: &mut Vec<Diagnostic>) {
        for entry in &block.entries {
            let key = entry.key.value.as_str();
            if !NAMING_KEYS.contains(&key) {
                diags.push(Diagnostic::error(
                    entry.key.span,
                    format!("知らない `naming` のキー `{key}`"),
                ));
                continue;
            }
            let span = entry.value.span;
            match key {
                "table_name" => match value_ident(&entry.value.value) {
                    Some("plural") => self.naming.table_name = TableNameStyle::Plural,
                    Some("singular") => self.naming.table_name = TableNameStyle::Singular,
                    _ => diags.push(Diagnostic::error(
                        span,
                        "`table_name` は `plural` か `singular`",
                    )),
                },
                "primary_key" | "column_separator" => match value_string(&entry.value.value) {
                    Some(s) => {
                        if key == "primary_key" {
                            self.naming.primary_key = s.into();
                        } else {
                            self.naming.column_separator = s.into();
                        }
                    }
                    None => {
                        diags.push(Diagnostic::error(span, format!("`{key}` には文字列を書く")))
                    }
                },
                _ => {
                    let Some(s) = value_string(&entry.value.value) else {
                        diags.push(Diagnostic::error(span, format!("`{key}` には文字列を書く")));
                        continue;
                    };
                    match Template::parse(s) {
                        Ok(t) => match key {
                            "foreign_key" => self.naming.foreign_key = t,
                            "join_table" => self.naming.join_table = t,
                            "name_join" => self.naming.name_join = t,
                            "index" => self.naming.index = t,
                            "unique_index" => self.naming.unique_index = t,
                            _ => {}
                        },
                        Err(msg) => diags.push(Diagnostic::error(span, msg)),
                    }
                }
            }
        }
    }

    fn apply_constraints(&mut self, block: &ConfigBlock, diags: &mut Vec<Diagnostic>) {
        for entry in &block.entries {
            let key = entry.key.value.as_str();
            if !CONSTRAINT_KEYS.contains(&key) {
                diags.push(Diagnostic::error(
                    entry.key.span,
                    format!("知らない `constraints` のキー `{key}`"),
                ));
                continue;
            }
            let span = entry.value.span;
            match key {
                "null_default" | "foreign_key_index" => match value_bool(&entry.value.value) {
                    Some(b) => {
                        if key == "null_default" {
                            self.constraints.null_default = b;
                        } else {
                            self.constraints.foreign_key_index = b;
                        }
                    }
                    None => diags.push(Diagnostic::error(
                        span,
                        format!("`{key}` は `true` か `false`"),
                    )),
                },
                _ => match value_ident(&entry.value.value) {
                    Some(a) if REFERENTIAL_ACTIONS.contains(&a) => {
                        if key == "on_delete_default" {
                            self.constraints.on_delete_default = a.into();
                        } else {
                            self.constraints.on_update_default = a.into();
                        }
                    }
                    _ => diags.push(Diagnostic::error(
                        span,
                        format!("`{key}` は {} のいずれか", REFERENTIAL_ACTIONS.join(" / ")),
                    )),
                },
            }
        }
    }
}

fn value_ident(v: &Value) -> Option<&str> {
    match v {
        Value::Ident(s) => Some(s),
        _ => None,
    }
}

fn value_string(v: &Value) -> Option<&str> {
    match v {
        Value::Str(s) => Some(s),
        _ => None,
    }
}

fn value_bool(v: &Value) -> Option<bool> {
    match value_ident(v)? {
        "true" => Some(true),
        "false" => Some(false),
        _ => None,
    }
}

/// テンプレート展開の変数束縛。
pub type Vars<'a> = HashMap<&'a str, String>;

pub fn missing_var(span: Span, name: &str) -> Diagnostic {
    Diagnostic::error(
        span,
        format!("テンプレート変数 `{name}` はここでは使えない"),
    )
}
