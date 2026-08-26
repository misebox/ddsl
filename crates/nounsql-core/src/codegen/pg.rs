use crate::dialect::Dialect;
use crate::ir::{Schema, Table, Val};

fn quote_in(d: Dialect, name: &str) -> String {
    let plain = !name.is_empty()
        && name
            .chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_')
        && !name.starts_with(|c: char| c.is_ascii_digit());
    if plain && !d.is_reserved(name) {
        name.to_string()
    } else {
        format!("\"{}\"", name.replace('"', "\"\""))
    }
}

fn literal(s: &str) -> String {
    format!("'{}'", s.replace('\'', "''"))
}

fn val(v: &Val) -> String {
    match v {
        Val::Literal(s) => s.clone(),
        Val::Eval(e) => e.clone(),
    }
}

fn action(a: &str) -> &'static str {
    match a {
        "restrict" => "RESTRICT",
        "set_null" => "SET NULL",
        "no_action" => "NO ACTION",
        _ => "CASCADE",
    }
}

pub fn emit(dialect: Dialect, schema: &Schema) -> String {
    let quote = |n: &str| quote_in(dialect, n);
    let mut out = String::new();
    for table in &schema.tables {
        out.push_str(&create_table(dialect, table));
        out.push('\n');
    }
    for table in &schema.tables {
        for idx in &table.indexes {
            let kind = if idx.unique { "UNIQUE INDEX" } else { "INDEX" };
            let cols = idx
                .columns
                .iter()
                .map(|c| quote(c))
                .collect::<Vec<_>>()
                .join(", ");
            out.push_str(&format!(
                "CREATE {kind} {} ON {} ({cols});\n",
                quote(&idx.name),
                quote(&table.name)
            ));
        }
    }
    if schema.tables.iter().any(|t| !t.foreign_keys.is_empty()) {
        out.push('\n');
    }
    for table in &schema.tables {
        for fk in &table.foreign_keys {
            let cols = fk
                .columns
                .iter()
                .map(|c| quote(c))
                .collect::<Vec<_>>()
                .join(", ");
            let refs = fk
                .ref_columns
                .iter()
                .map(|c| quote(c))
                .collect::<Vec<_>>()
                .join(", ");
            let name = format!("{}_{}_fkey", table.name, fk.columns.join("_"));
            out.push_str(&format!(
                "ALTER TABLE {} ADD CONSTRAINT {} FOREIGN KEY ({cols}) REFERENCES {} ({refs}) ON DELETE {} ON UPDATE {};\n",
                quote(&table.name),
                quote(&name),
                quote(&fk.ref_table),
                action(&fk.on_delete),
                action(&fk.on_update),
            ));
        }
    }

    let triggers = emit_on_update_triggers(dialect, schema);
    if !triggers.is_empty() {
        out.push('\n');
        out.push_str(&triggers);
    }
    let comments = emit_comments(dialect, schema);
    if !comments.is_empty() {
        out.push('\n');
        out.push_str(&comments);
    }
    out
}

fn create_table(dialect: Dialect, table: &Table) -> String {
    let quote = |n: &str| quote_in(dialect, n);
    let mut lines: Vec<String> = Vec::new();
    for col in table.columns.values() {
        let mut line = format!("  {} {}", quote(&col.name), col.ty);
        if !col.null {
            line.push_str(" NOT NULL");
        }
        if let Some(d) = &col.default {
            line.push_str(&format!(" DEFAULT {}", val(d)));
        }
        lines.push(line);
    }
    if !table.pk.is_empty() {
        let cols = table
            .pk
            .iter()
            .map(|c| quote(c))
            .collect::<Vec<_>>()
            .join(", ");
        lines.push(format!(
            "  CONSTRAINT {} PRIMARY KEY ({cols})",
            quote(&format!("{}_pkey", table.name))
        ));
    }
    format!(
        "CREATE TABLE {} (\n{}\n);\n",
        quote(&table.name),
        lines.join(",\n")
    )
}

/// `on_update=` は PostgreSQL に対応する構文が無いのでトリガで実現する。
fn emit_on_update_triggers(dialect: Dialect, schema: &Schema) -> String {
    let quote = |n: &str| quote_in(dialect, n);
    let mut out = String::new();
    for table in &schema.tables {
        for col in table.columns.values() {
            let Some(v) = &col.on_update else { continue };
            let fname = format!("{}_{}_on_update", table.name, col.name);
            out.push_str(&format!(
                "CREATE OR REPLACE FUNCTION {}() RETURNS trigger AS $$\nBEGIN\n  NEW.{} := {};\n  RETURN NEW;\nEND;\n$$ LANGUAGE plpgsql;\n",
                quote(&fname),
                quote(&col.name),
                val(v)
            ));
            out.push_str(&format!(
                "CREATE TRIGGER {} BEFORE UPDATE ON {} FOR EACH ROW EXECUTE FUNCTION {}();\n",
                quote(&format!("{fname}_trg")),
                quote(&table.name),
                quote(&fname)
            ));
        }
    }
    out
}

fn emit_comments(dialect: Dialect, schema: &Schema) -> String {
    let quote = |n: &str| quote_in(dialect, n);
    let mut out = String::new();
    for table in &schema.tables {
        if let Some(c) = &table.comment {
            out.push_str(&format!(
                "COMMENT ON TABLE {} IS {};\n",
                quote(&table.name),
                literal(c)
            ));
        }
        for col in table.columns.values() {
            if let Some(c) = &col.comment {
                out.push_str(&format!(
                    "COMMENT ON COLUMN {}.{} IS {};\n",
                    quote(&table.name),
                    quote(&col.name),
                    literal(c)
                ));
            }
        }
    }
    out
}
