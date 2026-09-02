use nounsql_core::{Severity, codegen, dialect, parse, resolve};

const SAMPLE: &str = include_str!("../../../examples/sample.nsql");
const EXPECTED_SQL: &str = include_str!("../../../examples/sample.sql");

fn compile(src: &str) -> (nounsql_core::ir::Schema, Vec<nounsql_core::Diagnostic>) {
    let (doc, mut diags) = parse(src);
    let (schema, mut rd) = resolve(&doc, dialect::default());
    diags.append(&mut rd);
    (schema, diags)
}

fn errors(diags: &[nounsql_core::Diagnostic]) -> Vec<&str> {
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
fn associate_defaults_to_the_joined_noun_name() {
    let src = concat!(
        "nouns {\n  user \"u\"\n  post \"p\"\n}\n",
        "mixin base {\n  column id type=serial\n  pk id\n}\n",
        "table user {\n  use base\n}\ntable post {\n  use base\n}\n",
        "associate(user, post)\n"
    );
    let (schema, diags) = compile(src);
    assert_eq!(errors(&diags).len(), 0, "{:?}", errors(&diags));
    let t = schema.table("user_post").expect("user_post");
    assert_eq!(t.pk, vec!["user_id", "post_id"]);
}

#[test]
fn compound_noun_number_follows_context() {
    let src = concat!(
        "nouns {\n  user \"u\"\n  message \"m\"\n",
        "  profile \"p\"\n}\n",
        "mixin base {\n  column id type=serial\n  pk id\n}\n",
        "table user {\n  use base\n",
        "  has_many message alias=noun(\"sent\", message)\n",
        "  has_one profile alias=noun(\"main\", profile)\n}\n",
        "table message {\n  use base\n  belongs_to user\n}\n",
        "table profile {\n  use base\n  unique_belongs_to user\n}\n"
    );
    let (schema, diags) = compile(src);
    assert_eq!(errors(&diags).len(), 0, "{:?}", errors(&diags));
    let users = schema.table("user").expect("user");
    let aliases: Vec<&str> = users.reverses.iter().map(|r| r.alias.as_str()).collect();
    // 同じ noun(...) でも has_many は複数形、has_one は単数形になる
    assert!(aliases.contains(&"sent_messages"), "{aliases:?}");
    assert!(aliases.contains(&"main_profile"), "{aliases:?}");
}

#[test]
fn compound_noun_inflects_only_the_last_element() {
    let (schema, _) = compile(SAMPLE);
    // blueprint の let noun(target, history) が post_histories になる
    assert!(schema.table("post_histories").is_some());
}

#[test]
fn fk_type_follows_referenced_serial_pk() {
    let (schema, _) = compile(SAMPLE);
    let user_id = &schema.table("orders").expect("orders").columns["user_id"];
    assert_eq!(user_id.ty, "integer");
}

#[test]
fn reverse_relations_default_to_plural_and_singular() {
    let (schema, _) = compile(SAMPLE);
    let users = schema.table("users").expect("users");
    let by_alias = |a: &str| users.reverses.iter().find(|r| r.alias == a).cloned();
    assert!(by_alias("posts").is_some_and(|r| !r.unique));
    assert!(by_alias("profile").is_some_and(|r| r.unique));
    // 明示していない belongs_to にも逆参照が付く
    assert!(by_alias("orders").is_some());
}

#[test]
fn belongs_to_alias_defaults_to_singular() {
    let (schema, _) = compile(SAMPLE);
    let posts = schema.table("posts").expect("posts");
    let aliases: Vec<&str> = posts
        .foreign_keys
        .iter()
        .map(|f| f.alias.as_str())
        .collect();
    assert_eq!(aliases, vec!["user", "category"]);
}

const TWO_FKS: &str = concat!(
    "nouns {\n  user \"u\"\n  message \"m\"\n}\n",
    "mixin base {\n  column id type=serial\n  pk id\n}\n",
    "table user {\n  use base\n",
    "  has_many message via=\"sender_id\" alias=\"sent\"\n",
    "  has_many message via=\"receiver_id\" alias=\"received\"\n}\n",
    "table message {\n  use base\n",
    "  belongs_to user fk=\"sender_id\" alias=\"sender\"\n",
    "  belongs_to user fk=\"receiver_id\" alias=\"receiver\"\n}\n"
);

#[test]
fn two_fks_to_same_table_resolve_by_fk_and_via() {
    let (schema, diags) = compile(TWO_FKS);
    assert_eq!(errors(&diags).len(), 0, "{:?}", errors(&diags));
    let messages = schema.table("message").expect("message");
    let cols: Vec<&str> = messages.columns.keys().map(String::as_str).collect();
    assert_eq!(cols, vec!["id", "sender_id", "receiver_id"]);
    let users = schema.table("user").expect("user");
    let mut aliases: Vec<&str> = users.reverses.iter().map(|r| r.alias.as_str()).collect();
    aliases.sort_unstable();
    assert_eq!(aliases, vec!["received", "sent"]);
}

#[test]
fn requires_via_when_several_fks_match() {
    let src = TWO_FKS.replace("via=\"sender_id\" alias=\"sent\"", "alias=\"sent\"");
    let (_, diags) = compile(&src);
    assert!(
        errors(&diags)
            .iter()
            .any(|m| m.contains("pick one with `via=`")),
        "{:?}",
        errors(&diags)
    );
}

#[test]
fn rejects_has_many_on_one_to_one() {
    let src = concat!(
        "nouns {\n  user \"u\"\n  profile \"p\"\n}\n",
        "mixin base {\n  column id type=serial\n  pk id\n}\n",
        "table user {\n  use base\n  has_many profile\n}\n",
        "table profile {\n  use base\n  unique_belongs_to user\n}\n"
    );
    let (_, diags) = compile(src);
    assert!(
        errors(&diags).iter().any(|m| m.contains("has_one")),
        "{:?}",
        errors(&diags)
    );
}

#[test]
fn rejects_duplicate_relation_alias() {
    let src = TWO_FKS.replace("alias=\"received\"", "alias=\"sent\"");
    let (_, diags) = compile(&src);
    assert!(
        errors(&diags).iter().any(|m| m.contains("duplicate")),
        "{:?}",
        errors(&diags)
    );
}

#[test]
fn belongs_to_comment_lands_on_the_fk_column() {
    let (schema, _) = compile(SAMPLE);
    let orders = schema.table("orders").expect("orders");
    assert_eq!(
        orders.columns["user_id"].comment.as_deref(),
        Some("Who placed it")
    );
}

#[test]
fn associate_name_and_comment_apply() {
    let (schema, _) = compile(SAMPLE);
    let t = schema.table("favorites").expect("favorites");
    assert_eq!(t.comment.as_deref(), Some("A user liking a post"));
    assert_eq!(t.pk, vec!["user_id", "post_id"]);
}

#[test]
fn comment_template_expands_desc() {
    let (schema, _) = compile(SAMPLE);
    let t = schema.table("post_histories").expect("post_histories");
    assert_eq!(t.comment.as_deref(), Some("Past states of a post"));
}

#[test]
fn table_comment_wins_over_the_dictionary() {
    let src = concat!(
        "nouns {\n  user \"辞書の説明\"\n}\n",
        "table user {\n  comment \"テーブルの説明\"\n",
        "  column id type=serial\n  pk id\n}\n"
    );
    let (schema, _) = compile(src);
    assert_eq!(
        schema.table("user").expect("user").comment.as_deref(),
        Some("テーブルの説明")
    );
}

#[test]
fn rejects_comment_in_a_mixin() {
    let src = "mixin m {\n  comment \"だめ\"\n}\n";
    let (_, diags) = compile(src);
    assert!(
        errors(&diags)
            .iter()
            .any(|m| m.contains("only allowed in `table`")),
        "{:?}",
        errors(&diags)
    );
}

#[test]
fn blueprint_argument_must_be_a_registered_noun() {
    let src = concat!(
        "nouns {\n  post \"投稿\"\n}\n",
        "mixin base {\n  column id type=serial\n  pk id\n}\n",
        "blueprint b target {\n  table t {\n    name noun(target, post)\n    use base\n  }\n}\n",
        "apply_blueprint(b, unknown)\n"
    );
    let (_, diags) = compile(src);
    assert!(
        errors(&diags).iter().any(|m| m.contains("must be a noun")),
        "{:?}",
        errors(&diags)
    );
}

#[test]
fn rejects_comment_on_has_many() {
    let src = concat!(
        "nouns {\n  user \"u\"\n  post \"p\"\n}\n",
        "mixin base {\n  column id type=serial\n  pk id\n}\n",
        "table user {\n  use base\n  has_many post comment=\"だめ\"\n}\n",
        "table post {\n  use base\n  belongs_to user\n}\n"
    );
    let (_, diags) = compile(src);
    assert!(
        errors(&diags)
            .iter()
            .any(|m| m.contains("`comment=` is not allowed")),
        "{:?}",
        errors(&diags)
    );
}

#[test]
fn name_composes_a_noun_that_is_not_registered() {
    let src = concat!(
        "nouns {\n  user \"u\"\n  role \"r\"\n}\n",
        "mixin base {\n  column id type=serial\n  pk id\n}\n",
        "table user { use base }\ntable role { use base }\n",
        "table user_role {\n  name noun(user, role)\n",
        "  belongs_to user\n  belongs_to role\n  pk [user_id, role_id]\n}\n"
    );
    let (schema, diags) = compile(src);
    assert_eq!(errors(&diags).len(), 0, "{:?}", errors(&diags));
    assert!(schema.table("user_role").is_some());
}

#[test]
fn a_literal_name_sets_only_the_table_name() {
    let src = concat!(
        "nouns {\n  customer \"c\"\n}\n",
        "table customer {\n  name \"tbl_cust_master\"\n",
        "  column id type=serial\n  pk id\n}\n"
    );
    let (schema, diags) = compile(src);
    assert_eq!(errors(&diags).len(), 0, "{:?}", errors(&diags));
    let table = schema.table("tbl_cust_master").expect("tbl_cust_master");
    // 名詞は識別子のまま。FK 列名はそこから作られる。
    assert_eq!(table.singular.as_deref(), Some("customer"));
}

#[test]
fn detects_a_cycle_between_table_names() {
    let src = concat!(
        "nouns {\n  a \"a\"\n  b \"b\"\n}\n",
        "mixin m {\n  column id type=serial\n  pk id\n}\n",
        "table x {\n  name noun(y, a)\n  use m\n}\n",
        "table y {\n  name noun(x, b)\n  use m\n}\n"
    );
    let (_, diags) = compile(src);
    assert!(
        errors(&diags).iter().any(|m| m.contains("circular")),
        "{:?}",
        errors(&diags)
    );
}

#[test]
fn rejects_name_on_an_identifier_that_is_a_noun() {
    let src = concat!(
        "nouns {\n  user \"u\"\n  role \"r\"\n}\n",
        "table user {\n  name noun(user, role)\n",
        "  column id type=serial\n  pk id\n}\n"
    );
    let (_, diags) = compile(src);
    assert!(
        errors(&diags)
            .iter()
            .any(|m| m.contains("collides with a noun")),
        "{:?}",
        errors(&diags)
    );
}

#[test]
fn detects_mixin_cycle() {
    let src = "mixin a {\n  use b\n}\nmixin b {\n  use a\n}\ntable x {\n  use a\n}\n";
    let (_, diags) = compile(src);
    assert!(
        errors(&diags).iter().any(|m| m.contains("circular")),
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
            .any(|m| m.contains("is not a postgres type")),
        "{:?}",
        errors(&diags)
    );
}

#[test]
fn rejects_override_of_missing_column() {
    let src = "table x {\n  column a type=text\n  override b null=true\n}\n";
    let (_, diags) = compile(src);
    assert!(
        errors(&diags).iter().any(|m| m.contains("cannot override")),
        "{:?}",
        errors(&diags)
    );
}

#[test]
fn rejects_blueprint_param_shadowing_noun() {
    let src = concat!(
        "nouns {\n  user \"u\"\n  post \"p\"\n}\n",
        "blueprint b user {\n  table t {\n    name noun(user, post)\n    column x type=text\n  }\n}\n",
        "apply_blueprint(b, post)\n"
    );
    let (_, diags) = compile(src);
    assert!(
        errors(&diags)
            .iter()
            .any(|m| m.contains("collides with a noun")),
        "{:?}",
        errors(&diags)
    );
}

/// 直列化した中間表現のキーは nounsql-wasm の TypeScript 型定義と対応している。
/// 片方だけ変えると npm の利用者が壊れるので、ここで気づけるようにする。
#[cfg(feature = "serde")]
#[test]
fn serialized_ir_keys_are_stable() {
    use serde_json::Value;

    let (schema, _) = compile(SAMPLE);
    let json = serde_json::to_value(&schema).expect("直列化");

    let keys = |v: &Value| -> Vec<String> {
        v.as_object()
            .expect("オブジェクト")
            .keys()
            .cloned()
            .collect()
    };

    assert_eq!(keys(&json), vec!["tables"]);

    let table = &json["tables"][0];
    assert_eq!(
        keys(table),
        vec![
            "name",
            "singular",
            "plural",
            "comment",
            "columns",
            "pk",
            "indexes",
            "foreignKeys",
            "reverses",
            "origin",
        ]
    );

    let column = table["columns"]
        .as_object()
        .expect("カラム")
        .values()
        .next()
        .expect("1つ以上");
    assert_eq!(
        keys(column),
        vec![
            "name", "type", "null", "default", "onUpdate", "comment", "origin"
        ]
    );

    // カラムは宣言順のまま出る。DDL の列順がこれで決まる。
    let names: Vec<&str> = table["columns"]
        .as_object()
        .expect("カラム")
        .keys()
        .map(String::as_str)
        .collect();
    assert_eq!(
        names,
        vec!["id", "created_at", "updated_at", "email", "name"]
    );

    // span のようなソース依存の情報は出さない。
    assert!(!json.to_string().contains("span"));
}

fn warnings(diags: &[nounsql_core::Diagnostic]) -> Vec<&str> {
    diags
        .iter()
        .filter(|d| d.severity == Severity::Warning)
        .map(|d| d.message.as_str())
        .collect()
}

#[test]
fn warns_on_two_indexes_over_the_same_columns() {
    let (_, diags) = compile(
        "nouns {\n  user \"A person\"\n}\n\
         table user {\n  column id type=serial\n  column email type=text\n\
         \x20 pk id\n  index email\n  index email\n}\n",
    );
    assert!(
        warnings(&diags)
            .iter()
            .any(|m| m.contains("indexes on the same columns")),
        "{:?}",
        diags
    );
}

#[test]
fn unique_belongs_to_index_counts_as_a_duplicate() {
    let (_, diags) = compile(
        "nouns {\n  user \"A person\"\n  profile \"Details\"\n}\n\
         table user {\n  column id type=serial\n  pk id\n}\n\
         table profile {\n  column id type=serial\n  unique_belongs_to user\n\
         \x20 pk id\n  index user_id\n}\n",
    );
    assert!(
        warnings(&diags)
            .iter()
            .any(|m| m.contains("indexes on the same columns")),
        "{:?}",
        diags
    );
}

#[test]
fn warns_when_foreign_key_index_is_off_and_no_index_covers_it() {
    let (_, diags) = compile(
        "constraints {\n  foreign_key_index = false\n}\n\
         nouns {\n  user \"A person\"\n  post \"An article\"\n}\n\
         table user {\n  column id type=serial\n  pk id\n}\n\
         table post {\n  column id type=serial\n  belongs_to user\n  pk id\n}\n",
    );
    assert!(
        warnings(&diags)
            .iter()
            .any(|m| m.contains("no index on foreign key columns")),
        "{:?}",
        diags
    );
}

#[test]
fn an_index_over_the_foreign_key_satisfies_that_warning() {
    let (_, diags) = compile(
        "constraints {\n  foreign_key_index = false\n}\n\
         nouns {\n  member \"A person\"\n  post \"An article\"\n}\n\
         table member {\n  column id type=serial\n  pk id\n}\n\
         table post {\n  column id type=serial\n  belongs_to member\n  pk id\n\
         \x20 index member_id, id\n}\n",
    );
    assert_eq!(warnings(&diags), Vec::<&str>::new());
}

// ---------- 名詞の識別子と語形 ----------

#[test]
fn the_identifier_seeds_the_singular_but_never_reaches_the_output() {
    let (schema, diags) = compile(concat!(
        "nouns {\n  u short=usr singular=member plural=members \"A person\"\n}\n",
        "table u {\n  column id type=serial\n  pk id\n  index id\n}\n",
    ));
    assert_eq!(errors(&diags).len(), 0, "{:?}", errors(&diags));
    let t = schema.table("member").expect("member");
    assert_eq!(t.singular.as_deref(), Some("member"));
    assert_eq!(t.plural.as_deref(), Some("members"));
    // `u` はソースにしか出ない。
    assert!(!nounsql_core::codegen::emit(dialect::default(), &schema).contains(" u "));
}

#[test]
fn omitted_word_forms_fall_back_through_the_singular() {
    let (schema, _) = compile(concat!(
        "nouns {\n  category \"A grouping\"\n}\n",
        "naming {\n  table_name = plural\n  index = \"idx_${short(table)}_${columns}\"\n}\n",
        "table category {\n  column id type=serial\n  pk id\n  index id\n}\n",
    ));
    let t = schema.table("categories").expect("categories");
    assert_eq!(t.singular.as_deref(), Some("category"));
    // 複数形は単数形の規則変化、略語は単数形。
    assert_eq!(t.plural.as_deref(), Some("categories"));
    assert_eq!(t.indexes[0].name, "idx_category_id");
}

#[test]
fn short_shortens_the_index_name() {
    let (schema, diags) = compile(concat!(
        "nouns {\n  organization short=org \"A company\"\n}\n",
        "naming {\n  index = \"idx_${short(table)}_${columns}\"\n}\n",
        "table organization {\n  column id type=serial\n  pk id\n  index id\n}\n",
    ));
    assert_eq!(errors(&diags).len(), 0, "{:?}", errors(&diags));
    let t = schema.table("organization").expect("organization");
    assert_eq!(t.indexes[0].name, "idx_org_id");
}

#[test]
fn table_names_default_to_the_singular() {
    let (schema, _) = compile(concat!(
        "nouns {\n  member \"A person\"\n}\n",
        "table member {\n  column id type=serial\n  pk id\n}\n",
    ));
    assert!(schema.table("member").is_some());
    assert!(schema.table("members").is_none());
}

#[test]
fn an_unknown_noun_key_is_reported() {
    let (_, diags) = compile("nouns {\n  member abbrev=mbr \"A person\"\n}\n");
    assert!(
        errors(&diags)
            .iter()
            .any(|e| e.contains("unknown noun key `abbrev`")),
        "{:?}",
        errors(&diags)
    );
}

#[test]
fn a_relation_to_a_noun_without_a_table_names_the_identifier() {
    let (_, diags) = compile(concat!(
        "nouns {\n  member \"A person\"\n  badge \"A label\"\n}\n",
        "table member {\n  column id type=serial\n  pk id\n  belongs_to badge\n}\n",
    ));
    assert_eq!(
        errors(&diags),
        vec!["`badge` has no table; it needs `table badge { }`"]
    );
}

#[test]
fn a_relation_to_an_unregistered_noun_says_so() {
    let (_, diags) = compile(concat!(
        "nouns {\n  member \"A person\"\n}\n",
        "table member {\n  column id type=serial\n  pk id\n  belongs_to badge\n}\n",
    ));
    assert!(
        errors(&diags).contains(&"`badge` is not in `nouns`"),
        "{:?}",
        errors(&diags)
    );
}

#[test]
fn warns_when_a_generated_identifier_exceeds_the_dialect_limit() {
    let long = "a".repeat(60);
    let (_, diags) = compile(&format!(
        "nouns {{\n  {long} \"A very long noun\"\n}}\n\
         table {long} {{\n  column id type=serial\n  pk id\n}}\n",
    ));
    assert!(
        warnings(&diags)
            .iter()
            .any(|w| w.contains("primary key name") && w.contains("limit of 63")),
        "{:?}",
        warnings(&diags)
    );
}
