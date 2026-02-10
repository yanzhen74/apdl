//! 分层拆包引擎
//!
//! 自动识别协议层级关系，递归拆包直到应用数据层

pub mod core;
pub mod layer_data;

pub use core::LayeredDisassembler;
pub use layer_data::{DisassembleResult, LayerData, ValidationError};
