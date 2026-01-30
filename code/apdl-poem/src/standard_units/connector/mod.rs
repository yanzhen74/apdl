//! 连接器模块
//!
//! 实现独立的字段映射机制（连接器模式），用于在不同包之间建立字段映射关系

mod connector_engine;
mod field_mapper;

pub use connector_engine::*;
pub use field_mapper::*;
