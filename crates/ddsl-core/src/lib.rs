pub mod ast;
pub mod codegen;
pub mod config;
pub mod diag;
pub mod dialect;
pub mod dict;
pub mod ir;
pub mod lexer;
pub mod parser;
pub mod resolve;
pub mod span;
pub mod template;

pub use diag::{Diagnostic, Severity};
pub use dialect::Dialect;
pub use parser::parse;
pub use resolve::resolve;
