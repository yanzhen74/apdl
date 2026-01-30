//! 安全规则解析器
//!
//! 处理安全相关的语义规则解析

use apdl_core::SemanticRule;

/// 解析安全规则
pub fn parse_security(params: &str) -> Result<SemanticRule, String> {
    // 解析安全规则
    let params = params.trim();
    let mut field_name = String::new();
    let mut algorithm = String::new();
    let mut description = String::new();

    if params.contains("field:") && params.contains("algorithm:") {
        if let Some(field_start) = params.find("field:") {
            if let Some(semi_pos) = params[field_start..].find(';').map(|p| p + field_start) {
                field_name = params[field_start + 6..semi_pos].trim().to_string();
            } else {
                let remaining = &params[field_start + 6..];
                if let Some(next_semicolon) = remaining.find(';') {
                    field_name = remaining[..next_semicolon].trim().to_string();
                } else {
                    field_name = remaining.trim().to_string();
                }
            }
        }

        if let Some(alg_start) = params.find("algorithm:") {
            let remaining = &params[alg_start + 10..];
            if let Some(semi_pos) = remaining.find(';').map(|p| p + alg_start + 10) {
                algorithm = remaining[..semi_pos - (alg_start + 10)].trim().to_string();
            } else {
                algorithm = remaining.trim().to_string();
            }
        }

        if let Some(desc_start) = params.find("desc:") {
            description = params[desc_start + 5..].trim().to_string();
        }
    }

    Ok(SemanticRule::Security {
        field_name,
        algorithm,
        description,
    })
}

/// 解析冗余规则
pub fn parse_redundancy(params: &str) -> Result<SemanticRule, String> {
    // 解析冗余规则
    let params = params.trim();
    let mut field_name = String::new();
    let mut algorithm = String::new();
    let mut description = String::new();

    if params.contains("field:") && params.contains("algorithm:") {
        if let Some(field_start) = params.find("field:") {
            if let Some(semi_pos) = params[field_start..].find(';').map(|p| p + field_start) {
                field_name = params[field_start + 6..semi_pos].trim().to_string();
            } else {
                let remaining = &params[field_start + 6..];
                if let Some(next_semicolon) = remaining.find(';') {
                    field_name = remaining[..next_semicolon].trim().to_string();
                } else {
                    field_name = remaining.trim().to_string();
                }
            }
        }

        if let Some(alg_start) = params.find("algorithm:") {
            let remaining = &params[alg_start + 10..];
            if let Some(semi_pos) = remaining.find(';').map(|p| p + alg_start + 10) {
                algorithm = remaining[..semi_pos - (alg_start + 10)].trim().to_string();
            } else {
                algorithm = remaining.trim().to_string();
            }
        }

        if let Some(desc_start) = params.find("desc:") {
            description = params[desc_start + 5..].trim().to_string();
        }
    }

    Ok(SemanticRule::Redundancy {
        field_name,
        algorithm,
        description,
    })
}
