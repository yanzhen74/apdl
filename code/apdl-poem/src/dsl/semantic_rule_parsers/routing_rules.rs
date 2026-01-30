//! 路由规则解析器
//!
//! 处理路由相关的语义规则解析

use apdl_core::SemanticRule;

/// 解析路由分发规则
pub fn parse_routing_dispatch(params: &str) -> Result<SemanticRule, String> {
    // 解析路由分发规则，例如 "field: vcid, apid; algorithm: hash_vcid_apid_to_route; desc: ..."
    let params = params.trim();
    let mut field_name = String::new();
    let mut algorithm = String::new();
    let mut description = String::new();

    // 简单解析，查找关键部分
    if params.contains("field:") && params.contains("algorithm:") {
        if let Some(field_start) = params.find("field:") {
            if let Some(semi_pos) = params[field_start..].find(';').map(|p| p + field_start) {
                let field_part = &params[field_start + 6..semi_pos].trim();
                field_name = field_part.to_string();
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

    // 解析字段列表
    let fields: Vec<String> = field_name
        .split(',')
        .map(|s| s.trim().to_string())
        .collect();

    Ok(SemanticRule::RoutingDispatch {
        fields,
        algorithm,
        description,
    })
}

/// 解析地址解析规则
pub fn parse_address_resolution(params: &str) -> Result<SemanticRule, String> {
    // 解析地址解析规则
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

    Ok(SemanticRule::AddressResolution {
        field_name,
        algorithm,
        description,
    })
}
