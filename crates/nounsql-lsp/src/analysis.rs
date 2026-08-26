use nounsql_core::ast::{self, Document};
use nounsql_core::span::Span;

/// カーソル位置にある識別子が何を指しているか。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Reference {
    Mixin(String),
    Noun(String),
    Blueprint(String),
}

pub fn reference_at(doc: &Document, offset: usize) -> Option<(Reference, Span)> {
    for m in &doc.mixins {
        if let Some(hit) = member_reference(&m.members, offset) {
            return Some(hit);
        }
    }
    for t in &doc.tables {
        if let Some(hit) = member_reference(&t.members, offset) {
            return Some(hit);
        }
    }
    for b in &doc.blueprints {
        for item in &b.items {
            if let ast::BlueprintItem::Table(t) = item
                && let Some(hit) = member_reference(&t.members, offset)
            {
                return Some(hit);
            }
        }
    }
    for call in &doc.macros {
        for (i, arg) in call.args.iter().enumerate() {
            if !contains(arg.span, offset) {
                continue;
            }
            let r = if call.name.value == "apply_blueprint" && i == 0 {
                Reference::Blueprint(arg.value.clone())
            } else {
                Reference::Noun(arg.value.clone())
            };
            return Some((r, arg.span));
        }
    }
    None
}

fn member_reference(
    members: &[nounsql_core::span::Spanned<ast::Member>],
    offset: usize,
) -> Option<(Reference, Span)> {
    for m in members {
        let (name, make): (_, fn(String) -> Reference) = match &m.value {
            ast::Member::Use(n) => (n, Reference::Mixin),
            ast::Member::Relation(r) => (&r.target, Reference::Noun),
            _ => continue,
        };
        if contains(name.span, offset) {
            return Some((make(name.value.clone()), name.span));
        }
    }
    None
}

pub fn definition_span(doc: &Document, r: &Reference) -> Option<Span> {
    match r {
        Reference::Mixin(name) => doc
            .mixins
            .iter()
            .find(|m| &m.name.value == name)
            .map(|m| m.name.span),
        Reference::Blueprint(name) => doc
            .blueprints
            .iter()
            .find(|b| &b.name.value == name)
            .map(|b| b.name.span),
        Reference::Noun(name) => doc
            .tables
            .iter()
            .find(|t| &t.name.value == name)
            .map(|t| t.name.span)
            .or_else(|| {
                doc.nouns.as_ref().and_then(|e| {
                    e.entries
                        .iter()
                        .find(|x| &x.singular.value == name)
                        .map(|x| x.singular.span)
                })
            }),
    }
}

fn contains(span: Span, offset: usize) -> bool {
    span.start <= offset && offset <= span.end
}
