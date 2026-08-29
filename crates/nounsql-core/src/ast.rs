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
        args: Vec<Spanned<Value>>,
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RelationKind {
    BelongsTo,
    UniqueBelongsTo,
    HasMany,
    HasOne,
}

impl RelationKind {
    pub fn keyword(self) -> &'static str {
        match self {
            RelationKind::BelongsTo => "belongs_to",
            RelationKind::UniqueBelongsTo => "unique_belongs_to",
            RelationKind::HasMany => "has_many",
            RelationKind::HasOne => "has_one",
        }
    }

    /// FK列を生成する側か。
    pub fn owns_fk(self) -> bool {
        matches!(
            self,
            RelationKind::BelongsTo | RelationKind::UniqueBelongsTo
        )
    }

    pub fn is_unique(self) -> bool {
        matches!(self, RelationKind::UniqueBelongsTo | RelationKind::HasOne)
    }
}

#[derive(Debug, Clone)]
pub struct Relation {
    pub kind: RelationKind,
    pub target: Name,
    /// FK列名。`belongs_to` 系のみ。
    pub fk: Option<Spanned<String>>,
    /// この側の関連名。文字列か `noun(...)`。
    pub alias: Option<Spanned<Value>>,
    /// 対応するFK列名。`has_many` 系のみ。
    pub via: Option<Spanned<String>>,
    /// FK列のコメント。`belongs_to` 系のみ。
    pub comment: Option<Spanned<String>>,
}

#[derive(Debug, Clone)]
pub enum Member {
    /// `comment "..."`。テーブルの説明。
    Comment(Spanned<String>),
    /// `name 名詞`。テーブルの名詞。省略するとブロックの識別子を使う。
    Name(Spanned<Value>),
    Column(Column),
    Pk(Vec<Name>),
    Index(Index),
    Use(Name),
    Override(Override),
    Except(Vec<Name>),
    ExceptIndex(Vec<Name>),
    Relation(Relation),
}

#[derive(Debug, Clone)]
pub struct Table {
    pub name: Name,
    pub members: Vec<Spanned<Member>>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct Mixin {
    pub name: Name,
    pub members: Vec<Spanned<Member>>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct Blueprint {
    pub name: Name,
    pub params: Vec<Name>,
    pub tables: Vec<Table>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct MacroCall {
    pub name: Name,
    pub args: Vec<Name>,
    /// 生成するテーブルの名詞。`associate` のみ。
    pub table_name: Option<Spanned<Value>>,
    /// 生成するテーブルのコメント。`associate` のみ。
    pub comment: Option<Spanned<String>>,
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
pub struct Noun {
    /// ソースが名詞を指す名前。`table` や `belongs_to` が書くのはこれ。
    /// 出力には出ない。省略された語形の種になるだけ。
    pub id: Name,
    /// 省略時は識別子。
    pub singular: Option<Name>,
    /// 省略時は単数形の規則変化。
    pub plural: Option<Name>,
    /// 名前を詰めたいときの形。省略時は単数形。
    pub short: Option<Name>,
    pub comment: Option<Spanned<String>>,
}

#[derive(Debug, Clone)]
pub struct NounsBlock {
    pub entries: Vec<Noun>,
    pub span: Span,
}

#[derive(Debug, Clone, Default)]
pub struct Document {
    pub naming: Option<ConfigBlock>,
    pub constraints: Option<ConfigBlock>,
    pub nouns: Option<NounsBlock>,
    pub mixins: Vec<Mixin>,
    pub blueprints: Vec<Blueprint>,
    pub tables: Vec<Table>,
    pub macros: Vec<MacroCall>,
}
