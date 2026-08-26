/// ソース内のバイト範囲。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Span {
    pub start: usize,
    pub end: usize,
}

impl Span {
    pub fn new(start: usize, end: usize) -> Self {
        Self { start, end }
    }

    pub fn join(self, other: Span) -> Span {
        Span::new(self.start.min(other.start), self.end.max(other.end))
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Spanned<T> {
    pub value: T,
    pub span: Span,
}

impl<T> Spanned<T> {
    pub fn new(value: T, span: Span) -> Self {
        Self { value, span }
    }
}

/// 1始まりの行・列。診断表示用。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LineCol {
    pub line: usize,
    pub col: usize,
}

/// バイトオフセットから行・列を引くための索引。
pub struct LineIndex {
    line_starts: Vec<usize>,
}

impl LineIndex {
    pub fn new(src: &str) -> Self {
        let mut line_starts = vec![0];
        line_starts.extend(src.match_indices('\n').map(|(i, _)| i + 1));
        Self { line_starts }
    }

    pub fn line_col(&self, offset: usize) -> LineCol {
        let line = self.line_starts.partition_point(|&s| s <= offset).max(1) - 1;
        LineCol {
            line: line + 1,
            col: offset - self.line_starts[line] + 1,
        }
    }

    pub fn line_text<'a>(&self, src: &'a str, line: usize) -> &'a str {
        let start = self.line_starts[line - 1];
        let end = self
            .line_starts
            .get(line)
            .map(|&s| s.saturating_sub(1))
            .unwrap_or(src.len());
        src[start..end].trim_end_matches('\r')
    }
}
