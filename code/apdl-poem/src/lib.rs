//! APDL 协议对象与实体映射模块
//!
//! 实现协议语法单元的定义、组装和解析功能

pub mod dsl;
pub mod protocol_unit;
pub mod standard_units;

// 导出主要类型
pub use apdl_core::ProtocolUnit; // 修正：直接从apdl_core导入
pub use dsl::parser::DslParserImpl;
pub use standard_units::field_unit::FieldUnit;
pub use standard_units::frame_assembler::FrameAssembler;
