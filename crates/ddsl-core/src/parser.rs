use crate::ast::*;
use crate::diag::Diagnostic;
use crate::lexer::{Tok, Token, lex};
use crate::span::{Span, Spanned};

pub fn parse(src: &str) -> (Document, Vec<Diagnostic>) {
    let (tokens, mut diags) = lex(src);
    let mut parser = Parser {
        tokens,
        pos: 0,
        diags: Vec::new(),
    };
    let doc = parser.document();
    diags.append(&mut parser.diags);
    (doc, diags)
}

/// 文の解析に失敗したことを表す。行末まで読み飛ばして回復する。
struct Bail;
type PResult<T> = Result<T, Bail>;

const ATTR_KEYS: &[&str] = &["type", "null", "default", "on_update", "comment"];
const VALUE_FNS: &[&str] = &["name_join", "singular", "plural"];

struct Parser {
    tokens: Vec<Token>,
    pos: usize,
    diags: Vec<Diagnostic>,
}

impl Parser {
    // ---------- トークン操作 ----------

    fn peek(&self) -> &Tok {
        &self.tokens[self.pos].value
    }

    fn peek_at(&self, n: usize) -> &Tok {
        let i = (self.pos + n).min(self.tokens.len() - 1);
        &self.tokens[i].value
    }

    fn span(&self) -> Span {
        self.tokens[self.pos].span
    }

    fn prev_span(&self) -> Span {
        self.tokens[self.pos.saturating_sub(1)].span
    }

    fn bump(&mut self) -> Token {
        let tok = self.tokens[self.pos].clone();
        if self.pos + 1 < self.tokens.len() {
            self.pos += 1;
        }
        tok
    }

    fn at(&self, tok: &Tok) -> bool {
        self.peek() == tok
    }

    fn eat(&mut self, tok: &Tok) -> bool {
        if self.at(tok) {
            self.bump();
            true
        } else {
            false
        }
    }

    fn at_keyword(&self, kw: &str) -> bool {
        matches!(self.peek(), Tok::Ident(s) if s == kw)
    }

    fn error(&mut self, span: Span, msg: impl Into<String>) -> Bail {
        self.diags.push(Diagnostic::error(span, msg));
        Bail
    }

    fn expect(&mut self, tok: Tok) -> PResult<Span> {
        if self.at(&tok) {
            Ok(self.bump().span)
        } else {
            let span = self.span();
            let found = self.peek().describe();
            Err(self.error(span, format!("{} が必要。{} が来た", tok.describe(), found)))
        }
    }

    fn expect_ident(&mut self, what: &str) -> PResult<Name> {
        match self.peek().clone() {
            Tok::Ident(name) => {
                let span = self.bump().span;
                Ok(Spanned::new(name, span))
            }
            other => {
                let span = self.span();
                Err(self.error(span, format!("{what} が必要。{} が来た", other.describe())))
            }
        }
    }

    fn expect_string(&mut self, what: &str) -> PResult<Spanned<String>> {
        match self.peek().clone() {
            Tok::Str(s) => {
                let span = self.bump().span;
                Ok(Spanned::new(s, span))
            }
            other => {
                let span = self.span();
                Err(self.error(span, format!("{what} が必要。{} が来た", other.describe())))
            }
        }
    }

    fn skip_newlines(&mut self) {
        while self.at(&Tok::Newline) {
            self.bump();
        }
    }

    /// 行末まで読み飛ばす。1行1文なので、これで次の文から再開できる。
    fn recover_line(&mut self) {
        while !matches!(self.peek(), Tok::Newline | Tok::Eof | Tok::RBrace) {
            self.bump();
        }
    }

    /// 対応する `}` まで読み飛ばす。
    fn recover_block(&mut self) {
        let mut depth = 0usize;
        loop {
            match self.peek() {
                Tok::Eof => return,
                Tok::LBrace => depth += 1,
                Tok::RBrace => {
                    self.bump();
                    if depth == 0 {
                        return;
                    }
                    depth -= 1;
                    continue;
                }
                _ => {}
            }
            self.bump();
        }
    }

    /// 文の終わり（改行 / `}` / EOF）を要求する。
    fn end_of_statement(&mut self) -> PResult<()> {
        match self.peek() {
            Tok::Newline | Tok::Eof | Tok::RBrace => Ok(()),
            other => {
                let span = self.span();
                let found = other.describe();
                Err(self.error(span, format!("1行1文。行末が必要だが {found} が来た")))
            }
        }
    }

    // ---------- トップレベル ----------

    fn document(&mut self) -> Document {
        let mut doc = Document::default();
        loop {
            self.skip_newlines();
            if self.at(&Tok::Eof) {
                break;
            }
            if self.item(&mut doc).is_err() {
                self.recover_line();
            }
        }
        doc
    }

