//! 同步规则解析器
//!
//! 处理同步相关的语义规则解析

use apdl_core::SemanticRule;

/// 解析同步规则
pub fn parse_synchronization(params: &str) -> Result<SemanticRule, String> {
    // 解析同步规则
    let params = params.trim();
    let mut field_name = String::new();
    let mut algorithm = String::new();
    let mut description = String::new();

    if params.contains("field:") && params.contains("algorithm:") {
        if let Some(field_start) = params.find("field:") {
            if let Some(semi_pos) = params[field_start..].find(';').map(|p| p + field_start) {
                field_name = params[field_start + 6..semi_pos].trim().to_string();
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

    Ok(SemanticRule::Synchronization {
        field_name,
        algorithm,
        description,
    })
}

/// 解析嵌套同步规则
pub fn parse_nested_sync(params: &str) -> Result<SemanticRule, String> {
    // 解析嵌套同步规则
    let params = params.trim();
    let mut field_name = String::new();
    let mut target = String::new();
    let mut _algorithm = String::new();
    let mut _description = String::new();

    if params.contains("field:") && params.contains("target:") && params.contains("algorithm:") {
        if let Some(field_start) = params.find("field:") {
            if let Some(semi_pos) = params[field_start..].find(';').map(|p| p + field_start) {
                field_name = params[field_start + 6..semi_pos].trim().to_string();
            }
        }

        if let Some(target_start) = params.find("target:") {
            let remaining = &params[target_start + 7..];
            if let Some(semi_pos) = remaining.find(';').map(|p| p + target_start + 7) {
                target = remaining[..semi_pos - (target_start + 7)]
                    .trim()
                    .to_string();
            } else {
                target = remaining.trim().to_string();
            }
        }

        if let Some(alg_start) = params.find("algorithm:") {
            let remaining = &params[alg_start + 10..];
            if let Some(semi_pos) = remaining.find(';').map(|p| p + alg_start + 10) {
                let _ = remaining[..semi_pos - (alg_start + 10)].trim().to_string();
            } else {
                let _ = remaining.trim().to_string();
            }
        }

        if let Some(desc_start) = params.find("desc:") {
            let _ = params[desc_start + 5..].trim().to_string();
        }
    }

    Ok(SemanticRule::Pointer {
        pointer_field: field_name,
        target_field: target,
    })
}

/// 解析时间同步规则
pub fn parse_time_synchronization(params: &str) -> Result<SemanticRule, String> {
    // 解析时间同步规则
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

    Ok(SemanticRule::TimeSynchronization {
        field_name,
        algorithm,
        description,
    })
}
