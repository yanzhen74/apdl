//! APDL Core Library
//!
//! This crate provides the core abstractions and data structures for the
//! APDL (APDS Protocol Definition Language) system.

pub mod error;
pub mod protocol_meta;
pub mod utils;

use std::collections::HashMap;

// 导出错误类型
pub use error::ProtocolError;

// 导出协议元数据类型，便于其他模块使用
pub use protocol_meta::*;

/// 语法单元基础接口 - 实现平台与语法单元的分离
pub trait ProtocolUnit: Send + Sync {
    /// 获取单元元数据
    fn get_meta(&self) -> &protocol_meta::UnitMeta;

    /// 扱包：将SDU（服务数据单元）封装为PDU（协议数据单元）
    fn pack(&self, sdu: &[u8]) -> Result<Vec<u8>, error::ProtocolError>;

    /// 拆包：将PDU解析为SDU
    fn unpack<'a>(&self, pdu: &'a [u8]) -> Result<(Vec<u8>, &'a [u8]), error::ProtocolError>;

    /// 单元合法性校验
    fn validate(&self) -> Result<(), error::ProtocolError>;

    /// 获取单元配置参数
    fn get_params(&self) -> &HashMap<String, String>;

    /// 设置单元配置参数
    fn set_param(&mut self, key: &str, value: &str) -> Result<(), error::ProtocolError>;

    /// 获取单元类型标识
    fn get_unit_type(&self) -> &str;
}

/// DSL解析器接口
pub trait DslParser {
    fn parse_dsl(
        &self,
        dsl_text: &str,
    ) -> Result<protocol_meta::SyntaxUnit, protocol_meta::DslParseError>;
    fn validate_dsl(&self, dsl_text: &str) -> Result<(), protocol_meta::DslValidateError>;
}
