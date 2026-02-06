//! 长度验证规则处理器
//!
//! 处理长度验证相关的语义规则

use apdl_core::ProtocolError;

use crate::standard_units::frame_assembler::core::FrameAssembler;

impl FrameAssembler {
    /// 应用长度验证规则
    pub fn apply_length_validation_rule(
        &mut self,
        field_name: &str,
        condition: &str,
        description: &str,
        frame_data: &mut [u8],
    ) -> Result<(), ProtocolError> {
        println!(
            "Applying length validation rule: {description} for field {field_name} with condition {condition}"
        );

        match condition {
            "equals_remaining" => {
                self.validate_equals_remaining(field_name, frame_data)?;
            }
            "equals_data_field_plus_header_minus_one" => {
                self.validate_data_field_plus_header_minus_one(field_name, frame_data)?;
            }
            "equals_total_frame_length" => {
                self.validate_equals_total_frame_length(field_name, frame_data)?;
            }
            "greater_than_zero" => {
                self.validate_greater_than_zero(field_name, frame_data)?;
            }
            "within_range" => {
                self.validate_within_range(field_name, frame_data)?;
            }
            _ => {
                // 尝试解析条件表达式
                self.validate_with_expression(field_name, condition, frame_data)?;
            }
        }

        Ok(())
    }

    /// 验证等于剩余长度
    fn validate_equals_remaining(
        &self,
        field_name: &str,
        frame_data: &[u8],
    ) -> Result<(), ProtocolError> {
        let field_pos = self.get_field_position(field_name)?;
        let field_size = self.get_field_size_by_name(field_name)?;

        // 计算从字段结束位置到帧结束的剩余长度
        let remaining_len = if field_pos + field_size < frame_data.len() {
            (frame_data.len() - field_pos - field_size) as u64
        } else {
            0
        };

        // 获取字段值
        let field_value = if let Ok(value) = self.get_field_value(field_name) {
            self.bytes_to_u64(&value)
        } else {
            return Err(ProtocolError::FieldNotFound(format!(
                "Length validation field {field_name} not found"
            )));
        };

        if field_value == remaining_len {
            println!(
                "Length validation passed: field {field_name} = {field_value} (remaining length)"
            );
            Ok(())
        } else {
            Err(ProtocolError::ValidationError(format!(
                "Length validation failed: field {field_name} = {field_value}, remaining = {remaining_len}"
            )))
        }
    }

    /// 验证等于数据字段长度加头部减一
    fn validate_data_field_plus_header_minus_one(
        &self,
        field_name: &str,
        _frame_data: &[u8],
    ) -> Result<(), ProtocolError> {
        // 获取字段值
        let field_value = if let Ok(value) = self.get_field_value(field_name) {
            self.bytes_to_u64(&value)
        } else {
            return Err(ProtocolError::FieldNotFound(format!(
                "Length validation field {field_name} not found"
            )));
        };

        // 查找数据字段
        let mut data_field_size = 0;
        for field in &self.fields {
            if self.is_data_field(field) {
                data_field_size = self.get_field_size(field)? as u64;
                break;
            }
        }

        // 假设头部长度为固定值或根据协议计算
        let header_len = self.calculate_header_length()?;
        let expected_len = data_field_size + header_len - 1;

        if field_value == expected_len {
            println!(
                "Length validation passed: field {field_name} = {field_value} (data field size {data_field_size} + header {header_len} - 1)"
            );
            Ok(())
        } else {
            Err(ProtocolError::ValidationError(format!(
                "Length validation failed: field {field_name} = {field_value}, expected = {expected_len}"
            )))
        }
    }

    /// 验证等于总帧长度
    fn validate_equals_total_frame_length(
        &self,
        field_name: &str,
        frame_data: &[u8],
    ) -> Result<(), ProtocolError> {
        let field_value = if let Ok(value) = self.get_field_value(field_name) {
            self.bytes_to_u64(&value)
        } else {
            return Err(ProtocolError::FieldNotFound(format!(
                "Length validation field {field_name} not found"
            )));
        };

        let total_len = frame_data.len() as u64;

        if field_value == total_len {
            println!(
                "Length validation passed: field {field_name} = {field_value} (total frame length)"
            );
            Ok(())
        } else {
            Err(ProtocolError::ValidationError(format!(
                "Length validation failed: field {field_name} = {field_value}, total frame = {total_len}"
            )))
        }
    }

