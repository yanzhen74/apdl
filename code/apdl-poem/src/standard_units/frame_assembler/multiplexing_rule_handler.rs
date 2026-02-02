//! 多路复用规则处理器
//!
//! 处理多路复用相关的语义规则

use apdl_core::ProtocolError;

use crate::standard_units::frame_assembler::core::FrameAssembler;

impl FrameAssembler {
    /// 应用多路复用规则
    pub fn apply_multiplexing_rule(
        &mut self,
        field_name: &str,
        condition: &str,
        route_target: &str,
        description: &str,
        _frame_data: &mut Vec<u8>,
    ) -> Result<(), ProtocolError> {
        println!(
            "Applying multiplexing rule: {description} for field {field_name} with condition {condition} and route to {route_target}"
        );

        // 获取字段值
        let field_value = if let Ok(value) = self.get_field_value(field_name) {
            value
        } else {
            return Err(ProtocolError::FieldNotFound(format!(
                "Multiplexing field {field_name} not found"
            )));
        };

        // 根据条件判断是否需要进行多路复用处理
        let should_multiplex =
            self.evaluate_multiplexing_condition(field_name, condition, &field_value)?;

        if should_multiplex {
            println!(
                "Multiplexing condition met for field {field_name}: routing to {route_target}"
            );
            // TODO: 在实际应用中，这里可能会根据条件将数据路由到不同的处理路径
            // 在实际应用中，这里可能会根据条件将数据路由到不同的处理路径
            // 当前我们只是记录路由决策
        } else {
            println!(
                "Multiplexing condition not met for field {field_name}: no routing to {route_target}"
            );
        }

        Ok(())
    }

    /// 评估多路复用条件
    fn evaluate_multiplexing_condition(
        &self,
        field_name: &str,
        condition: &str,
        field_value: &[u8],
    ) -> Result<bool, ProtocolError> {
        // 解析条件表达式，例如 "data_type_check" 或具体条件
        match condition {
            "data_type_check" => {
                // 检查数据类型是否为特定值
                // 这里我们简单地检查字段值是否非零
                Ok(!field_value.is_empty() && field_value.iter().any(|&b| b != 0))
            }
            "always_route" => Ok(true),
            "never_route" => Ok(false),
            _ => {
                // 尝试解析更复杂的条件表达式
                self.parse_complex_condition(field_name, condition, field_value)
            }
        }
    }

    /// 解析复杂条件
    fn parse_complex_condition(
        &self,
        field_name: &str,
        condition: &str,
        field_value: &[u8],
    ) -> Result<bool, ProtocolError> {
        // 检查是否包含比较操作符
        if condition.contains("==") {
            self.parse_equality_condition(field_name, condition, field_value)
        } else if condition.contains(">=") {
            self.multiplex_parse_greater_equal_condition(field_name, condition, field_value)
        } else if condition.contains(">") {
            self.multiplex_parse_greater_condition(field_name, condition, field_value)
        } else if condition.contains("<=") {
            self.multiplex_parse_less_equal_condition(field_name, condition, field_value)
        } else if condition.contains("<") {
            self.multiplex_parse_less_condition(field_name, condition, field_value)
        } else if condition.contains("!=") {
            self.multiplex_parse_not_equal_condition(field_name, condition, field_value)
        } else if condition.contains("contains") {
            self.multiplex_parse_contains_condition(field_name, condition, field_value)
        } else {
            // 如果无法解析，假设条件为真
            println!("Unknown condition format '{condition}', defaulting to true");
            Ok(true)
        }
    }

    /// 解析相等条件
    fn parse_equality_condition(
        &self,
        field_name: &str,
        condition: &str,
        field_value: &[u8],
    ) -> Result<bool, ProtocolError> {
        let parts: Vec<&str> = condition.split("==").collect();
        if parts.len() == 2 {
            let field_expr = parts[0].trim();
            let value_expr = parts[1].trim();

            if field_expr == field_name {
                // 解析期望值
                let expected_value = self.multiplex_parse_value_expression(value_expr)?;
                let actual_value = self.bytes_to_u64(field_value);

                Ok(actual_value == expected_value)
            } else {
                // 如果条件中的字段名与当前字段名不匹配，返回true以允许继续处理
                Ok(true)
            }
        } else {
            Ok(false)
        }
    }

    /// 解析大于等于条件
    fn multiplex_parse_greater_equal_condition(
        &self,
        field_name: &str,
        condition: &str,
        field_value: &[u8],
    ) -> Result<bool, ProtocolError> {
        let parts: Vec<&str> = condition.split(">=").collect();
        if parts.len() == 2 {
            let field_expr = parts[0].trim();
            let value_expr = parts[1].trim();

            if field_expr == field_name {
                let expected_value = self.multiplex_parse_value_expression(value_expr)?;
                let actual_value = self.bytes_to_u64(field_value);

                Ok(actual_value >= expected_value)
            } else {
                Ok(true)
            }
        } else {
            Ok(false)
        }
    }

