use std::path::PathBuf;

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use ddsl_core::{Severity, codegen, diag, dialect, parse, resolve};

#[derive(Parser)]
#[command(name = "ddsl", about = "DB設計用DSLコンパイラ")]
struct Cli {
    #[command(subcommand)]
    command: Command,

    /// 出力ターゲット
    #[arg(long, global = true, default_value = "postgres")]
    dialect: String,
}

#[derive(Subcommand)]
enum Command {
    /// 解析と検証だけ行い、診断を表示する
    Check { input: PathBuf },
    /// 構文木をデバッグ出力する
    Ast { input: PathBuf },
    /// 解決済みスキーマをデバッグ出力する
    Ir { input: PathBuf },
    /// PostgreSQL の DDL を出力する
    Build { input: PathBuf },
}

impl Command {
    fn input(&self) -> &PathBuf {
        match self {
            Command::Check { input }
            | Command::Ast { input }
            | Command::Ir { input }
            | Command::Build { input } => input,
        }
    }
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    let Some(dialect) = dialect::by_name(&cli.dialect) else {
        anyhow::bail!(
            "知らない dialect `{}`。使えるのは {}",
            cli.dialect,
            dialect::names().join(" / ")
        );
    };
    let input = cli.command.input();
    let path = input.display().to_string();
    let src = std::fs::read_to_string(input).with_context(|| format!("読めない: {path}"))?;

    let (doc, mut diags) = parse(&src);
    let parse_errors = count_errors(&diags);

    if let Command::Ast { .. } = cli.command {
        report(&src, &path, &diags);
        println!("{doc:#?}");
        return finish(parse_errors);
    }

    if parse_errors > 0 {
        report(&src, &path, &diags);
        return finish(parse_errors);
    }

    let (schema, mut resolve_diags) = resolve(&doc, dialect);
    diags.append(&mut resolve_diags);
    let errors = count_errors(&diags);
    report(&src, &path, &diags);

    match cli.command {
        Command::Ir { .. } => println!("{schema:#?}"),
        Command::Build { .. } if errors == 0 => print!("{}", codegen::emit(dialect, &schema)),
        Command::Check { .. } if errors == 0 => println!(
            "ok: {} tables, {} columns",
            schema.tables.len(),
            schema.tables.iter().map(|t| t.columns.len()).sum::<usize>()
        ),
        _ => {}
    }
    finish(errors)
}

fn count_errors(diags: &[ddsl_core::Diagnostic]) -> usize {
    diags
        .iter()
        .filter(|d| d.severity == Severity::Error)
        .count()
}

fn report(src: &str, path: &str, diags: &[ddsl_core::Diagnostic]) {
    if !diags.is_empty() {
        eprint!("{}", diag::render(src, path, diags));
    }
}

fn finish(errors: usize) -> Result<()> {
    if errors > 0 {
        std::process::exit(1);
    }
    Ok(())
}
