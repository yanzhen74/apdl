//! 验证规则解析器
//!
//! 处理验证相关的语义规则解析

use apdl_core::SemanticRule;

/// 解析校验规则
pub fn parse_validation(params: &str) -> Result<SemanticRule, String> {
    // 解析校验规则
    let params = params.trim();
    let mut field_name = String::new();
    let mut algorithm = String::new();
    let mut range_start = String::new();
    let mut range_end = String::new();
    let mut description = String::new();

    if params.contains("field:") && params.contains("algorithm:") && params.contains("range:") {
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

        // 解析 from(...) to(...) 格式的范围
        if let Some(from_start) = params.find("from(") {
            let from_content_start = from_start + 5; // 跳过 "from("
            if let Some(to_pos) = params[from_content_start..].find(") to(") {
                let actual_to_pos = from_content_start + to_pos;
                range_start = params[from_content_start..actual_to_pos].trim().to_string();

                // 查找结束括号的位置
                let remaining = &params[actual_to_pos + 5..]; // 跳过 ") to("
                if let Some(end_bracket_pos) = remaining.find(')') {
                    range_end = remaining[..end_bracket_pos].trim().to_string();
                }
            }
        }

        if let Some(desc_start) = params.find("desc:") {
            description = params[desc_start + 5..].trim().to_string();
            // 移除字符串两端的引号
            description = description.trim_matches('"').to_string();
        }
    }

    Ok(SemanticRule::Validation {
        field_name,
        algorithm,
        range_start,
        range_end,
        description,
    })
}

/// 解析长度验证规则
pub fn parse_length_validation(params: &str) -> Result<SemanticRule, String> {
    // 解析长度验证规则
    let params = params.trim();
    let mut field_name = String::new();
    let mut condition = String::new();
    let mut description = String::new();

    if params.contains("field:") && params.contains("condition:") {
        if let Some(field_start) = params.find("field:") {
            if let Some(semi_pos) = params[field_start..].find(';').map(|p| p + field_start) {
                field_name = params[field_start + 6..semi_pos].trim().to_string();
            }
        }

        if let Some(cond_start) = params.find("condition:") {
            let remaining = &params[cond_start + 10..];
            if let Some(semi_pos) = remaining.find(';').map(|p| p + cond_start + 10) {
                condition = remaining[..semi_pos - (cond_start + 10)].trim().to_string();
            } else {
                condition = remaining.trim().to_string();
            }
        }

        if let Some(desc_start) = params.find("desc:") {
            description = params[desc_start + 5..].trim().to_string();
            // 移除字符串两端的引号
            description = description.trim_matches('"').to_string();
        }
    }

    Ok(SemanticRule::LengthValidation {
        field_name,
        condition,
        description,
    })
}

/// 解析错误检测规则
pub fn parse_error_detection(params: &str) -> Result<SemanticRule, String> {
    // 解析错误检测规则
    let params = params.trim();
    let mut algorithm = String::new();
    let mut description = String::new();

    if params.contains("algorithm:") {
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
            // 移除字符串两端的引号
            description = description.trim_matches('"').to_string();
        }
    } else {
        algorithm = params.to_string();
    }

    Ok(SemanticRule::ErrorDetection {
        algorithm,
        description,
    })
}
