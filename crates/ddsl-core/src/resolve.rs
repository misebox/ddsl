use std::collections::{HashMap, HashSet};

use indexmap::IndexMap;

use crate::ast::{self, Document, Value};
use crate::config::{Config, TableNameStyle, Vars};
use crate::diag::Diagnostic;
use crate::dialect::Dialect;
use crate::dict::{Compound, Dict, Part};
use crate::ir;
use crate::span::{Span, Spanned};
use crate::template::{Seg, Template};

pub fn resolve(doc: &Document, dialect: Dialect) -> (ir::Schema, Vec<Diagnostic>) {
    let mut diags = Vec::new();
    let config = Config::from_document(doc, &mut diags);
    let dict = Dict::from_block(doc.nouns.as_ref());
    let mut r = Resolver {
        doc,
        dialect,
        config,
        dict,
        diags,
        mixins: HashMap::new(),
        blueprints: HashMap::new(),
    };
    let schema = r.run();
    (schema, r.diags)
}

/// 展開待ちのテーブル。名前だけ先に確定させる。
struct Pending<'a> {
    ast: &'a ast::Table,
    name: String,
    noun: Option<Compound>,
    scope: Scope,
    origin: ir::Origin,
}

/// blueprint 内で仮引数と `let` が束縛する名詞。
type Scope = HashMap<String, Compound>;

struct Resolver<'a> {
    doc: &'a Document,
    dialect: Dialect,
    config: Config,
    dict: Dict,
    diags: Vec<Diagnostic>,
    mixins: HashMap<String, &'a ast::Mixin>,
    blueprints: HashMap<String, &'a ast::Blueprint>,
}

impl<'a> Resolver<'a> {
    fn run(&mut self) -> ir::Schema {
        self.index_definitions();
        let pendings = self.collect_pending();

        // 名前解決を先に済ませる。belongs_to が参照先のPKを引けるようにするため。
        let mut schema = ir::Schema::default();
        let mut relations: Vec<Vec<RelationSpec>> = Vec::new();
        let mut reverses: Vec<Vec<ast::Relation>> = Vec::new();
        for p in &pendings {
            let (table, rels, revs) = self.build_table(p);
            schema.tables.push(table);
            relations.push(rels);
            reverses.push(revs);
        }
        self.attach_relations(&pendings, &relations, &mut schema);
        self.expand_associates(&mut schema);
        self.attach_reverses(&pendings, &reverses, &mut schema);
        self.name_indexes(&mut schema);
        schema
    }

    // ---------- 定義の索引 ----------

    fn index_definitions(&mut self) {
        for m in &self.doc.mixins {
            if let Some(prev) = self.mixins.insert(m.name.value.clone(), m) {
                self.diags.push(
                    Diagnostic::error(m.name.span, format!("mixin `{}` が重複", m.name.value))
                        .with_label(prev.name.span, "最初の定義"),
                );
            }
        }
        for b in &self.doc.blueprints {
            if let Some(prev) = self.blueprints.insert(b.name.value.clone(), b) {
                self.diags.push(
                    Diagnostic::error(b.name.span, format!("blueprint `{}` が重複", b.name.value))
                        .with_label(prev.name.span, "最初の定義"),
                );
            }
        }
    }

    // ---------- テーブルの収集 ----------

