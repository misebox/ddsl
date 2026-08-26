pub mod pg;

use crate::dialect::Dialect;
use crate::ir::Schema;

/// 出力ターゲットごとの DDL 生成に振り分ける。
pub fn emit(dialect: Dialect, schema: &Schema) -> String {
    pg::emit(dialect, schema)
}
