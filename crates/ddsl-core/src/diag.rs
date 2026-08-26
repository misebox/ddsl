use crate::span::{LineIndex, Span};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Severity {
    Error,
    Warning,
}

#[derive(Debug, Clone)]
pub struct Label {
    pub span: Span,
    pub message: String,
}

#[derive(Debug, Clone)]
pub struct Diagnostic {
    pub severity: Severity,
    pub message: String,
    pub span: Span,
    /// 関連位置。mixin や blueprint の定義元を指すのに使う。
    pub labels: Vec<Label>,
}

impl Diagnostic {
    pub fn error(span: Span, message: impl Into<String>) -> Self {
        Self {
            severity: Severity::Error,
            message: message.into(),
            span,
            labels: Vec::new(),
        }
    }

    pub fn warning(span: Span, message: impl Into<String>) -> Self {
        Self {
            severity: Severity::Warning,
            message: message.into(),
            span,
            labels: Vec::new(),
        }
    }

    pub fn with_label(mut self, span: Span, message: impl Into<String>) -> Self {
        self.labels.push(Label {
            span,
            message: message.into(),
        });
        self
    }
}

/// 診断をテキストに描画する。
pub fn render(src: &str, path: &str, diags: &[Diagnostic]) -> String {
    let index = LineIndex::new(src);
    let mut out = String::new();
    for d in diags {
        let pos = index.line_col(d.span.start);
        let tag = match d.severity {
            Severity::Error => "error",
            Severity::Warning => "warning",
        };
        out.push_str(&format!(
            "{tag}: {msg}\n  --> {path}:{line}:{col}\n",
            msg = d.message,
            line = pos.line,
            col = pos.col
        ));
        out.push_str(&render_snippet(src, &index, d.span));
        for label in &d.labels {
            let lpos = index.line_col(label.span.start);
            out.push_str(&format!(
                "  note: {} ({path}:{}:{})\n",
                label.message, lpos.line, lpos.col
            ));
            out.push_str(&render_snippet(src, &index, label.span));
        }
        out.push('\n');
    }
    out
}

fn render_snippet(src: &str, index: &LineIndex, span: Span) -> String {
    let pos = index.line_col(span.start);
    let text = index.line_text(src, pos.line);
    let width = span.end.saturating_sub(span.start).max(1);
    let indent = " ".repeat(pos.col - 1);
    let carets = "^".repeat(width);
    format!("{:>4} | {text}\n     | {indent}{carets}\n", pos.line)
}