    fn item(&mut self, doc: &mut Document) -> PResult<()> {
        let Tok::Ident(kw) = self.peek().clone() else {
            let span = self.span();
            let found = self.peek().describe();
            return Err(self.error(span, format!("トップレベルに書けない: {found}")));
        };
        match kw.as_str() {
            "naming" | "constraints" => {
                let block = self.config_block()?;
                let slot = if kw == "naming" {
                    &mut doc.naming
                } else {
                    &mut doc.constraints
                };
                if let Some(prev) = slot {
                    let span = block.name.span;
                    let prev_span = prev.name.span;
                    self.diags.push(
                        Diagnostic::error(span, format!("`{kw}` ブロックが重複している"))
                            .with_label(prev_span, "最初の定義"),
                    );
                } else {
                    *slot = Some(block);
                }
                Ok(())
            }
            "entities" => {
                let block = self.entities_block()?;
                if let Some(prev) = &doc.entities {
                    let prev_span = prev.span;
                    self.diags.push(
                        Diagnostic::error(block.span, "`entities` ブロックが重複している")
                            .with_label(prev_span, "最初の定義"),
                    );
                } else {
                    doc.entities = Some(block);
                }
                Ok(())
            }
            "mixin" => {
                let m = self.mixin_block()?;
                doc.mixins.push(m);
                Ok(())
            }
            "table" => {
                let t = self.table_block()?;
                doc.tables.push(t);
                Ok(())
            }
            "blueprint" => {
                let b = self.blueprint_block()?;
                doc.blueprints.push(b);
                Ok(())
            }
            "associate" | "apply_blueprint" => {
                let m = self.macro_call()?;
                doc.macros.push(m);
                self.end_of_statement()
            }
            other => {
                let span = self.span();
                Err(self.error(span, format!("トップレベルに書けないキーワード `{other}`")))
            }
        }
    }

    // ---------- 設定ブロック ----------

    fn config_block(&mut self) -> PResult<ConfigBlock> {
        let name = self.expect_ident("ブロック名")?;
        let start = name.span;
        if self.expect(Tok::LBrace).is_err() {
            self.recover_block();
            return Err(Bail);
        }
        let mut entries = Vec::new();
        loop {
            self.skip_newlines();
            if self.at(&Tok::RBrace) || self.at(&Tok::Eof) {
                break;
            }
            match self.assign() {
                Ok(a) => {
                    entries.push(a);
                    if self.end_of_statement().is_err() {
                        self.recover_line();
                    }
                }
                Err(Bail) => self.recover_line(),
            }
        }
        let end = self.expect(Tok::RBrace).unwrap_or(self.prev_span());
        Ok(ConfigBlock {
            name,
            entries,
            span: start.join(end),
        })
    }

    fn assign(&mut self) -> PResult<Assign> {
        let key = self.expect_ident("設定キー")?;
        self.expect(Tok::Eq)?;
        let value = self.value()?;
        Ok(Assign { key, value })
    }

    fn entities_block(&mut self) -> PResult<EntitiesBlock> {
        let start = self.expect_ident("`entities`")?.span;
        if self.expect(Tok::LBrace).is_err() {
            self.recover_block();
            return Err(Bail);
        }
        let mut entries = Vec::new();
        loop {
            self.skip_newlines();
            if self.at(&Tok::RBrace) || self.at(&Tok::Eof) {
                break;
            }
            match self.entity() {
                Ok(e) => {
                    entries.push(e);
                    if self.end_of_statement().is_err() {
                        self.recover_line();
                    }
                }
                Err(Bail) => self.recover_line(),
            }
        }
        let end = self.expect(Tok::RBrace).unwrap_or(self.prev_span());
        Ok(EntitiesBlock {
            entries,
            span: start.join(end),
        })
    }

    fn entity(&mut self) -> PResult<Entity> {
        let singular = self.expect_ident("単数形")?;
        let plural = self.expect_ident("複数形")?;
        let comment = match self.peek() {
            Tok::Str(_) => Some(self.expect_string("コメント")?),
            _ => None,
        };
        Ok(Entity {
            singular,
            plural,
            comment,
        })
    }

    // ---------- mixin / table ----------

    fn mixin_block(&mut self) -> PResult<Mixin> {
        let start = self.expect_ident("`mixin`")?.span;
        let name = self.expect_ident("mixin名")?;
        let comment = self.opt_comment_attr()?;
        let (members, end) = self.member_block()?;
        Ok(Mixin {
            name,
            comment,
            members,
            span: start.join(end),
        })
    }

    fn table_block(&mut self) -> PResult<Table> {
        let start = self.expect_ident("`table`")?.span;
        let name = self.expect_ident("テーブル名")?;
        let comment = self.opt_comment_attr()?;
        let (members, end) = self.member_block()?;
        Ok(Table {
            name,
            comment,
            members,
            span: start.join(end),
        })
    }

