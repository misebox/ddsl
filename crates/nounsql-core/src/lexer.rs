use crate::diag::Diagnostic;
use crate::span::{Span, Spanned};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Tok {
    Ident(String),
    /// 文字列リテラル。引用符を除いた中身。
    Str(String),
    Num(String),
    /// `eval(...)` の中身を生のまま保持する。
    Eval(String),
    /// 行コメント。`parse` では捨てられる。ハイライトのために保持する。
    Comment(String),
    LBrace,
    RBrace,
    LBracket,
    RBracket,
    LParen,
    RParen,
    Comma,
    Eq,
    Newline,
    Eof,
}

impl Tok {
    pub fn describe(&self) -> String {
        match self {
            Tok::Ident(s) => format!("`{s}`"),
            Tok::Str(_) => "a string".into(),
            Tok::Num(s) => format!("`{s}`"),
            Tok::Eval(_) => "`eval(...)`".into(),
            Tok::Comment(_) => "a comment".into(),
            Tok::LBrace => "`{`".into(),
            Tok::RBrace => "`}`".into(),
            Tok::LBracket => "`[`".into(),
            Tok::RBracket => "`]`".into(),
            Tok::LParen => "`(`".into(),
            Tok::RParen => "`)`".into(),
            Tok::Comma => "`,`".into(),
            Tok::Eq => "`=`".into(),
            Tok::Newline => "a line break".into(),
            Tok::Eof => "the end of the input".into(),
        }
    }
}

pub type Token = Spanned<Tok>;

/// 構文解析用。コメントを落とす。
pub fn lex(src: &str) -> (Vec<Token>, Vec<Diagnostic>) {
    let (tokens, diags) = lex_all(src);
    (
        tokens
            .into_iter()
            .filter(|t| !matches!(t.value, Tok::Comment(_)))
            .collect(),
        diags,
    )
}

/// コメントを含む全トークン。シンタックスハイライト用。
pub fn lex_all(src: &str) -> (Vec<Token>, Vec<Diagnostic>) {
    Lexer::new(src).run()
}

struct Lexer<'a> {
    src: &'a str,
    bytes: &'a [u8],
    pos: usize,
    tokens: Vec<Token>,
    diags: Vec<Diagnostic>,
}

impl<'a> Lexer<'a> {
    fn new(src: &'a str) -> Self {
        Self {
            src,
            bytes: src.as_bytes(),
            pos: 0,
            tokens: Vec::new(),
            diags: Vec::new(),
        }
    }

    fn run(mut self) -> (Vec<Token>, Vec<Diagnostic>) {
        while self.pos < self.bytes.len() {
            let c = self.bytes[self.pos];
            match c {
                b' ' | b'\t' | b'\r' => self.pos += 1,
                b'#' => self.lex_comment(),
                b'\n' => self.push_single(Tok::Newline),
                b'{' => self.push_single(Tok::LBrace),
                b'}' => self.push_single(Tok::RBrace),
                b'[' => self.push_single(Tok::LBracket),
                b']' => self.push_single(Tok::RBracket),
                b'(' => self.push_single(Tok::LParen),
                b')' => self.push_single(Tok::RParen),
                b',' => self.push_single(Tok::Comma),
                b'=' => self.push_single(Tok::Eq),
                b'"' => self.lex_string(),
                b'0'..=b'9' | b'-' => self.lex_number(),
                _ if is_ident_start(c) => self.lex_ident(),
                _ => {
                    let span = Span::new(self.pos, self.pos + 1);
                    let ch = self.src[self.pos..].chars().next().unwrap_or('?');
                    self.diags.push(Diagnostic::error(
                        span,
                        format!("unexpected character `{ch}`"),
                    ));
                    self.pos += ch.len_utf8();
                }
            }
        }
        let end = self.bytes.len();
        self.tokens.push(Token::new(Tok::Eof, Span::new(end, end)));
        (self.tokens, self.diags)
    }

    fn push_single(&mut self, tok: Tok) {
        let span = Span::new(self.pos, self.pos + 1);
        self.tokens.push(Token::new(tok, span));
        self.pos += 1;
    }

