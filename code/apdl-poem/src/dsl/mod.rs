//! DSL解析模块
//!
//! 实现APDL的领域特定语言解析功能

pub mod parser;

pub use apdl_core::{
    AlgorithmAst, ChecksumAlgorithm, CoverDesc, LengthDesc, LengthUnit, ScopeDesc, SemanticRule,
    SyntaxUnit, UnitType,
};
pub use parser::DslParserImpl;
