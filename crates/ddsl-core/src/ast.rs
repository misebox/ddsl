use crate::span::{Span, Spanned};

pub type Name = Spanned<String>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Value {
    Ident(String),
    Str(String),
    Num(String),
    /// `eval(...)` の中身。DB側で評価される式。
    Eval(String),
    List(Vec<Name>),
    Call {
        name: Name,
        args: Vec<Name>,
    },
}

#[derive(Debug, Clone)]
pub struct Attr {
    pub key: Name,
    pub value: Spanned<Value>,
}

#[derive(Debug, Clone)]
pub struct Column {
    pub name: Name,
    pub attrs: Vec<Attr>,
}

#[derive(Debug, Clone)]
pub struct Index {
    pub columns: Vec<Name>,
    pub unique: bool,
}

#[derive(Debug, Clone)]
pub struct Override {
    pub name: Name,
    pub attrs: Vec<Attr>,
}

#[derive(Debug, Clone)]
pub enum Member {
    Column(Column),
    Pk(Vec<Name>),
    Index(Index),
    Use(Name),
    Override(Override),
    Except(Vec<Name>),
    ExceptIndex(Vec<Name>),
    BelongsTo(Name),
    UniqueBelongsTo(Name),
}

#[derive(Debug, Clone)]
pub struct Table {
    pub name: Name,
    pub comment: Option<Spanned<String>>,
    pub members: Vec<Spanned<Member>>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct Mixin {
    pub name: Name,
    pub comment: Option<Spanned<String>>,
    pub members: Vec<Spanned<Member>>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct Let {
    pub name: Name,
    pub value: Spanned<Value>,
}

#[derive(Debug, Clone)]
pub enum BlueprintItem {
    Let(Let),
    Table(Table),
}

#[derive(Debug, Clone)]
pub struct Blueprint {
    pub name: Name,
    pub params: Vec<Name>,
    pub comment: Option<Spanned<String>>,
    pub items: Vec<BlueprintItem>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct MacroCall {
    pub name: Name,
    pub args: Vec<Name>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct Assign {
    pub key: Name,
    pub value: Spanned<Value>,
}

#[derive(Debug, Clone)]
pub struct ConfigBlock {
    pub name: Name,
    pub entries: Vec<Assign>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct Entity {
    pub singular: Name,
    pub plural: Name,
    pub comment: Option<Spanned<String>>,
}

#[derive(Debug, Clone)]
pub struct EntitiesBlock {
    pub entries: Vec<Entity>,
    pub span: Span,
}

#[derive(Debug, Clone, Default)]
pub struct Document {
    pub naming: Option<ConfigBlock>,
    pub constraints: Option<ConfigBlock>,
    pub entities: Option<EntitiesBlock>,
    pub mixins: Vec<Mixin>,
    pub blueprints: Vec<Blueprint>,
    pub tables: Vec<Table>,
    pub macros: Vec<MacroCall>,
}