    fn collect_pending(&mut self) -> Vec<Pending<'a>> {
        let mut pendings = Vec::new();
        for t in &self.doc.tables {
            let word = t.name.value.clone();
            if self.dict.get(&word).is_none() {
                self.diags.push(Diagnostic::warning(
                    t.name.span,
                    format!("`{word}` が `nouns` に無い。規則変化で解決する"),
                ));
            }
            pendings.push(Pending {
                ast: t,
                name: self.table_name_of(&Compound::noun(word.clone())),
                noun: Some(Compound::noun(word)),
                scope: HashMap::new(),
                origin: ir::Origin::Own,
            });
        }
        pendings.extend(self.expand_blueprints());
        pendings
    }

    fn expand_blueprints(&mut self) -> Vec<Pending<'a>> {
        let mut out = Vec::new();
        let applies: Vec<&ast::MacroCall> = self
            .doc
            .macros
            .iter()
            .filter(|m| m.name.value == "apply_blueprint")
            .collect();

        for call in applies {
            if let Some(comment) = &call.comment {
                self.diags.push(Diagnostic::error(
                    comment.span,
                    "`apply_blueprint` に `comment=` は書けない。blueprint 内の `table` に書く",
                ));
            }
            let Some((bp_name, args)) = call.args.split_first() else {
                self.diags.push(Diagnostic::error(
                    call.span,
                    "`apply_blueprint` には blueprint 名が必要",
                ));
                continue;
            };
            let Some(bp) = self.blueprints.get(bp_name.value.as_str()).copied() else {
                self.diags.push(Diagnostic::error(
                    bp_name.span,
                    format!("blueprint `{}` が無い", bp_name.value),
                ));
                continue;
            };
            if bp.params.len() != args.len() {
                self.diags.push(
                    Diagnostic::error(
                        call.span,
                        format!(
                            "blueprint `{}` は引数 {} 個。{} 個渡されている",
                            bp.name.value,
                            bp.params.len(),
                            args.len()
                        ),
                    )
                    .with_label(bp.name.span, "定義"),
                );
                continue;
            }

            let mut scope: Scope = HashMap::new();
            for (param, arg) in bp.params.iter().zip(args) {
                let noun = Compound::noun(arg.value.clone());
                self.check_nouns_registered(&noun, arg.span);
                self.check_shadowing(param);
                scope.insert(param.value.clone(), noun);
            }

            for item in &bp.items {
                match item {
                    ast::BlueprintItem::Let(l) => {
                        self.check_shadowing(&l.name);
                        if let Some(noun) = self.eval_noun(&l.value, &scope) {
                            scope.insert(l.name.value.clone(), noun);
                        }
                    }
                    ast::BlueprintItem::Table(t) => {
                        let Some(noun) = scope.get(&t.name.value).cloned() else {
                            self.diags.push(Diagnostic::error(
                                t.name.span,
                                format!("`{}` は blueprint 内で束縛されていない", t.name.value),
                            ));
                            continue;
                        };
                        out.push(Pending {
                            ast: t,
                            name: self.table_name_of(&noun),
                            noun: Some(noun),
                            scope: scope.clone(),
                            origin: ir::Origin::Blueprint {
                                name: bp.name.value.clone(),
                                def_span: t.name.span,
                                apply_span: call.span,
                            },
                        });
                    }
                }
            }
        }
        out
    }

    fn check_shadowing(&mut self, name: &Spanned<String>) {
        if self.dict.get(&name.value).is_some() {
            self.diags.push(Diagnostic::error(
                name.span,
                format!("`{}` は名詞と衝突している", name.value),
            ));
        }
    }

    /// blueprint スコープを踏まえて識別子が指す名詞を返す。
    fn resolve_noun(&self, ident: &Spanned<String>, scope: &Scope) -> Compound {
        match scope.get(&ident.value) {
            Some(c) => c.clone(),
            None => Compound::noun(ident.value.clone()),
        }
    }

    fn separator(&self) -> String {
        self.config.naming.noun_separator.clone()
    }

    fn singular_of(&self, c: &Compound) -> String {
        c.singular(&self.dict, &self.separator())
    }

    fn plural_of(&self, c: &Compound) -> String {
        c.plural(&self.dict, &self.separator())
    }

    fn table_name_of(&self, c: &Compound) -> String {
        match self.config.naming.table_name {
            TableNameStyle::Plural => self.plural_of(c),
            TableNameStyle::Singular => self.singular_of(c),
        }
    }

    /// `noun(...)` などの値を名詞として評価する。
    fn eval_noun(&mut self, value: &Spanned<Value>, scope: &Scope) -> Option<Compound> {
        match &value.value {
            Value::Ident(name) => {
                Some(self.resolve_noun(&Spanned::new(name.clone(), value.span), scope))
            }
            Value::Str(text) => Some(Compound::literal(text.clone())),
            Value::Call { name, args } if name.value == "noun" => {
                if args.is_empty() {
                    self.diags
                        .push(Diagnostic::error(value.span, "`noun()` に引数が無い"));
                    return None;
                }
                let mut parts = Vec::new();
                for arg in args {
                    let c = self.eval_noun(arg, scope)?;
                    parts.extend(c.parts);
                }
                Some(Compound { parts })
            }
            Value::Call { name, args } if matches!(name.value.as_str(), "singular" | "plural") => {
                let [arg] = args.as_slice() else {
                    self.diags.push(Diagnostic::error(
                        value.span,
                        format!("`{}()` は引数を1つ取る", name.value),
                    ));
                    return None;
                };
                let c = self.eval_noun(arg, scope)?;
                let text = if name.value == "plural" {
                    self.plural_of(&c)
                } else {
                    self.singular_of(&c)
                };
                Some(Compound::literal(text))
            }
            _ => {
                self.diags
                    .push(Diagnostic::error(value.span, "名詞として評価できない"));
                None
            }
        }
    }

    /// 名詞の各要素が辞書にあるか確かめる。
    fn check_nouns_registered(&mut self, c: &Compound, span: Span) {
        let missing: Vec<String> = c
            .nouns()
            .filter(|n| self.dict.get(n).is_none())
            .map(str::to_string)
            .collect();
        for name in missing {
            self.diags.push(Diagnostic::warning(
                span,
                format!("`{name}` が `nouns` に無い。規則変化で解決する"),
            ));
        }
    }

    // ---------- テンプレート ----------

    fn render(&mut self, tpl: &Template, vars: &Vars, span: Span) -> String {
        let sep = self.separator();
        let mut out = String::new();
        for seg in &tpl.segments {
            let (name, rendered) = match seg {
                Seg::Text(t) => {
                    out.push_str(t);
                    continue;
                }
                Seg::Var(v) => (v, vars.get(v.as_str()).map(|c| c.as_written(&sep))),
                Seg::Call { func, arg } => (
                    arg,
                    vars.get(arg.as_str()).map(|c| {
                        if func == "plural" {
                            c.plural(&self.dict, &sep)
                        } else {
                            c.singular(&self.dict, &sep)
                        }
                    }),
                ),
            };
            match rendered {
                Some(text) => out.push_str(&text),
                None => self.diags.push(Diagnostic::error(
                    span,
                    format!("テンプレート変数 `{name}` はここでは使えない"),
                )),
            }
        }
        out
    }

    // ---------- テーブル本体 ----------

    fn build_table(
        &mut self,
        p: &Pending<'a>,
    ) -> (ir::Table, Vec<RelationSpec>, Vec<ast::Relation>) {
        let mut columns: IndexMap<String, ir::Column> = IndexMap::new();
        let mut pk: Vec<String> = Vec::new();
        let mut indexes: Vec<ir::Index> = Vec::new();
        let mut excepts: Vec<Spanned<String>> = Vec::new();
        let mut except_indexes: Vec<Vec<String>> = Vec::new();
        let mut overrides: Vec<&ast::Override> = Vec::new();

        let mut relations: Vec<RelationSpec> = Vec::new();
        let mut reverses: Vec<ast::Relation> = Vec::new();
        let mut seen = Vec::new();
        self.walk_members(
            &p.ast.members,
            &ir::Origin::Own,
            &p.scope,
            &mut seen,
            &mut columns,
            &mut pk,
            &mut indexes,
            &mut excepts,
            &mut except_indexes,
            &mut overrides,
            &mut relations,
            &mut reverses,
        );

        for name in &excepts {
            if columns.shift_remove(&name.value).is_none() {
                self.diags.push(Diagnostic::error(
                    name.span,
                    format!("`{}` は除外できない。定義が無い", name.value),
                ));
            }
            pk.retain(|c| c != &name.value);
        }
        for cols in &except_indexes {
            let before = indexes.len();
            indexes.retain(|i| &i.columns != cols);
            if indexes.len() == before {
                self.diags.push(Diagnostic::error(
                    p.ast.name.span,
                    format!("除外対象の index `[{}]` が無い", cols.join(", ")),
                ));
            }
        }
        for ov in &overrides {
            let Some(col) = columns.get_mut(&ov.name.value) else {
                self.diags.push(Diagnostic::error(
                    ov.name.span,
                    format!("`{}` は上書きできない。定義が無い", ov.name.value),
                ));
                continue;
            };
            let mut errors = Vec::new();
            apply_attrs(col, &ov.attrs, &mut errors);
            self.diags.append(&mut errors);
        }

        // 除外した列を参照している index を落とす。
        indexes.retain(|i| i.columns.iter().all(|c| columns.contains_key(c)));

        for col in columns.values() {
            // 型が空なのは belongs_to が確保した仮の列。参照先の解決後に埋まる。
            if !col.ty.is_empty() && !self.dialect.has_type(&col.ty) {
                self.diags.push(Diagnostic::error(
                    col.span,
                    format!("`{}` は {} の型ではない", col.ty, self.dialect.name),
                ));
            }
            if self.dialect.is_reserved(&col.name) {
                self.diags.push(Diagnostic::warning(
                    col.span,
                    format!("`{}` は {} の予約語", col.name, self.dialect.name),
                ));
            }
        }

        let comment = p.ast.comment.as_ref().map(|c| c.value.clone()).or_else(|| {
            p.noun
                .as_ref()
                .and_then(Compound::as_single_noun)
                .and_then(|n| self.dict.get(n))
                .and_then(|n| n.comment.clone())
        });

        relations.retain(|r| columns.contains_key(&r.column));

        if self.dialect.is_reserved(&p.name) {
            self.diags.push(Diagnostic::warning(
                p.ast.name.span,
                format!("テーブル名 `{}` は {} の予約語", p.name, self.dialect.name),
            ));
        }

        (
            ir::Table {
                name: p.name.clone(),
                noun: p.noun.clone(),
                comment,
                columns,
                pk,
                indexes,
                foreign_keys: Vec::new(),
                reverses: Vec::new(),
                origin: p.origin.clone(),
                span: p.ast.name.span,
            },
            relations,
            reverses,
        )
    }

    #[allow(clippy::too_many_arguments)]
    fn walk_members(
        &mut self,
        members: &'a [Spanned<ast::Member>],
        origin: &ir::Origin,
        scope: &Scope,
        use_stack: &mut Vec<String>,
        columns: &mut IndexMap<String, ir::Column>,
        pk: &mut Vec<String>,
        indexes: &mut Vec<ir::Index>,
        excepts: &mut Vec<Spanned<String>>,
        except_indexes: &mut Vec<Vec<String>>,
        overrides: &mut Vec<&'a ast::Override>,
        relations: &mut Vec<RelationSpec>,
        reverses: &mut Vec<ast::Relation>,
    ) {
        for m in members {
            match &m.value {
                ast::Member::Column(c) => {
                    let mut errors = Vec::new();
                    let col = self.make_column(c, origin.clone(), m.span, &mut errors);
                    self.diags.append(&mut errors);
                    if let Some(col) = col
                        && let Some(prev) = columns.insert(col.name.clone(), col)
                    {
                        {
                            self.diags.push(
                                Diagnostic::error(
                                    m.span,
                                    format!("カラム `{}` が重複している", prev.name),
                                )
                                .with_label(prev.span, "先の定義"),
                            );
                        }
                    }
                }
                ast::Member::Pk(cols) => {
                    *pk = cols.iter().map(|c| c.value.clone()).collect();
                }
                ast::Member::Index(idx) => indexes.push(ir::Index {
                    name: String::new(),
                    columns: idx.columns.iter().map(|c| c.value.clone()).collect(),
                    unique: idx.unique,
                    span: m.span,
                }),
                ast::Member::Use(name) => self.splice_mixin(
                    name,
                    scope,
                    use_stack,
                    columns,
                    pk,
                    indexes,
                    excepts,
                    except_indexes,
                    overrides,
                    relations,
                    reverses,
                ),
                ast::Member::Override(ov) => overrides.push(ov),
                ast::Member::Except(names) => excepts.extend(names.iter().cloned()),
                ast::Member::ExceptIndex(cols) => {
                    except_indexes.push(cols.iter().map(|c| c.value.clone()).collect())
                }
                ast::Member::Relation(rel) if rel.kind.owns_fk() => {
                    let fk_col = match &rel.fk {
                        Some(name) => name.value.clone(),
                        None => {
                            let target = self.resolve_noun(&rel.target, scope);
                            let vars = Vars::from([("table", target)]);
                            self.render(&self.config.naming.foreign_key.clone(), &vars, m.span)
                        }
                    };
                    let placeholder = ir::Column {
                        name: fk_col.clone(),
                        ty: String::new(),
                        null: false,
                        default: None,
                        on_update: None,
                        comment: rel.comment.as_ref().map(|c| c.value.clone()),
                        origin: ir::Origin::Generated {
                            by: rel.kind.keyword().into(),
                        },
                        span: m.span,
                    };
                    if let Some(prev) = columns.insert(fk_col.clone(), placeholder) {
                        self.diags.push(
                            Diagnostic::error(
                                m.span,
                                format!("FK列 `{fk_col}` が既存のカラムと衝突している"),
                            )
                            .with_label(prev.span, "先の定義"),
                        );
                        continue;
                    }
                    relations.push(RelationSpec {
                        target: rel.target.clone(),
                        unique: rel.kind.is_unique(),
                        column: fk_col,
                        alias: rel.alias.clone(),
                        span: m.span,
                    });
                }
                ast::Member::Relation(rel) => reverses.push(rel.clone()),
            }
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn splice_mixin(
        &mut self,
        name: &Spanned<String>,
        scope: &Scope,
        use_stack: &mut Vec<String>,
        columns: &mut IndexMap<String, ir::Column>,
        pk: &mut Vec<String>,
        indexes: &mut Vec<ir::Index>,
        excepts: &mut Vec<Spanned<String>>,
        except_indexes: &mut Vec<Vec<String>>,
        overrides: &mut Vec<&'a ast::Override>,
        relations: &mut Vec<RelationSpec>,
        reverses: &mut Vec<ast::Relation>,
    ) {
        if use_stack.contains(&name.value) {
            let path = use_stack.join(" -> ");
            self.diags.push(Diagnostic::error(
                name.span,
                format!("mixin が循環している: {path} -> {}", name.value),
            ));
            return;
        }
        let Some(mixin): Option<&'a ast::Mixin> = self.mixins.get(name.value.as_str()).copied()
        else {
            self.diags.push(Diagnostic::error(
                name.span,
                format!("mixin `{}` が無い", name.value),
            ));
            return;
        };
        let origin = ir::Origin::Mixin {
            name: mixin.name.value.clone(),
            def_span: mixin.name.span,
        };
        use_stack.push(name.value.clone());
        self.walk_members(
            &mixin.members,
            &origin,
            scope,
            use_stack,
            columns,
            pk,
            indexes,
            excepts,
            except_indexes,
            overrides,
            relations,
            reverses,
        );
        use_stack.pop();
    }

    fn make_column(
        &self,
        c: &ast::Column,
        origin: ir::Origin,
        span: Span,
        errors: &mut Vec<Diagnostic>,
    ) -> Option<ir::Column> {
        let mut col = ir::Column {
            name: c.name.value.clone(),
            ty: String::new(),
            null: self.config.constraints.null_default,
            default: None,
            on_update: None,
            comment: None,
            origin,
            span,
        };
        apply_attrs(&mut col, &c.attrs, errors);
        if col.ty.is_empty() {
            errors.push(Diagnostic::error(
                c.name.span,
                format!("カラム `{}` に `type=` が無い", c.name.value),
            ));
            return None;
        }
        Some(col)
    }

    // ---------- relation ----------

    fn attach_relations(
        &mut self,
        pendings: &[Pending<'a>],
        relations: &[Vec<RelationSpec>],
        schema: &mut ir::Schema,
    ) {
        for (i, specs) in relations.iter().enumerate() {
            let scope = pendings[i].scope.clone();
            let mut resolved = Vec::new();
            for spec in specs {
                if let Some(r) = self.resolve_relation(spec, &scope, schema) {
                    resolved.push(r);
                }
            }
            let Some(table) = schema.tables.get_mut(i) else {
                continue;
            };
            for r in resolved {
                if let Some(col) = table.columns.get_mut(&r.fk.columns[0])
                    && col.ty.is_empty()
                {
                    col.ty = r.ty;
                }
                table.foreign_keys.push(r.fk);
                if let Some(idx) = r.index {
                    table.indexes.push(idx);
                }
            }
        }
        // 型が埋まらなかったFK列は参照先が解決できなかったもの。落とす。
        for table in &mut schema.tables {
            table.columns.retain(|_, c| !c.ty.is_empty());
        }
    }

    fn resolve_relation(
        &mut self,
        spec: &RelationSpec,
        scope: &Scope,
        schema: &ir::Schema,
    ) -> Option<ResolvedRelation> {
        let target = self.resolve_noun(&spec.target, scope);
        let ref_table_name = self.table_name_of(&target);
        let Some(ref_table) = schema.table(&ref_table_name) else {
            self.diags.push(Diagnostic::error(
                spec.target.span,
                format!("参照先テーブル `{ref_table_name}` が無い"),
            ));
            return None;
        };
        if ref_table.pk.len() != 1 {
            self.diags.push(Diagnostic::error(
                spec.target.span,
                format!("`{ref_table_name}` の主キーが単一列でないため参照できない"),
            ));
            return None;
        }
        let ref_col_name = ref_table.pk[0].clone();
        let ty = self
            .dialect
            .fk_type(&ref_table.columns.get(&ref_col_name)?.ty);

        Some(ResolvedRelation {
            ty,
            fk: ir::ForeignKey {
                alias: self.relation_alias(spec, &target, scope),
                columns: vec![spec.column.clone()],
                ref_table: ref_table_name,
                ref_columns: vec![ref_col_name],
                on_delete: self.config.constraints.on_delete_default.clone(),
                on_update: self.config.constraints.on_update_default.clone(),
                span: spec.span,
            },
            index: if spec.unique || self.config.constraints.foreign_key_index {
                Some(ir::Index {
                    name: String::new(),
                    columns: vec![spec.column.clone()],
                    unique: spec.unique,
                    span: spec.span,
                })
            } else {
                None
            },
        })
    }

    fn expand_associates(&mut self, schema: &mut ir::Schema) {
        let calls: Vec<ast::MacroCall> = self
            .doc
            .macros
            .iter()
            .filter(|m| m.name.value == "associate")
            .cloned()
            .collect();

        for call in calls {
            let [a, b] = call.args.as_slice() else {
                self.diags
                    .push(Diagnostic::error(call.span, "`associate` は引数を2つ取る"));
                continue;
            };
            let joined = Compound {
                parts: vec![Part::Noun(a.value.clone()), Part::Noun(b.value.clone())],
            };
            self.check_nouns_registered(&joined, call.span);
            let name = self.table_name_of(&joined);

            let mut columns = IndexMap::new();
            let mut fks = Vec::new();
            let mut ok = true;
            for side in [a, b] {
                let vars = Vars::from([("table", Compound::noun(side.value.clone()))]);
                let col_name =
                    self.render(&self.config.naming.foreign_key.clone(), &vars, call.span);
                let spec = RelationSpec {
                    target: side.clone(),
                    unique: false,
                    column: col_name.clone(),
                    alias: None,
                    span: call.span,
                };
                let Some(r) = self.resolve_relation(&spec, &HashMap::new(), schema) else {
                    ok = false;
                    break;
                };
                columns.insert(
                    col_name.clone(),
                    ir::Column {
                        name: col_name,
                        ty: r.ty,
                        null: false,
                        default: None,
                        on_update: None,
                        comment: None,
                        origin: ir::Origin::Generated {
                            by: "associate".into(),
                        },
                        span: call.span,
                    },
                );
                fks.push(r.fk);
            }
            if !ok {
                continue;
            }
            let pk: Vec<String> = columns.keys().cloned().collect();
            schema.tables.push(ir::Table {
                name,
                noun: None,
                comment: call.comment.as_ref().map(|c| c.value.clone()),
                columns,
                pk,
                indexes: Vec::new(),
                foreign_keys: fks,
                reverses: Vec::new(),
                origin: ir::Origin::Generated {
                    by: "associate".into(),
                },
                span: call.span,
            });
        }
    }

    // ---------- 逆参照 ----------

    /// FKを持つ側の関連名。名詞は単数形にする。
    fn relation_alias(&mut self, spec: &RelationSpec, target: &Compound, scope: &Scope) -> String {
        match &spec.alias {
            Some(value) => {
                let value = value.clone();
                match self.eval_noun(&value, scope) {
                    Some(c) => self.singular_of(&c),
                    None => String::new(),
                }
            }
            None => {
                let vars = Vars::from([("table", target.clone())]);
                self.render(&self.config.naming.belongs_to.clone(), &vars, spec.span)
            }
        }
    }

    fn attach_reverses(
        &mut self,
        pendings: &[Pending<'a>],
        reverses: &[Vec<ast::Relation>],
        schema: &mut ir::Schema,
    ) {
        // 参照先テーブル名 -> そこへ向かうFK
        let mut incoming: HashMap<String, Vec<IncomingFk>> = HashMap::new();
        for t in &schema.tables {
            for fk in &t.foreign_keys {
                let unique = t
                    .indexes
                    .iter()
                    .any(|i| i.unique && i.columns == fk.columns);
                incoming
                    .entry(fk.ref_table.clone())
                    .or_default()
                    .push(IncomingFk {
                        from_table: t.name.clone(),
                        from_noun: t.noun.clone(),
                        columns: fk.columns.clone(),
                        unique,
                    });
            }
        }

        for i in 0..schema.tables.len() {
            let table_name = schema.tables[i].name.clone();
            let candidates = incoming.get(&table_name).cloned().unwrap_or_default();
            let mut used = vec![false; candidates.len()];
            let mut out: Vec<ir::Reverse> = Vec::new();

            let specs: Vec<ast::Relation> = reverses.get(i).cloned().unwrap_or_default();
            let scope = pendings.get(i).map(|p| p.scope.clone()).unwrap_or_default();

            for spec in &specs {
                let from_table = {
                    let noun = self.resolve_noun(&spec.target, &scope);
                    self.table_name_of(&noun)
                };
                let hits: Vec<usize> = candidates
                    .iter()
                    .enumerate()
                    .filter(|(j, c)| !used[*j] && c.from_table == from_table)
                    .filter(|(_, c)| match &spec.via {
                        Some(v) => c.columns == [v.value.clone()],
                        None => true,
                    })
                    .map(|(j, _)| j)
                    .collect();

                let j = match hits.as_slice() {
                    [j] => *j,
                    [] => {
                        self.diags.push(Diagnostic::error(
                            spec.target.span,
                            format!(
                                "`{from_table}` から `{table_name}` へのFKが無いため `{}` を解決できない",
                                spec.kind.keyword()
                            ),
                        ));
                        continue;
                    }
                    _ => {
                        self.diags.push(Diagnostic::error(
                            spec.target.span,
                            format!(
                                "`{from_table}` から `{table_name}` へのFKが複数ある。`via=` で選ぶ"
                            ),
                        ));
                        continue;
                    }
                };
                let c = &candidates[j];
                if c.unique != spec.kind.is_unique() {
                    let want = if c.unique { "has_one" } else { "has_many" };
                    self.diags.push(Diagnostic::error(
                        spec.target.span,
                        format!("この参照は1対1ではないため `{want}` を使う"),
                    ));
                    continue;
                }
                used[j] = true;
                let alias = match &spec.alias {
                    Some(value) => {
                        let value = value.clone();
                        match self.eval_noun(&value, &scope) {
                            Some(n) if spec.kind.is_unique() => self.singular_of(&n),
                            Some(n) => self.plural_of(&n),
                            None => continue,
                        }
                    }
                    None => self.default_reverse_alias(c, spec.target.span),
                };
                out.push(ir::Reverse {
                    alias,
                    from_table: c.from_table.clone(),
                    via: c.columns.clone(),
                    unique: c.unique,
                    span: spec.target.span,
                });
            }

            // 明示されなかったFKにも既定の名前で逆参照を作る。
            let span = schema.tables[i].span;
            for (j, c) in candidates.iter().enumerate() {
                if used[j] {
                    continue;
                }
                let alias = self.default_reverse_alias(c, span);
                out.push(ir::Reverse {
                    alias,
                    from_table: c.from_table.clone(),
                    via: c.columns.clone(),
                    unique: c.unique,
                    span,
                });
            }

            self.check_relation_names(&schema.tables[i], &out);
            schema.tables[i].reverses = out;
        }
    }

    fn default_reverse_alias(&mut self, c: &IncomingFk, span: Span) -> String {
        let base = c
            .from_noun
            .clone()
            .unwrap_or_else(|| Compound::literal(c.from_table.clone()));
        let tpl = if c.unique {
            self.config.naming.has_one.clone()
        } else {
            self.config.naming.has_many.clone()
        };
        let vars = Vars::from([("table", base)]);
        self.render(&tpl, &vars, span)
    }

    /// 関連名がカラム名や他の関連名と衝突していないか。
    fn check_relation_names(&mut self, table: &ir::Table, reverses: &[ir::Reverse]) {
        let mut seen: HashMap<&str, Span> = HashMap::new();
        for fk in &table.foreign_keys {
            seen.insert(fk.alias.as_str(), fk.span);
        }
        for r in reverses {
            if table.columns.contains_key(&r.alias) {
                self.diags.push(Diagnostic::error(
                    r.span,
                    format!("関連名 `{}` が同名のカラムと衝突している", r.alias),
                ));
                continue;
            }
            if let Some(prev) = seen.insert(r.alias.as_str(), r.span) {
                self.diags.push(
                    Diagnostic::error(
                        r.span,
                        format!("関連名 `{}` が重複している。`alias=` で分ける", r.alias),
                    )
                    .with_label(prev, "先の関連"),
                );
            }
        }
    }

    // ---------- index 名 ----------

    fn name_indexes(&mut self, schema: &mut ir::Schema) {
        let sep = self.config.naming.column_separator.clone();
        let idx_tpl = self.config.naming.index.clone();
        let uq_tpl = self.config.naming.unique_index.clone();
        for i in 0..schema.tables.len() {
            let table_name = schema.tables[i].name.clone();
            let specs: Vec<(usize, Vec<String>, bool, Span)> = schema.tables[i]
                .indexes
                .iter()
                .enumerate()
                .map(|(j, idx)| (j, idx.columns.clone(), idx.unique, idx.span))
                .collect();
            for (j, cols, unique, span) in specs {
                let vars = Vars::from([
                    ("table", Compound::literal(table_name.clone())),
                    ("columns", Compound::literal(cols.join(&sep))),
                ]);
                let tpl = if unique { &uq_tpl } else { &idx_tpl };
                let name = self.render(tpl, &vars, span);
                schema.tables[i].indexes[j].name = name;
            }
        }
    }
}

/// `belongs_to` の解決待ち情報。列の位置だけ先に確保しておく。
struct RelationSpec {
    target: Spanned<String>,
    unique: bool,
    column: String,
    alias: Option<Spanned<Value>>,
    span: Span,
}

#[derive(Debug, Clone)]
struct IncomingFk {
    from_table: String,
    from_noun: Option<Compound>,
    columns: Vec<String>,
    unique: bool,
}

struct ResolvedRelation {
    ty: String,
    fk: ir::ForeignKey,
    index: Option<ir::Index>,
}

fn apply_attrs(col: &mut ir::Column, attrs: &[ast::Attr], errors: &mut Vec<Diagnostic>) {
    for attr in attrs {
        let span = attr.value.span;
        match attr.key.value.as_str() {
            "type" => match &attr.value.value {
                Value::Ident(t) => col.ty = t.clone(),
                _ => errors.push(Diagnostic::error(span, "`type=` には型名を書く")),
            },
            "null" => match &attr.value.value {
                Value::Ident(v) if v == "true" => col.null = true,
                Value::Ident(v) if v == "false" => col.null = false,
                _ => errors.push(Diagnostic::error(span, "`null=` は `true` か `false`")),
            },
            "default" => col.default = to_val(&attr.value.value, span, errors),
            "on_update" => col.on_update = to_val(&attr.value.value, span, errors),
            "comment" => match &attr.value.value {
                Value::Str(s) => col.comment = Some(s.clone()),
                _ => errors.push(Diagnostic::error(span, "`comment=` には文字列を書く")),
            },
            _ => {}
        }
    }
}

fn to_val(v: &Value, span: Span, errors: &mut Vec<Diagnostic>) -> Option<ir::Val> {
    match v {
        Value::Eval(e) => Some(ir::Val::Eval(e.clone())),
        Value::Str(s) => Some(ir::Val::Literal(format!("'{}'", s.replace('\'', "''")))),
        Value::Num(n) => Some(ir::Val::Literal(n.clone())),
        Value::Ident(i) => Some(ir::Val::Literal(i.clone())),
        _ => {
            errors.push(Diagnostic::error(span, "値として使えない"));
            None
        }
    }
}

/// 未使用の警告を抑えるためのダミー。将来 validate で使う。
#[allow(dead_code)]
fn _unused(_: &HashSet<String>) {}