    fn lex_comment(&mut self) {
        let start = self.pos;
        while self.pos < self.bytes.len() && self.bytes[self.pos] != b'\n' {
            self.pos += 1;
        }
        let span = Span::new(start, self.pos);
        self.tokens.push(Token::new(
            Tok::Comment(self.src[start..self.pos].into()),
            span,
        ));
    }

    fn lex_string(&mut self) {
        let start = self.pos;
        self.pos += 1;
        let mut value = String::new();
        loop {
            if self.pos >= self.bytes.len() || self.bytes[self.pos] == b'\n' {
                let span = Span::new(start, self.pos);
                self.diags
                    .push(Diagnostic::error(span, "unterminated string"));
                self.tokens.push(Token::new(Tok::Str(value), span));
                return;
            }
            match self.bytes[self.pos] {
                b'"' => {
                    self.pos += 1;
                    let span = Span::new(start, self.pos);
                    self.tokens.push(Token::new(Tok::Str(value), span));
                    return;
                }
                b'\\' if self.pos + 1 < self.bytes.len() => {
                    let esc = self.bytes[self.pos + 1];
                    value.push(match esc {
                        b'n' => '\n',
                        b't' => '\t',
                        _ => esc as char,
                    });
                    self.pos += 2;
                }
                _ => {
                    let ch = self.src[self.pos..].chars().next().unwrap_or('?');
                    value.push(ch);
                    self.pos += ch.len_utf8();
                }
            }
        }
    }

    fn lex_number(&mut self) {
        let start = self.pos;
        if self.bytes[self.pos] == b'-' {
            self.pos += 1;
        }
        while self.pos < self.bytes.len()
            && (self.bytes[self.pos].is_ascii_digit() || self.bytes[self.pos] == b'.')
        {
            self.pos += 1;
        }
        let span = Span::new(start, self.pos);
        self.tokens
            .push(Token::new(Tok::Num(self.src[start..self.pos].into()), span));
    }

    fn lex_ident(&mut self) {
        let start = self.pos;
        while self.pos < self.bytes.len() && is_ident_continue(self.bytes[self.pos]) {
            self.pos += 1;
        }
        let name = &self.src[start..self.pos];

        // `eval(` は中身を生のまま1トークンにする。SQL式をDSLの字句解析にかけない。
        if name == "eval" && self.peek_non_space() == Some(b'(') {
            if let Some(body) = self.lex_eval_body() {
                let span = Span::new(start, self.pos);
                self.tokens.push(Token::new(Tok::Eval(body), span));
                return;
            }
            let span = Span::new(start, self.pos);
            self.diags
                .push(Diagnostic::error(span, "unterminated `eval(`"));
            self.tokens.push(Token::new(Tok::Eval(String::new()), span));
            return;
        }

        let span = Span::new(start, self.pos);
        self.tokens.push(Token::new(Tok::Ident(name.into()), span));
    }

    fn peek_non_space(&self) -> Option<u8> {
        self.bytes[self.pos..]
            .iter()
            .find(|&&c| c != b' ' && c != b'\t')
            .copied()
    }

    /// `(` から対応する `)` までを読み、中身を返す。読めなければ位置を戻さず None。
    fn lex_eval_body(&mut self) -> Option<String> {
        while self.pos < self.bytes.len() && matches!(self.bytes[self.pos], b' ' | b'\t') {
            self.pos += 1;
        }
        if self.bytes.get(self.pos) != Some(&b'(') {
            return None;
        }
        self.pos += 1;
        let body_start = self.pos;
        let mut depth = 1usize;
        while self.pos < self.bytes.len() {
            match self.bytes[self.pos] {
                b'(' => depth += 1,
                b')' => {
                    depth -= 1;
                    if depth == 0 {
                        let body = self.src[body_start..self.pos].trim().to_string();
                        self.pos += 1;
                        return Some(body);
                    }
                }
                b'\n' => return None,
                _ => {}
            }
            self.pos += 1;
        }
        None
    }
}

fn is_ident_start(c: u8) -> bool {
    c == b'_' || c.is_ascii_alphabetic()
}

fn is_ident_continue(c: u8) -> bool {
    c == b'_' || c.is_ascii_alphanumeric()
}
