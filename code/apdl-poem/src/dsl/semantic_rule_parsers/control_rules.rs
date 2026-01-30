//! 控制规则解析器
//!
//! 处理控制相关的语义规则解析

use apdl_core::SemanticRule;

/// 解析条件规则
pub fn parse_conditional(params: &str) -> Result<SemanticRule, String> {
    // 解析条件关系，例如 "fieldC if fieldA.value == 0x01"
    let params = params.trim();
    Ok(SemanticRule::Conditional {
        condition: params.to_string(),
    })
}

/// 解析顺序规则
pub fn parse_order(params: &str) -> Result<SemanticRule, String> {
    // 解析字段顺序，例如 "fieldA before fieldB"
    let params = params.trim();
    let parts: Vec<&str> = params.split(" before ").collect();
    if parts.len() == 2 {
        Ok(SemanticRule::Order {
            first_field: parts[0].trim().to_string(),
            second_field: parts[1].trim().to_string(),
        })
    } else {
        Err("Invalid order format, expected 'fieldA before fieldB'".to_string())
    }
}

/// 解析指针规则
pub fn parse_pointer(params: &str) -> Result<SemanticRule, String> {
    // 解析指针语义，例如 "pointer_field points_to target_field"
    let params = params.trim();
    let parts: Vec<&str> = params.split(" points_to ").collect();
    if parts.len() == 2 {
        Ok(SemanticRule::Pointer {
            pointer_field: parts[0].trim().to_string(),
            target_field: parts[1].trim().to_string(),
        })
    } else {
        Err("Invalid pointer format, expected 'pointer_field points_to target_field'".to_string())
    }
}

/// 解析算法规则
pub fn parse_algorithm(params: &str) -> Result<SemanticRule, String> {
    // 解析自定义算法，例如 "field_name uses custom_algorithm"
    let params = params.trim();
    let parts: Vec<&str> = params.split(" uses ").collect();
    if parts.len() == 2 {
        Ok(SemanticRule::Algorithm {
            field_name: parts[0].trim().to_string(),
            algorithm: parts[1].trim().to_string(),
        })
    } else {
        Err("Invalid algorithm format, expected 'field_name uses custom_algorithm'".to_string())
    }
}

/// 解析长度规则
pub fn parse_length_rule(params: &str) -> Result<SemanticRule, String> {
    // 解析长度规则，例如 "field_name equals expression"
    let params = params.trim();
    let parts: Vec<&str> = params.split(" equals ").collect();
    if parts.len() == 2 {
        let field_name = parts[0].trim().to_string();
        let expression = parts[1].trim().to_string();
        Ok(SemanticRule::LengthRule {
            field_name,
            expression,
        })
    } else {
        Err("Invalid length rule format, expected 'field_name equals expression'".to_string())
    }
}