    fn opt_comment_attr(&mut self) -> PResult<Option<Spanned<String>>> {
        if self.at_keyword("comment") && *self.peek_at(1) == Tok::Eq {
            self.bump();
            self.bump();
            Ok(Some(self.expect_string("コメント文字列")?))
        } else {
            Ok(None)
        }
    }

    fn member_block(&mut self) -> PResult<(Vec<Spanned<Member>>, Span)> {
        if self.expect(Tok::LBrace).is_err() {
            self.recover_block();
            return Err(Bail);
        }
        let mut members = Vec::new();
        loop {
            self.skip_newlines();
            if self.at(&Tok::RBrace) || self.at(&Tok::Eof) {
                break;
            }
            let start = self.span();
            match self.member() {
                Ok(m) => {
                    let span = start.join(self.prev_span());
                    members.push(Spanned::new(m, span));
                    if self.end_of_statement().is_err() {
                        self.recover_line();
                    }
                }
                Err(Bail) => self.recover_line(),
            }
        }
        let end = self.expect(Tok::RBrace).unwrap_or(self.prev_span());
        Ok((members, end))
    }

    fn member(&mut self) -> PResult<Member> {
        let Tok::Ident(kw) = self.peek().clone() else {
            let span = self.span();
            let found = self.peek().describe();
            return Err(self.error(span, format!("宣言文が必要。{found} が来た")));
        };
        match kw.as_str() {
            "column" => {
                self.bump();
                let name = self.expect_ident("カラム名")?;
                let attrs = self.attrs()?;
                Ok(Member::Column(Column { name, attrs }))
            }
            "pk" => {
                self.bump();
                Ok(Member::Pk(self.name_list("列名")?))
            }
            "index" => {
                self.bump();
                let columns = self.name_list("列名")?;
                let unique = if self.at_keyword("unique") {
                    self.bump();
                    true
                } else {
                    false
                };
                Ok(Member::Index(Index { columns, unique }))
            }
            "use" => {
                self.bump();
                Ok(Member::Use(self.expect_ident("mixin名")?))
            }
            "override" => {
                self.bump();
                let name = self.expect_ident("カラム名")?;
                let attrs = self.attrs()?;
                if attrs.is_empty() {
                    let span = name.span;
                    return Err(self.error(span, "`override` には属性が1つ以上必要"));
                }
                Ok(Member::Override(Override { name, attrs }))
            }
            "except" => {
                self.bump();
                if self.at_keyword("index") {
                    self.bump();
                    Ok(Member::ExceptIndex(self.name_list("列名")?))
                } else {
                    Ok(Member::Except(self.name_list("カラム名")?))
                }
            }
            "belongs_to" => {
                self.bump();
                Ok(Member::BelongsTo(self.expect_ident("参照先")?))
            }
            "unique_belongs_to" => {
                self.bump();
                Ok(Member::UniqueBelongsTo(self.expect_ident("参照先")?))
            }
            other => {
                let span = self.span();
                Err(self.error(span, format!("宣言文にならないキーワード `{other}`")))
            }
        }
    }

    fn attrs(&mut self) -> PResult<Vec<Attr>> {
        let mut attrs = Vec::new();
        while let Tok::Ident(key) = self.peek().clone() {
            if *self.peek_at(1) != Tok::Eq {
                break;
            }
            let key_span = self.span();
            self.bump();
            self.bump();
            if !ATTR_KEYS.contains(&key.as_str()) {
                self.diags.push(Diagnostic::error(
                    key_span,
                    format!(
                        "知らない属性キー `{key}`。使えるのは {}",
                        ATTR_KEYS.join(" / ")
                    ),
                ));
            }
            let value = self.value()?;
            attrs.push(Attr {
                key: Spanned::new(key, key_span),
                value,
            });
        }
        Ok(attrs)
    }

    /// `名前` または `[名前, ...]`
    fn name_list(&mut self, what: &str) -> PResult<Vec<Name>> {
        if self.eat(&Tok::LBracket) {
            let mut names = Vec::new();
            loop {
                if self.at(&Tok::RBracket) {
                    break;
                }
                names.push(self.expect_ident(what)?);
                if !self.eat(&Tok::Comma) {
                    break;
                }
            }
            self.expect(Tok::RBracket)?;
            if names.is_empty() {
                let span = self.prev_span();
                return Err(self.error(span, format!("{what} が空")));
            }
            Ok(names)
        } else {
            Ok(vec![self.expect_ident(what)?])
        }
    }

    // ---------- blueprint ----------

