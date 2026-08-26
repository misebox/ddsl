//! NounSQL コンパイラの WebAssembly バインディング。
//!
//! ブラウザ上で解析・診断・DDL 生成・シンタックスハイライトを行う。

use nounsql_core::span::{LineIndex, Span};
use nounsql_core::{Severity, codegen, dialect, highlight, ir, parse, resolve};
use serde::Serialize;
use wasm_bindgen::JsCast;
use wasm_bindgen::prelude::*;

// 直列化した中間表現の TypeScript 型。ir.rs の定義と対応させる。
#[wasm_bindgen(typescript_custom_section)]
const TYPES: &'static str = r#"
export type Severity = "error" | "warning";

export interface Diagnostic {
  severity: Severity;
  message: string;
  line: number;
  column: number;
  endLine: number;
  endColumn: number;
}

/** リテラルか、DB 側で評価される式。 */
export type Value = { literal: string } | { eval: string };

/** 定義がどこから来たか。 */
export type Origin =
  | { kind: "own" }
  | { kind: "mixin"; name: string }
  | { kind: "blueprint"; name: string }
  | { kind: "generated"; by: string };

export interface Column {
  name: string;
  type: string;
  null: boolean;
  default: Value | null;
  onUpdate: Value | null;
  comment: string | null;
  origin: Origin;
}

export interface Index {
  name: string;
  columns: string[];
  unique: boolean;
}

export interface ForeignKey {
  /** FK を持つ側の関連名。 */
  alias: string;
  columns: string[];
  refTable: string;
  refColumns: string[];
  onDelete: string;
  onUpdate: string;
}

/** 参照される側から見た関連。 */
export interface Reverse {
  alias: string;
  fromTable: string;
  via: string[];
  unique: boolean;
}

export interface Table {
  /** 最終テーブル名。 */
  name: string;
  /** 単数形。ORM のモデル名などに使う。 */
  singular: string | null;
  plural: string | null;
  comment: string | null;
  /** 宣言順のカラム。 */
  columns: Record<string, Column>;
  pk: string[];
  indexes: Index[];
  foreignKeys: ForeignKey[];
  reverses: Reverse[];
  origin: Origin;
}

/** 解決済みスキーマ。 */
export interface Schema {
  tables: Table[];
}

export interface CompileOutput {
  /** エラーがあれば空。 */
  sql: string;
  ir: Schema;
  diagnostics: Diagnostic[];
  tables: number;
  columns: number;
  errors: number;
  warnings: number;
}
"#;

#[wasm_bindgen]
unsafe extern "C" {
    #[wasm_bindgen(typescript_type = "CompileOutput")]
    pub type CompileOutput;
}

/// 1件の診断。位置は 1 始まりの行・列。
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Diagnostic {
    pub severity: &'static str,
    pub message: String,
    pub line: usize,
    pub column: usize,
    pub end_line: usize,
    pub end_column: usize,
}

/// コンパイル結果。エラーがあれば `sql` は空、`ir` はテーブルが空になる。
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Output {
    pub sql: String,
    /// 解決済みスキーマ。ORM のモデル生成などに使う。
    pub ir: ir::Schema,
    pub diagnostics: Vec<Diagnostic>,
    pub tables: usize,
    pub columns: usize,
    pub errors: usize,
    pub warnings: usize,
}

/// 使える出力ターゲットの名前。
#[wasm_bindgen]
pub fn dialects() -> Vec<String> {
    dialect::names().into_iter().map(String::from).collect()
}

/// NounSQL をコンパイルする。
///
/// `dialect_name` が空なら既定のターゲットを使う。
#[wasm_bindgen]
pub fn compile(source: &str, dialect_name: &str) -> Result<CompileOutput, JsValue> {
    let dialect = if dialect_name.is_empty() {
        dialect::default()
    } else {
        dialect::by_name(dialect_name)
            .ok_or_else(|| JsValue::from_str(&format!("知らない dialect `{dialect_name}`")))?
    };

    let index = LineIndex::new(source);
    let (doc, mut diags) = parse(source);

    let mut sql = String::new();
    let mut ir = ir::Schema::default();
    let mut tables = 0;
    let mut columns = 0;

    if !diags.iter().any(|d| d.severity == Severity::Error) {
        let (schema, mut resolved) = resolve(&doc, dialect);
        let failed = resolved.iter().any(|d| d.severity == Severity::Error);
        diags.append(&mut resolved);
        if !failed {
            sql = codegen::emit(dialect, &schema);
            tables = schema.tables.len();
            columns = schema.tables.iter().map(|t| t.columns.len()).sum();
            ir = schema;
        }
    }

    let errors = diags
        .iter()
        .filter(|d| d.severity == Severity::Error)
        .count();
    let output = Output {
        sql,
        ir,
        tables,
        columns,
        errors,
        warnings: diags.len() - errors,
        diagnostics: diags.iter().map(|d| to_diagnostic(&index, d)).collect(),
    };

    // 既定では構造体が Map になり JS からプロパティで引けないので、
    // 素のオブジェクトになる直列化を使う。
    let serializer = serde_wasm_bindgen::Serializer::json_compatible();
    output
        .serialize(&serializer)
        .map(JsCast::unchecked_into)
        .map_err(|e| JsValue::from_str(&e.to_string()))
}

/// NounSQL のソースを色付けした HTML にする。CLI やサイトと同じ字句解析を使う。
#[wasm_bindgen]
pub fn highlight_html(source: &str) -> String {
    highlight::to_html(source)
}

/// 生成した DDL を色付けした HTML にする。
#[wasm_bindgen]
pub fn highlight_sql_html(sql: &str) -> String {
    highlight::sql_to_html(sql)
}

fn to_diagnostic(index: &LineIndex, d: &nounsql_core::Diagnostic) -> Diagnostic {
    let start = index.line_col(d.span.start);
    let end = index.line_col(end_of(d.span));
    Diagnostic {
        severity: match d.severity {
            Severity::Error => "error",
            Severity::Warning => "warning",
        },
        message: d.message.clone(),
        line: start.line,
        column: start.col,
        end_line: end.line,
        end_column: end.col,
    }
}

fn end_of(span: Span) -> usize {
    span.end.max(span.start)
}
