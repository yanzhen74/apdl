//! Frame Assembler 模块
//!
//! 将 Frame Assembler 的功能拆分为多个子模块以提高可维护性

pub mod checksum_rule_handler;
pub mod core;
pub mod custom_algorithm_handler;
pub mod dependency_rule_handler;
pub mod length_rule_handler;
pub mod order_rule_handler;
pub mod pointer_rule_handler;

// 导出主要的结构和公共接口
pub use core::FrameAssembler;
