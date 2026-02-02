//! 协议错误定义

use std::fmt;

#[derive(Debug, Clone, PartialEq)]
pub enum ProtocolError {
    /// 字段未找到
    FieldNotFound(String),
    /// 无效的帧格式
    InvalidFrameFormat(String),
    /// 无效的字段定义
    InvalidFieldDefinition(String),
    /// 解析错误
    ParseError(String),
    /// 验证错误
    ValidationError(String),
    /// 长度错误
    LengthError(String),
    /// 校验错误
    ChecksumError(String),
    /// 依赖关系错误
    DependencyError(String),
    /// 无效的表达式
    InvalidExpression(String),
    /// 同步错误
    SynchronizationError(String),
    /// 其他错误
    Other(String),
}

impl fmt::Display for ProtocolError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ProtocolError::FieldNotFound(msg) => write!(f, "Field not found: {msg}"),
            ProtocolError::InvalidFrameFormat(msg) => write!(f, "Invalid frame format: {msg}"),
            ProtocolError::InvalidFieldDefinition(msg) => {
                write!(f, "Invalid field definition: {msg}")
            }
            ProtocolError::ParseError(msg) => write!(f, "Parse error: {msg}"),
            ProtocolError::ValidationError(msg) => write!(f, "Validation error: {msg}"),
            ProtocolError::LengthError(msg) => write!(f, "Length error: {msg}"),
            ProtocolError::ChecksumError(msg) => write!(f, "Checksum error: {msg}"),
            ProtocolError::DependencyError(msg) => write!(f, "Dependency error: {msg}"),
            ProtocolError::InvalidExpression(msg) => write!(f, "Invalid expression: {msg}"),
            ProtocolError::SynchronizationError(msg) => write!(f, "Synchronization error: {msg}"),
            ProtocolError::Other(msg) => write!(f, "Other error: {msg}"),
        }
    }
}

impl std::error::Error for ProtocolError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        None
    }
}

impl From<String> for ProtocolError {
    fn from(s: String) -> Self {
        ProtocolError::ParseError(s)
    }
}

impl From<&str> for ProtocolError {
    fn from(s: &str) -> Self {
        ProtocolError::ParseError(s.to_string())
    }
}