    /// 验证大于零
    fn validate_greater_than_zero(
        &self,
        field_name: &str,
        _frame_data: &[u8],
    ) -> Result<(), ProtocolError> {
        let field_value = if let Ok(value) = self.get_field_value(field_name) {
            self.bytes_to_u64(&value)
        } else {
            return Err(ProtocolError::FieldNotFound(format!(
                "Length validation field {field_name} not found"
            )));
        };

        if field_value > 0 {
            println!("Length validation passed: field {field_name} = {field_value} (> 0)");
            Ok(())
        } else {
            Err(ProtocolError::ValidationError(format!(
                "Length validation failed: field {field_name} = {field_value} (not greater than 0)"
            )))
        }
    }

    /// 验证在范围内
    fn validate_within_range(
        &self,
        field_name: &str,
        _frame_data: &[u8],
    ) -> Result<(), ProtocolError> {
        let field_value = if let Ok(value) = self.get_field_value(field_name) {
            self.bytes_to_u64(&value)
        } else {
            return Err(ProtocolError::FieldNotFound(format!(
                "Length validation field {field_name} not found"
            )));
        };

        // 这里假设范围是合理的（例如，对于协议字段长度）
        // 实际应用中可能需要从配置中读取范围
        let min_len = 1;
        let max_len = 65535; // 64KB - 合理的最大长度

        if field_value >= min_len && field_value <= max_len {
            println!(
                "Length validation passed: field {field_name} = {field_value} (within range {min_len}-{max_len})"
            );
            Ok(())
        } else {
            Err(ProtocolError::ValidationError(format!(
                "Length validation failed: field {field_name} = {field_value} (not in range {min_len}-{max_len})"
            )))
        }
    }

    /// 使用表达式验证长度
    fn validate_with_expression(
        &self,
        field_name: &str,
        expression: &str,
        frame_data: &[u8],
    ) -> Result<(), ProtocolError> {
        // 解析表达式，例如 "data_length + 7" 或 "total_length - 3"
        let field_value = if let Ok(value) = self.get_field_value(field_name) {
            self.bytes_to_u64(&value)
        } else {
            return Err(ProtocolError::FieldNotFound(format!(
                "Length validation field {field_name} not found"
            )));
        };

        let expected_value = self.evaluate_length_validation_expression(expression, frame_data)?;

        if field_value == expected_value {
            println!(
                "Length validation passed: field {field_name} = {field_value} (matches expression '{expression}')"
            );
            Ok(())
        } else {
            Err(ProtocolError::ValidationError(format!(
                "Length validation failed: field {field_name} = {field_value}, expected by expression '{expression}' = {expected_value}"
            )))
        }
    }

    /// 评估长度验证表达式
    fn evaluate_length_validation_expression(
        &self,
        expression: &str,
        frame_data: &[u8],
    ) -> Result<u64, ProtocolError> {
        let expr_lower = expression.to_lowercase();

        // 处理常见的表达式模式
        if expr_lower.contains("total_length") {
            let total_len = frame_data.len() as u64;

            if let Some(pos) = expr_lower.find("-") {
                let left = &expr_lower[..pos].trim();
                let right = &expr_lower[pos + 1..].trim();

                if left.trim() == "total_length" {
                    if let Ok(right_val) = right.parse::<u64>() {
                        return Ok(total_len.saturating_sub(right_val));
                    }
                }
            } else if let Some(pos) = expr_lower.find("+") {
                let left = &expr_lower[..pos].trim();
                let right = &expr_lower[pos + 1..].trim();

                if left.trim() == "total_length" {
                    if let Ok(right_val) = right.parse::<u64>() {
                        return Ok(total_len + right_val);
                    }
                }
            }

            return Ok(total_len);
        } else if expr_lower.contains("data_length") {
            // 查找数据字段长度
            for field in &self.fields {
                if self.is_data_field(field) {
                    return Ok(self.get_field_size(field)? as u64);
                }
            }
            return Ok(0); // 默认值
        } else {
            // 尝试直接解析为数字
            if let Ok(val) = expression.trim().parse::<u64>() {
                return Ok(val);
            }
        }

        // 如果都不能匹配，返回错误
        Err(ProtocolError::InvalidExpression(format!(
            "Cannot evaluate length validation expression: {expression}"
        )))
    }

    /// 计算头部长度
    fn calculate_header_length(&self) -> Result<u64, ProtocolError> {
        let mut header_len = 0;
        for field in &self.fields {
            if self.is_header_field(field) {
                header_len += self.get_field_size(field)? as u64;
            }
        }
        Ok(header_len)
    }
}
