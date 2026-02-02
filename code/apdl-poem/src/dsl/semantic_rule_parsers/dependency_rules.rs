//! 依赖规则解析器
//!
//! 处理依赖相关的语义规则解析

use apdl_core::SemanticRule;

/// 解析依赖关系规则
pub fn parse_dependency(params: &str) -> Result<SemanticRule, String> {
    // 解析依赖关系，例如 "fieldA depends_on fieldB" 或 "field: fieldA depends_on fieldB"
    let params = params.trim();
    let parts: Vec<&str> = params.split(" depends_on ").collect();
    if parts.len() == 2 {
        let dependent_field = parts[0].trim();
        let dependent_field = if dependent_field.starts_with("field: ") {
            dependent_field[6..].trim() // 跳过 "field: " 前缀
        } else {
            dependent_field
        };

        Ok(SemanticRule::Dependency {
            dependent_field: dependent_field.to_string(),
            dependency_field: parts[1].trim().to_string(),
        })
    } else {
        Err("Invalid dependency format, expected 'fieldA depends_on fieldB'".to_string())
    }
}
