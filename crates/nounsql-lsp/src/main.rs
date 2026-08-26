mod analysis;
mod text;

use std::collections::HashMap;

use nounsql_core::ast::Document;
use nounsql_core::dialect;
use nounsql_core::{Severity, parse, resolve};
use tokio::sync::RwLock;
use tower_lsp::jsonrpc::Result;
use tower_lsp::lsp_types::*;
use tower_lsp::{Client, LanguageServer, LspService, Server};

use analysis::{Reference, definition_span, reference_at};
use text::TextMap;

const STATEMENT_KEYWORDS: &[&str] = &[
    "column",
    "pk",
    "index",
    "use",
    "override",
    "except",
    "belongs_to",
    "unique_belongs_to",
    "has_many",
    "has_one",
    "let",
    "unique",
];

const BLOCK_KEYWORDS: &[&str] = &[
    "table",
    "mixin",
    "blueprint",
    "naming",
    "constraints",
    "nouns",
];

const ATTR_KEYS: &[&str] = &[
    "type",
    "null",
    "default",
    "on_update",
    "comment",
    "fk",
    "alias",
    "via",
];

struct Analyzed {
    map: TextMap,
    doc: Document,
}

struct Backend {
    client: Client,
    docs: RwLock<HashMap<Url, Analyzed>>,
}

impl Backend {
    async fn refresh(&self, uri: Url, text: String, version: Option<i32>) {
        let map = TextMap::new(text);
        let (doc, mut diags) = parse(map.text());
        let (_, mut resolved) = resolve(&doc, dialect::default());
        diags.append(&mut resolved);

        let lsp_diags = diags
            .iter()
            .map(|d| Diagnostic {
                range: map.range(d.span.start, d.span.end),
                severity: Some(match d.severity {
                    Severity::Error => DiagnosticSeverity::ERROR,
                    Severity::Warning => DiagnosticSeverity::WARNING,
                }),
                source: Some("nounsql".into()),
                message: d.message.clone(),
                related_information: related(&map, &uri, d),
                ..Default::default()
            })
            .collect();

        self.client
            .publish_diagnostics(uri.clone(), lsp_diags, version)
            .await;
        self.docs.write().await.insert(uri, Analyzed { map, doc });
    }
}

fn related(
    map: &TextMap,
    uri: &Url,
    d: &nounsql_core::Diagnostic,
) -> Option<Vec<DiagnosticRelatedInformation>> {
    if d.labels.is_empty() {
        return None;
    }
    Some(
        d.labels
            .iter()
            .map(|l| DiagnosticRelatedInformation {
                location: Location {
                    uri: uri.clone(),
                    range: map.range(l.span.start, l.span.end),
                },
                message: l.message.clone(),
            })
            .collect(),
    )
}

#[tower_lsp::async_trait]
impl LanguageServer for Backend {
    async fn initialize(&self, _: InitializeParams) -> Result<InitializeResult> {
        Ok(InitializeResult {
            server_info: Some(ServerInfo {
                name: "nounsql-lsp".into(),
                version: Some(env!("CARGO_PKG_VERSION").into()),
            }),
            capabilities: ServerCapabilities {
                text_document_sync: Some(TextDocumentSyncCapability::Kind(
                    TextDocumentSyncKind::FULL,
                )),
                definition_provider: Some(OneOf::Left(true)),
                document_symbol_provider: Some(OneOf::Left(true)),
                hover_provider: Some(HoverProviderCapability::Simple(true)),
                completion_provider: Some(CompletionOptions {
                    trigger_characters: Some(vec!["=".into()]),
                    ..Default::default()
                }),
                ..Default::default()
            },
        })
    }

    async fn initialized(&self, _: InitializedParams) {
        self.client
            .log_message(MessageType::INFO, "nounsql-lsp ready")
            .await;
    }

    async fn shutdown(&self) -> Result<()> {
        Ok(())
    }

    async fn did_open(&self, params: DidOpenTextDocumentParams) {
        let d = params.text_document;
        self.refresh(d.uri, d.text, Some(d.version)).await;
    }

    async fn did_change(&self, mut params: DidChangeTextDocumentParams) {
        let Some(change) = params.content_changes.pop() else {
            return;
        };
        self.refresh(
            params.text_document.uri,
            change.text,
            Some(params.text_document.version),
        )
        .await;
    }

    async fn did_close(&self, params: DidCloseTextDocumentParams) {
        self.docs.write().await.remove(&params.text_document.uri);
    }