/// 解析序列控制规则
pub fn parse_sequence_control(params: &str) -> Result<SemanticRule, String> {
    // 解析序列控制规则
    let params = params.trim();
    let mut field_name = String::new();
    let mut trigger_condition = String::new();
    let mut algorithm = String::new();
    let mut description = String::new();

    if params.contains("field:") && params.contains("trigger:") && params.contains("algorithm:") {
        if let Some(field_start) = params.find("field:") {
            if let Some(semi_pos) = params[field_start..].find(';').map(|p| p + field_start) {
                field_name = params[field_start + 6..semi_pos].trim().to_string();
            }
        }

        if let Some(trigger_start) = params.find("trigger:") {
            let remaining = &params[trigger_start + 8..];
            if let Some(semi_pos) = remaining.find(';').map(|p| p + trigger_start + 8) {
                trigger_condition = remaining[..semi_pos - (trigger_start + 8)]
                    .trim()
                    .to_string();
            } else {
                trigger_condition = remaining.trim().to_string();
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

    Ok(SemanticRule::SequenceControl {
        field_name,
        trigger_condition,
        algorithm,
        description,
    })
}

/// 解析优先级处理规则
pub fn parse_priority_processing(params: &str) -> Result<SemanticRule, String> {
    // 解析优先级处理规则
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

    Ok(SemanticRule::PriorityProcessing {
        field_name,
        algorithm,
        description,
    })
}

/// 解析状态机规则
pub fn parse_state_machine(params: &str) -> Result<SemanticRule, String> {
    // 解析状态机规则
    let params = params.trim();
    let mut condition = String::new();
    let mut algorithm = String::new();
    let mut description = String::new();

    if params.contains("condition:") && params.contains("algorithm:") {
        if let Some(cond_start) = params.find("condition:") {
            if let Some(semi_pos) = params[cond_start..].find(';').map(|p| p + cond_start) {
                condition = params[cond_start + 10..semi_pos].trim().to_string();
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

    Ok(SemanticRule::StateMachine {
        condition,
        algorithm,
        description,
    })
}

/// 解析周期性传输规则
pub fn parse_periodic_transmission(params: &str) -> Result<SemanticRule, String> {
    // 解析周期性传输规则
    let params = params.trim();
    let mut field_name = String::new();
    let mut condition = String::new();
    let mut algorithm = String::new();
    let mut description = String::new();

    if params.contains("field:") && params.contains("condition:") && params.contains("algorithm:") {
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

    Ok(SemanticRule::PeriodicTransmission {
        field_name,
        condition,
        algorithm,
        description,
    })
}

/// 解析消息过滤规则
pub fn parse_message_filtering(params: &str) -> Result<SemanticRule, String> {
    // 解析消息过滤规则
    let params = params.trim();
    let mut condition = String::new();
    let mut action = String::new();
    let mut description = String::new();

    if params.contains("condition:") && params.contains("action:") {
        if let Some(cond_start) = params.find("condition:") {
            if let Some(semi_pos) = params[cond_start..].find(';').map(|p| p + cond_start) {
                condition = params[cond_start + 10..semi_pos].trim().to_string();
            }
        }

        if let Some(action_start) = params.find("action:") {
            let remaining = &params[action_start + 7..];
            if let Some(semi_pos) = remaining.find(';').map(|p| p + action_start + 7) {
                action = remaining[..semi_pos - (action_start + 7)]
                    .trim()
                    .to_string();
            } else {
                action = remaining.trim().to_string();
            }
        }

        if let Some(desc_start) = params.find("desc:") {
            description = params[desc_start + 5..].trim().to_string();
        }
    }

    Ok(SemanticRule::MessageFiltering {
        condition,
        action,
        description,
    })
}

/// 解析序列重置规则
pub fn parse_sequence_reset(params: &str) -> Result<SemanticRule, String> {
    // 解析序列重置规则
    let params = params.trim();
    let mut field_name = String::new();
    let mut condition = String::new();
    let mut action = String::new();
    let mut description = String::new();

    if params.contains("field:") && params.contains("condition:") && params.contains("action:") {
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

        if let Some(action_start) = params.find("action:") {
            let remaining = &params[action_start + 7..];
            if let Some(semi_pos) = remaining.find(';').map(|p| p + action_start + 7) {
                action = remaining[..semi_pos - (action_start + 7)]
                    .trim()
                    .to_string();
            } else {
                action = remaining.trim().to_string();
            }
        }

        if let Some(desc_start) = params.find("desc:") {
            description = params[desc_start + 5..].trim().to_string();
        }
    }

    Ok(SemanticRule::Conditional {
        condition: format!("{} {} {}", field_name, condition, action),
    })
}

/// 解析时间戳插入规则
pub fn parse_timestamp_insertion(params: &str) -> Result<SemanticRule, String> {
    // 解析时间戳插入规则
    let params = params.trim();
    let mut condition = String::new();
    let mut field_name = String::new();
    let mut algorithm = String::new();
    let mut description = String::new();

    if params.contains("condition:") && params.contains("field:") && params.contains("algorithm:") {
        if let Some(cond_start) = params.find("condition:") {
            if let Some(semi_pos) = params[cond_start..].find(';').map(|p| p + cond_start) {
                condition = params[cond_start + 10..semi_pos].trim().to_string();
            }
        }

        if let Some(field_start) = params.find("field:") {
            let remaining = &params[field_start + 6..];
            if let Some(semi_pos) = remaining.find(';').map(|p| p + field_start + 6) {
                field_name = remaining[..semi_pos - (field_start + 6)].trim().to_string();
            } else {
                field_name = remaining.trim().to_string();
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

    Ok(SemanticRule::Conditional {
        condition: format!("{} on {} with {}", condition, field_name, algorithm),
    })
}

/// 解析流量控制规则
pub fn parse_flow_control(params: &str) -> Result<SemanticRule, String> {
    // 解析流量控制规则
    let params = params.trim();
    let mut field_name = String::new();
    let mut algorithm = String::new();
    let mut description = String::new();

    if params.contains("field:") && params.contains("algorithm:") {
        if let Some(field_start) = params.find("field:") {
            if let Some(semi_pos) = params[field_start..].find(';').map(|p| p + field_start) {
                field_name = params[field_start + 6..semi_pos].trim().to_string();
            } else {
                // 尝试解析没有分号的情况
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

    Ok(SemanticRule::FlowControl {
        field_name,
        algorithm,
        description,
    })
}
