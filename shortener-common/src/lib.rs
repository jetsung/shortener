pub mod errors;
/// 共享类型和工具函数
///
/// 这个库包含 shortener-server 和 shortener-cli 共享的类型定义、
/// 常量和工具函数。
pub mod types;

pub use errors::*;
pub use types::*;