    async fn goto_definition(
        &self,
        params: GotoDefinitionParams,
    ) -> Result<Option<GotoDefinitionResponse>> {
        let uri = params.text_document_position_params.text_document.uri;
        let docs = self.docs.read().await;
        let Some(a) = docs.get(&uri) else {
            return Ok(None);
        };
        let offset = a.map.offset(params.text_document_position_params.position);
        let Some((r, _)) = reference_at(&a.doc, offset) else {
            return Ok(None);
        };
        let Some(span) = definition_span(&a.doc, &r) else {
            return Ok(None);
        };
        Ok(Some(GotoDefinitionResponse::Scalar(Location {
            uri,
            range: a.map.range(span.start, span.end),
        })))
    }

    async fn hover(&self, params: HoverParams) -> Result<Option<Hover>> {
        let uri = params.text_document_position_params.text_document.uri;
        let docs = self.docs.read().await;
        let Some(a) = docs.get(&uri) else {
            return Ok(None);
        };
        let offset = a.map.offset(params.text_document_position_params.position);
        let Some((r, span)) = reference_at(&a.doc, offset) else {
            return Ok(None);
        };
        let text = match &r {
            Reference::Mixin(n) => format!("mixin `{n}`"),
            Reference::Blueprint(n) => format!("blueprint `{n}`"),
            Reference::Noun(n) => {
                let (schema, _) = resolve(&a.doc, dialect::default());
                match schema.table_by_noun(n) {
                    Some(t) => format!("名詞 `{n}` → テーブル `{}`", t.name),
                    None => format!("名詞 `{n}`"),
                }
            }
        };
        Ok(Some(Hover {
            contents: HoverContents::Markup(MarkupContent {
                kind: MarkupKind::Markdown,
                value: text,
            }),
            range: Some(a.map.range(span.start, span.end)),
        }))
    }

    async fn document_symbol(
        &self,
        params: DocumentSymbolParams,
    ) -> Result<Option<DocumentSymbolResponse>> {
        let docs = self.docs.read().await;
        let Some(a) = docs.get(&params.text_document.uri) else {
            return Ok(None);
        };
        let mut symbols = Vec::new();
        for t in &a.doc.tables {
            symbols.push(block_symbol(
                a,
                &t.name.value,
                t.span,
                t.name.span,
                SymbolKind::CLASS,
            ));
        }
        for m in &a.doc.mixins {
            symbols.push(block_symbol(
                a,
                &m.name.value,
                m.span,
                m.name.span,
                SymbolKind::INTERFACE,
            ));
        }
        for b in &a.doc.blueprints {
            symbols.push(block_symbol(
                a,
                &b.name.value,
                b.span,
                b.name.span,
                SymbolKind::MODULE,
            ));
        }
        Ok(Some(DocumentSymbolResponse::Nested(symbols)))
    }

    async fn completion(&self, params: CompletionParams) -> Result<Option<CompletionResponse>> {
        let docs = self.docs.read().await;
        let Some(a) = docs.get(&params.text_document_position.text_document.uri) else {
            return Ok(None);
        };
        let mut items = Vec::new();
        let mut push = |label: &str, kind: CompletionItemKind, detail: &str| {
            items.push(CompletionItem {
                label: label.into(),
                kind: Some(kind),
                detail: Some(detail.into()),
                ..Default::default()
            });
        };
        for kw in STATEMENT_KEYWORDS {
            push(kw, CompletionItemKind::KEYWORD, "宣言文");
        }
        for kw in BLOCK_KEYWORDS {
            push(kw, CompletionItemKind::KEYWORD, "ブロック");
        }
        for key in ATTR_KEYS {
            push(key, CompletionItemKind::PROPERTY, "属性キー");
        }
        for ty in dialect::default().types {
            push(ty, CompletionItemKind::TYPE_PARAMETER, "型");
        }
        for m in &a.doc.mixins {
            push(&m.name.value, CompletionItemKind::INTERFACE, "mixin");
        }
        for b in &a.doc.blueprints {
            push(&b.name.value, CompletionItemKind::MODULE, "blueprint");
        }
        if let Some(e) = &a.doc.nouns {
            for entry in &e.entries {
                push(&entry.singular.value, CompletionItemKind::CLASS, "名詞");
            }
        }
        Ok(Some(CompletionResponse::Array(items)))
    }
}

#[allow(deprecated)]
fn block_symbol(
    a: &Analyzed,
    name: &str,
    span: nounsql_core::span::Span,
    name_span: nounsql_core::span::Span,
    kind: SymbolKind,
) -> DocumentSymbol {
    DocumentSymbol {
        name: name.into(),
        detail: None,
        kind,
        tags: None,
        deprecated: None,
        range: a.map.range(span.start, span.end),
        selection_range: a.map.range(name_span.start, name_span.end),
        children: None,
    }
}

#[tokio::main]
async fn main() {
    let stdin = tokio::io::stdin();
    let stdout = tokio::io::stdout();
    let (service, socket) = LspService::new(|client| Backend {
        client,
        docs: RwLock::new(HashMap::new()),
    });
    Server::new(stdin, stdout, socket).serve(service).await;
}
