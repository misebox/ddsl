use std::io::{Read, Write};
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use nounsql_core::{Diagnostic, Severity, codegen, diag, dialect, parse, resolve};

#[derive(Parser)]
#[command(
    name = "nounsql",
    version,
    about = "A DSL compiler for database schema design"
)]
struct Cli {
    #[command(subcommand)]
    command: Command,

    /// Output target
    #[arg(long, global = true, default_value = "postgres")]
    dialect: String,

    /// Write to a file instead of stdout
    #[arg(short, long, global = true, value_name = "PATH")]
    output: Option<PathBuf>,

    /// Exit non-zero if anything was warned about
    #[arg(long, global = true)]
    deny_warnings: bool,
}

#[derive(Subcommand)]
enum Command {
    /// Parse and resolve only, printing the diagnostics
    Check { input: PathBuf },
    /// Print the syntax tree
    Ast { input: PathBuf },
    /// Print the resolved schema
    Ir {
        input: PathBuf,
        /// Print it as JSON, for handing to another code generator
        #[arg(long)]
        json: bool,
    },
    /// Print the DDL
    Sql { input: PathBuf },
}

impl Command {
    fn input(&self) -> &Path {
        match self {
            Command::Check { input }
            | Command::Ast { input }
            | Command::Ir { input, .. }
            | Command::Sql { input } => input,
        }
    }
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    let Some(dialect) = dialect::by_name(&cli.dialect) else {
        anyhow::bail!(
            "unknown dialect `{}`; expected {}",
            cli.dialect,
            dialect::names().join(" / ")
        );
    };

    let input = cli.command.input();
    let (src, path) = read_input(input)?;

    let (doc, mut diags) = parse(&src);
    let mut out = String::new();

    if count(&diags, Severity::Error) == 0 {
        let (schema, mut resolved) = resolve(&doc, dialect);
        let has_error = count(&resolved, Severity::Error) > 0;
        diags.append(&mut resolved);

        match &cli.command {
            Command::Ast { .. } => out = format!("{doc:#?}\n"),
            Command::Ir { json, .. } => {
                out = if *json {
                    let mut text = serde_json::to_string_pretty(&schema)
                        .context("could not encode the schema as JSON")?;
                    text.push('\n');
                    text
                } else {
                    format!("{schema:#?}\n")
                }
            }
            Command::Sql { .. } if !has_error => out = codegen::emit(dialect, &schema),
            Command::Check { .. } if !has_error => {
                out = format!(
                    "ok: {} tables, {} columns\n",
                    schema.tables.len(),
                    schema.tables.iter().map(|t| t.columns.len()).sum::<usize>()
                )
            }
            _ => {}
        }
    } else if let Command::Ast { .. } = cli.command {
        // 構文エラーがあっても、そこまで組み立てた木は見せる。
        out = format!("{doc:#?}\n");
    }

    if !diags.is_empty() {
        eprint!("{}", diag::render(&src, &path, &diags));
    }
    if !out.is_empty() {
        write_output(cli.output.as_deref(), &out)?;
    }

    let errors = count(&diags, Severity::Error);
    let warnings = count(&diags, Severity::Warning);
    if errors > 0 || (cli.deny_warnings && warnings > 0) {
        if errors == 0 {
            let s = if warnings == 1 { "" } else { "s" };
            eprintln!("error: {warnings} warning{s} (--deny-warnings)");
        }
        std::process::exit(1);
    }
    Ok(())
}

fn count(diags: &[Diagnostic], severity: Severity) -> usize {
    diags.iter().filter(|d| d.severity == severity).count()
}

/// `-` なら標準入力から読む。
fn read_input(path: &Path) -> Result<(String, String)> {
    if path == Path::new("-") {
        let mut src = String::new();
        std::io::stdin()
            .read_to_string(&mut src)
            .context("could not read standard input")?;
        return Ok((src, "<stdin>".into()));
    }
    let name = path.display().to_string();
    let src = std::fs::read_to_string(path).with_context(|| format!("could not read {name}"))?;
    Ok((src, name))
}

fn write_output(path: Option<&Path>, text: &str) -> Result<()> {
    match path {
        Some(path) => {
            if let Some(dir) = path.parent().filter(|d| !d.as_os_str().is_empty()) {
                std::fs::create_dir_all(dir)
                    .with_context(|| format!("could not create {}", dir.display()))?;
            }
            std::fs::write(path, text)
                .with_context(|| format!("could not write {}", path.display()))
        }
        None => {
            let mut stdout = std::io::stdout().lock();
            stdout.write_all(text.as_bytes())?;
            stdout.flush().map_err(Into::into)
        }
    }
}
