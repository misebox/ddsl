/// `"idx_${table}_${columns}"` のような命名テンプレート。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Seg {
    Text(String),
    Var(String),
    Call { func: String, arg: String },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Template {
    pub segments: Vec<Seg>,
}

impl Template {
    pub fn parse(src: &str) -> Result<Template, String> {
        let mut segments = Vec::new();
        let mut text = String::new();
        let mut rest = src;

        while let Some(i) = rest.find("${") {
            text.push_str(&rest[..i]);
            let after = &rest[i + 2..];
            let Some(close) = after.find('}') else {
                return Err("unterminated `${`; expected `}`".into());
            };
            if !text.is_empty() {
                segments.push(Seg::Text(std::mem::take(&mut text)));
            }
            segments.push(parse_expr(after[..close].trim())?);
            rest = &after[close + 1..];
        }
        text.push_str(rest);
        if !text.is_empty() {
            segments.push(Seg::Text(text));
        }
        Ok(Template { segments })
    }

    /// テンプレートが参照している変数名を列挙する。
    pub fn vars(&self) -> Vec<&str> {
        self.segments
            .iter()
            .filter_map(|s| match s {
                Seg::Text(_) => None,
                Seg::Var(v) => Some(v.as_str()),
                Seg::Call { arg, .. } => Some(arg.as_str()),
            })
            .collect()
    }
}

fn parse_expr(expr: &str) -> Result<Seg, String> {
    match expr.split_once('(') {
        Some((func, rest)) => {
            let arg = rest
                .strip_suffix(')')
                .ok_or_else(|| format!("unterminated `(` in `{expr}`"))?;
            let func = func.trim();
            if !matches!(func, "singular" | "plural" | "short" | "desc") {
                return Err(format!(
                    "unknown template function `{func}`; expected singular / plural / short / desc"
                ));
            }
            Ok(Seg::Call {
                func: func.into(),
                arg: arg.trim().into(),
            })
        }
        None => {
            if expr.is_empty() {
                Err("empty `${}`".into())
            } else {
                Ok(Seg::Var(expr.into()))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_var_and_call() {
        let t = Template::parse("${singular(table)}_id").expect("parse");
        assert_eq!(
            t.segments,
            vec![
                Seg::Call {
                    func: "singular".into(),
                    arg: "table".into()
                },
                Seg::Text("_id".into())
            ]
        );
    }

    #[test]
    fn parses_multiple_vars() {
        let t = Template::parse("idx_${table}_${columns}").expect("parse");
        assert_eq!(t.vars(), vec!["table", "columns"]);
    }

    #[test]
    fn rejects_unknown_function() {
        assert!(Template::parse("${upper(table)}").is_err());
    }
}
