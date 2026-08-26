use indexmap::IndexMap;

use crate::span::Span;

/// 定義がどこから来たか。診断で定義元を指すのに使う。
#[derive(Debug, Clone)]
pub enum Origin {
    /// テーブル定義に直接書かれた。
    Own,
    Mixin {
        name: String,
        def_span: Span,
    },
    Blueprint {
        name: String,
        def_span: Span,
        apply_span: Span,
    },
    /// `belongs_to` / `associate` が生成した。
    Generated {
        by: String,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Val {
    Literal(String),
    Eval(String),
}

#[derive(Debug, Clone)]
pub struct Column {
    pub name: String,
    pub ty: String,
    pub null: bool,
    pub default: Option<Val>,
    pub on_update: Option<Val>,
    pub comment: Option<String>,
    pub origin: Origin,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct Index {
    pub name: String,
    pub columns: Vec<String>,
    pub unique: bool,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct ForeignKey {
    /// FKを持つ側の関連名。DDL には出ない。
    pub alias: String,
    pub columns: Vec<String>,
    pub ref_table: String,
    pub ref_columns: Vec<String>,
    pub on_delete: String,
    pub on_update: String,
    pub span: Span,
}

/// 参照される側から見た関連。DDL には出ない。
#[derive(Debug, Clone)]
pub struct Reverse {
    pub alias: String,
    /// FKを持つテーブル。
    pub from_table: String,
    /// 対応するFK列。
    pub via: Vec<String>,
    /// one_to_one か。
    pub unique: bool,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct Table {
    /// 最終テーブル名。
    pub name: String,
    /// 由来した語（単数形）。生成テーブルでは None。
    pub word: Option<String>,
    pub comment: Option<String>,
    pub columns: IndexMap<String, Column>,
    pub pk: Vec<String>,
    pub indexes: Vec<Index>,
    pub foreign_keys: Vec<ForeignKey>,
    pub reverses: Vec<Reverse>,
    pub origin: Origin,
    pub span: Span,
}

#[derive(Debug, Clone, Default)]
pub struct Schema {
    pub tables: Vec<Table>,
}

impl Schema {
    pub fn table(&self, name: &str) -> Option<&Table> {
        self.tables.iter().find(|t| t.name == name)
    }

    pub fn table_by_word(&self, word: &str) -> Option<&Table> {
        self.tables.iter().find(|t| t.word.as_deref() == Some(word))
    }
}