    /// 解析大于条件
    fn multiplex_parse_greater_condition(
        &self,
        field_name: &str,
        condition: &str,
        field_value: &[u8],
    ) -> Result<bool, ProtocolError> {
        let parts: Vec<&str> = condition.split(">").collect();
        if parts.len() == 2 {
            let field_expr = parts[0].trim();
            let value_expr = parts[1].trim();

            if field_expr == field_name {
                let expected_value = self.multiplex_parse_value_expression(value_expr)?;
                let actual_value = self.bytes_to_u64(field_value);

                Ok(actual_value > expected_value)
            } else {
                Ok(true)
            }
        } else {
            Ok(false)
        }
    }

    /// 解析小于等于条件
    fn multiplex_parse_less_equal_condition(
        &self,
        field_name: &str,
        condition: &str,
        field_value: &[u8],
    ) -> Result<bool, ProtocolError> {
        let parts: Vec<&str> = condition.split("<=").collect();
        if parts.len() == 2 {
            let field_expr = parts[0].trim();
            let value_expr = parts[1].trim();

            if field_expr == field_name {
                let expected_value = self.multiplex_parse_value_expression(value_expr)?;
                let actual_value = self.bytes_to_u64(field_value);

                Ok(actual_value <= expected_value)
            } else {
                Ok(true)
            }
        } else {
            Ok(false)
        }
    }

    /// 解析小于条件
    fn multiplex_parse_less_condition(
        &self,
        field_name: &str,
        condition: &str,
        field_value: &[u8],
    ) -> Result<bool, ProtocolError> {
        let parts: Vec<&str> = condition.split("<").collect();
        if parts.len() == 2 {
            let field_expr = parts[0].trim();
            let value_expr = parts[1].trim();

            if field_expr == field_name {
                let expected_value = self.multiplex_parse_value_expression(value_expr)?;
                let actual_value = self.bytes_to_u64(field_value);

                Ok(actual_value < expected_value)
            } else {
                Ok(true)
            }
        } else {
            Ok(false)
        }
    }

    /// 解析不等于条件
    fn multiplex_parse_not_equal_condition(
        &self,
        field_name: &str,
        condition: &str,
        field_value: &[u8],
    ) -> Result<bool, ProtocolError> {
        let parts: Vec<&str> = condition.split("!=").collect();
        if parts.len() == 2 {
            let field_expr = parts[0].trim();
            let value_expr = parts[1].trim();

            if field_expr == field_name {
                let expected_value = self.multiplex_parse_value_expression(value_expr)?;
                let actual_value = self.bytes_to_u64(field_value);

                Ok(actual_value != expected_value)
            } else {
                Ok(true)
            }
        } else {
            Ok(false)
        }
    }

    /// 解析包含条件
    fn multiplex_parse_contains_condition(
        &self,
        _field_name: &str,
        condition: &str,
        field_value: &[u8],
    ) -> Result<bool, ProtocolError> {
        // 暂时实现简单的包含检查
        // 实际应用中可能需要更复杂的模式匹配
        if condition.contains("contains") {
            // 提取要包含的值
            if let Some(start) = condition.find('"') {
                if let Some(end) = condition[start + 1..].find('"').map(|i| i + start + 1) {
                    let pattern = &condition[start + 1..end];
                    let field_str = String::from_utf8_lossy(field_value);
                    Ok(field_str.contains(pattern))
                } else {
                    Ok(false)
                }
            } else {
                Ok(false)
            }
        } else {
            Ok(false)
        }
    }

    /// 解析值表达式
    fn multiplex_parse_value_expression(&self, expr: &str) -> Result<u64, ProtocolError> {
        let trimmed = expr.trim();

        if trimmed.starts_with("0x") || trimmed.starts_with("0X") {
            // 解析十六进制数
            u64::from_str_radix(&trimmed[2..], 16)
                .map_err(|e| ProtocolError::InvalidExpression(format!("Hex parsing error: {e}")))
        } else if trimmed.starts_with('"') && trimmed.ends_with('"') && trimmed.len() > 1 {
            // 解析字符串长度
            Ok((trimmed.len() - 2) as u64) // 减去两个引号
        } else {
            // 解析十进制数
            trimmed.parse::<u64>().map_err(|e| {
                ProtocolError::InvalidExpression(format!("Decimal parsing error: {e}"))
            })
        }
    }
}
