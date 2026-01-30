//! 标准语法单元模块
//!
//! 实现字段级语法单元，支持通过DSL定义协议结构

pub mod connector;
pub mod field_unit;
pub mod frame_assembler;

pub use field_unit::FieldUnit;
pub use frame_assembler::FrameAssembler;
