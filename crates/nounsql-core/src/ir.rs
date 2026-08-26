use indexmap::IndexMap;

use crate::dict::Compound;
use crate::span::Span;

#[cfg(feature = "serde")]
use serde::Serialize;

/// 定義がどこから来たか。診断で定義元を指すのに使う。
///
/// 直列化では種別と名前だけを出す。span はソースが無いと意味を持たない。
#[cfg_attr(
    feature = "serde",
    derive(Serialize),
    serde(tag = "kind", rename_all = "camelCase")
)]
#[derive(Debug, Clone)]
pub enum Origin {
    /// テーブル定義に直接書かれた。
    Own,
    Mixin {
        name: String,
        #[cfg_attr(feature = "serde", serde(skip))]
        def_span: Span,
    },
    Blueprint {
        name: String,
        #[cfg_attr(feature = "serde", serde(skip))]
        def_span: Span,
        #[cfg_attr(feature = "serde", serde(skip))]
        apply_span: Span,
    },
    /// `belongs_to` / `associate` が生成した。
    Generated { by: String },
}

#[cfg_attr(feature = "serde", derive(Serialize), serde(rename_all = "camelCase"))]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Val {
    Literal(String),
    Eval(String),
}

#[cfg_attr(feature = "serde", derive(Serialize), serde(rename_all = "camelCase"))]
#[derive(Debug, Clone)]
pub struct Column {
    pub name: String,
    #[cfg_attr(feature = "serde", serde(rename = "type"))]
    pub ty: String,
    pub null: bool,
    pub default: Option<Val>,
    pub on_update: Option<Val>,
    pub comment: Option<String>,
    pub origin: Origin,
    #[cfg_attr(feature = "serde", serde(skip))]
    pub span: Span,
}

#[cfg_attr(feature = "serde", derive(Serialize), serde(rename_all = "camelCase"))]
#[derive(Debug, Clone)]
pub struct Index {
    pub name: String,
    pub columns: Vec<String>,
    pub unique: bool,
    #[cfg_attr(feature = "serde", serde(skip))]
    pub span: Span,
}

#[cfg_attr(feature = "serde", derive(Serialize), serde(rename_all = "camelCase"))]
#[derive(Debug, Clone)]
pub struct ForeignKey {
    /// FKを持つ側の関連名。DDL には出ない。
    pub alias: String,
    pub columns: Vec<String>,
    pub ref_table: String,
    pub ref_columns: Vec<String>,
    pub on_delete: String,
    pub on_update: String,
    #[cfg_attr(feature = "serde", serde(skip))]
    pub span: Span,
}

/// 参照される側から見た関連。DDL には出ない。
#[cfg_attr(feature = "serde", derive(Serialize), serde(rename_all = "camelCase"))]
#[derive(Debug, Clone)]
pub struct Reverse {
    pub alias: String,
    /// FKを持つテーブル。
    pub from_table: String,
    /// 対応するFK列。
    pub via: Vec<String>,
    /// one_to_one か。
    pub unique: bool,
    #[cfg_attr(feature = "serde", serde(skip))]
    pub span: Span,
}

#[cfg_attr(feature = "serde", derive(Serialize), serde(rename_all = "camelCase"))]
#[derive(Debug, Clone)]
pub struct Table {
    /// 最終テーブル名。
    pub name: String,
    /// 由来した名詞。名前の組み立てに使う内部表現。
    #[cfg_attr(feature = "serde", serde(skip))]
    pub noun: Option<Compound>,
    /// 単数形。ORM のモデル名などに使う。
    pub singular: Option<String>,
    /// 複数形。
    pub plural: Option<String>,
    pub comment: Option<String>,
    pub columns: IndexMap<String, Column>,
    pub pk: Vec<String>,
    pub indexes: Vec<Index>,
    pub foreign_keys: Vec<ForeignKey>,
    pub reverses: Vec<Reverse>,
    pub origin: Origin,
    #[cfg_attr(feature = "serde", serde(skip))]
    pub span: Span,
}

#[cfg_attr(feature = "serde", derive(Serialize), serde(rename_all = "camelCase"))]
#[derive(Debug, Clone, Default)]
pub struct Schema {
    pub tables: Vec<Table>,
}

impl Schema {
    pub fn table(&self, name: &str) -> Option<&Table> {
        self.tables.iter().find(|t| t.name == name)
    }

    /// 単一の名詞から作られたテーブルを引く。
    pub fn table_by_noun(&self, name: &str) -> Option<&Table> {
        self.tables
            .iter()
            .find(|t| t.noun.as_ref().and_then(Compound::as_single_noun) == Some(name))
    }
}