    fn blueprint_block(&mut self) -> PResult<Blueprint> {
        let start = self.expect_ident("`blueprint`")?.span;
        let name = self.expect_ident("blueprint名")?;
        let mut params = Vec::new();
        while let Tok::Ident(p) = self.peek().clone() {
            if p == "comment" && *self.peek_at(1) == Tok::Eq {
                break;
            }
            params.push(self.expect_ident("引数名")?);
        }
        let comment = self.opt_comment_attr()?;

        if self.expect(Tok::LBrace).is_err() {
            self.recover_block();
            return Err(Bail);
        }
        let mut items = Vec::new();
        loop {
            self.skip_newlines();
            if self.at(&Tok::RBrace) || self.at(&Tok::Eof) {
                break;
            }
            if self.at_keyword("let") {
                match self.let_stmt() {
                    Ok(l) => {
                        items.push(BlueprintItem::Let(l));
                        if self.end_of_statement().is_err() {
                            self.recover_line();
                        }
                    }
                    Err(Bail) => self.recover_line(),
                }
            } else if self.at_keyword("table") {
                match self.table_block() {
                    Ok(t) => items.push(BlueprintItem::Table(t)),
                    Err(Bail) => self.recover_line(),
                }
            } else {
                let span = self.span();
                let found = self.peek().describe();
                self.diags.push(Diagnostic::error(
                    span,
                    format!("blueprint 内に書けるのは `let` と `table` のみ。{found} が来た"),
                ));
                self.recover_line();
            }
        }
        let end = self.expect(Tok::RBrace).unwrap_or(self.prev_span());
        Ok(Blueprint {
            name,
            params,
            comment,
            items,
            span: start.join(end),
        })
    }

    fn let_stmt(&mut self) -> PResult<Let> {
        self.expect_ident("`let`")?;
        let name = self.expect_ident("束縛名")?;
        self.expect(Tok::Eq)?;
        let value = self.value()?;
        Ok(Let { name, value })
    }

    // ---------- マクロ ----------

    fn macro_call(&mut self) -> PResult<MacroCall> {
        let name = self.expect_ident("マクロ名")?;
        let start = name.span;
        self.expect(Tok::LParen)?;
        let mut args = Vec::new();
        loop {
            if self.at(&Tok::RParen) {
                break;
            }
            args.push(self.expect_ident("引数")?);
            if !self.eat(&Tok::Comma) {
                break;
            }
        }
        let end = self.expect(Tok::RParen)?;
        Ok(MacroCall {
            name,
            args,
            span: start.join(end),
        })
    }

    // ---------- 値 ----------

    fn value(&mut self) -> PResult<Spanned<Value>> {
        let start = self.span();
        match self.peek().clone() {
            Tok::Str(s) => {
                let span = self.bump().span;
                Ok(Spanned::new(Value::Str(s), span))
            }
            Tok::Num(n) => {
                let span = self.bump().span;
                Ok(Spanned::new(Value::Num(n), span))
            }
            Tok::Eval(body) => {
                let span = self.bump().span;
                Ok(Spanned::new(Value::Eval(body), span))
            }
            Tok::LBracket => {
                let items = self.value_list()?;
                let span = start.join(self.prev_span());
                Ok(Spanned::new(Value::List(items), span))
            }
            Tok::Ident(name) => {
                if *self.peek_at(1) == Tok::LParen {
                    let call_name = self.expect_ident("関数名")?;
                    if !VALUE_FNS.contains(&name.as_str()) {
                        self.diags.push(Diagnostic::error(
                            call_name.span,
                            format!(
                                "値の位置で使える関数は {} のみ。`{name}` は使えない",
                                VALUE_FNS.join(" / ")
                            ),
                        ));
                    }
                    self.expect(Tok::LParen)?;
                    let mut args = Vec::new();
                    loop {
                        if self.at(&Tok::RParen) {
                            break;
                        }
                        args.push(self.expect_ident("引数")?);
                        if !self.eat(&Tok::Comma) {
                            break;
                        }
                    }
                    let end = self.expect(Tok::RParen)?;
                    Ok(Spanned::new(
                        Value::Call {
                            name: call_name,
                            args,
                        },
                        start.join(end),
                    ))
                } else {
                    let span = self.bump().span;
                    Ok(Spanned::new(Value::Ident(name), span))
                }
            }
            other => {
                let span = self.span();
                let found = other.describe();
                Err(self.error(span, format!("値が必要。{found} が来た")))
            }
        }
    }

    /// リストは要素の途中で改行できる。
    fn value_list(&mut self) -> PResult<Vec<Name>> {
        self.expect(Tok::LBracket)?;
        let mut items = Vec::new();
        loop {
            self.skip_newlines();
            if self.at(&Tok::RBracket) || self.at(&Tok::Eof) {
                break;
            }
            items.push(self.expect_ident("リスト要素")?);
            self.skip_newlines();
            if !self.eat(&Tok::Comma) {
                break;
            }
        }
        self.skip_newlines();
        self.expect(Tok::RBracket)?;
        Ok(items)
    }
}
