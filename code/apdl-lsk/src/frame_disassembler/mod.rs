//! 帧拆包模块
//!
//! 提供与FrameAssembler对称的帧拆包功能，支持：
//! - bit级字段精确提取
//! - 字段值校验（固定值、范围、约束）
//! - CRC/Checksum验证
//! - 字段到结构化数据的映射

pub mod bit_extractor;
pub mod core;
pub mod field_validator;

pub use bit_extractor::extract_bit_field;
pub use core::FrameDisassembler;
pub use field_validator::FieldValidator;
