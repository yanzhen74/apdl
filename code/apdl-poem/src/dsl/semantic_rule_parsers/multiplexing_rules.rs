//! 多路复用规则解析器
//!
//! 处理多路复用相关的语义规则解析

use apdl_core::SemanticRule;

/// 解析多路复用规则
pub fn parse_multiplexing(params: &str) -> Result<SemanticRule, String> {
    // 解析多路复用规则
    let params = params.trim();
    let mut field_name = String::new();
    let mut condition = String::new();
    let mut route_target = String::new();
    let mut description = String::new();

    if params.contains("field:") && params.contains("condition:") && params.contains("route_to:") {
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

        if let Some(route_start) = params.find("route_to:") {
            let remaining = &params[route_start + 9..];
            if let Some(semi_pos) = remaining.find(';').map(|p| p + route_start + 9) {
                route_target = remaining[..semi_pos - (route_start + 9)].trim().to_string();
            } else {
                route_target = remaining.trim().to_string();
            }
        }

        if let Some(desc_start) = params.find("desc:") {
            description = params[desc_start + 5..].trim().to_string();
        }
    }

    Ok(SemanticRule::Multiplexing {
        field_name,
        condition,
        route_target,
        description,
    })
}
