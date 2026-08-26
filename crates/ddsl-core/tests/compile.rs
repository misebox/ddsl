use ddsl_core::{Severity, codegen, dialect, parse, resolve};

const SAMPLE: &str = include_str!("../../../examples/sample.ddsl");
const EXPECTED_SQL: &str = include_str!("../../../examples/sample.sql");

fn compile(src: &str) -> (ddsl_core::ir::Schema, Vec<ddsl_core::Diagnostic>) {
    let (doc, mut diags) = parse(src);
    let (schema, mut rd) = resolve(&doc, dialect::default());
    diags.append(&mut rd);
    (schema, diags)
}

fn errors(diags: &[ddsl_core::Diagnostic]) -> Vec<&str> {
    diags
        .iter()
        .filter(|d| d.severity == Severity::Error)
        .map(|d| d.message.as_str())
        .collect()
}

#[test]
fn sample_compiles_without_diagnostics() {
    let (_, diags) = compile(SAMPLE);
    assert_eq!(diags.len(), 0, "{:?}", errors(&diags));
}

#[test]
fn sample_ddl_is_stable() {
    let (schema, _) = compile(SAMPLE);
    assert_eq!(codegen::emit(dialect::default(), &schema), EXPECTED_SQL);
}

#[test]
fn mixin_expands_at_use_position() {
    let (schema, _) = compile(SAMPLE);
    let order_items = schema.table("order_items").expect("order_items");
    let cols: Vec<&str> = order_items.columns.keys().map(String::as_str).collect();
    assert_eq!(
        cols,
        vec![
            "id",
            "created_at",
            "updated_at",
            "order_id",
            "product_id",
            "quantity"
        ]
    );
}

#[test]
fn except_removes_mixin_column_and_its_trigger() {
    let (schema, _) = compile(SAMPLE);
    let categories = schema.table("categories").expect("categories");
    assert!(!categories.columns.contains_key("updated_at"));
}

#[test]
fn except_index_removes_mixin_index() {
    let (schema, _) = compile(SAMPLE);
    let posts = schema.table("posts").expect("posts");
    assert!(posts.columns.contains_key("published_at"));
    assert!(!posts.indexes.iter().any(|i| i.columns == ["published_at"]));
    assert!(posts.indexes.iter().any(|i| i.columns == ["status"]));
}

#[test]
fn override_merges_only_named_keys() {
    let (schema, _) = compile(SAMPLE);
    let note = &schema.table("categories").expect("categories").columns["note"];
    assert_eq!(note.ty, "text");
    assert!(!note.null);
}

#[test]
fn blueprint_generates_table_via_name_join() {
    let (schema, _) = compile(SAMPLE);
    let t = schema.table("post_histories").expect("post_histories");
    assert!(t.columns.contains_key("post_id"));
    assert!(t.columns.contains_key("user_id"));
}

#[test]
fn associate_makes_composite_pk() {
    let (schema, _) = compile(SAMPLE);
    let t = schema.table("user_post").expect("user_post");
    assert_eq!(t.pk, vec!["user_id", "post_id"]);
}

#[test]
fn fk_type_follows_referenced_serial_pk() {
    let (schema, _) = compile(SAMPLE);
    let user_id = &schema.table("orders").expect("orders").columns["user_id"];
    assert_eq!(user_id.ty, "integer");
}

#[test]
fn detects_mixin_cycle() {
    let src = "mixin a {\n  use b\n}\nmixin b {\n  use a\n}\ntable x {\n  use a\n}\n";
    let (_, diags) = compile(src);
    assert!(
        errors(&diags).iter().any(|m| m.contains("循環")),
        "{:?}",
        errors(&diags)
    );
}

#[test]
fn rejects_unknown_type() {
    let src = "table x {\n  column a type=nosuchtype\n}\n";
    let (_, diags) = compile(src);
    assert!(
        errors(&diags)
            .iter()
            .any(|m| m.contains("postgres の型ではない")),
        "{:?}",
        errors(&diags)
    );
}

#[test]
fn rejects_override_of_missing_column() {
    let src = "table x {\n  column a type=text\n  override b null=true\n}\n";
    let (_, diags) = compile(src);
    assert!(
        errors(&diags).iter().any(|m| m.contains("上書きできない")),
        "{:?}",
        errors(&diags)
    );
}

#[test]
fn rejects_blueprint_param_shadowing_entity() {
    let src = concat!(
        "entities {\n  user users \"u\"\n  post posts \"p\"\n}\n",
        "blueprint b user {\n  let t = name_join(user, post)\n  table t {\n    column x type=text\n  }\n}\n",
        "apply_blueprint(b, post)\n"
    );
    let (_, diags) = compile(src);
    assert!(
        errors(&diags).iter().any(|m| m.contains("entity 名と衝突")),
        "{:?}",
        errors(&diags)
    );
}
