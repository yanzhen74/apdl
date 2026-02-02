//! 消息过滤规则处理器
//!
//! 处理消息过滤相关的语义规则

use apdl_core::ProtocolError;

use crate::standard_units::frame_assembler::core::FrameAssembler;

impl FrameAssembler {
    /// 应用消息过滤规则
    pub fn apply_message_filtering_rule(
        &mut self,
        condition: &str,
        action: &str,
        description: &str,
        frame_data: &mut Vec<u8>,
    ) -> Result<(), ProtocolError> {
        println!(
            "Applying message filtering rule: {description} with condition {condition} and action {action}"
        );

        // 评估过滤条件
        let should_apply_action = self.evaluate_filter_condition(condition, frame_data)?;

        if should_apply_action {
            println!("Filter condition '{condition}' matched, applying action: {action}");

            match action {
                "accept_msg" => {
                    self.accept_message(description)?;
                }
                "reject_msg" => {
                    self.reject_message(description)?;
                }
                "forward_msg" => {
                    self.forward_message(description)?;
                }
                "drop_msg" => {
                    self.drop_message(description)?;
                }
                "modify_msg" => {
                    self.modify_message(frame_data, description)?;
                }
                "log_msg" => {
                    self.log_message(frame_data, description)?;
                }
                "redirect_msg" => {
                    self.redirect_message(description)?;
                }
                _ => {
                    println!("Unknown action: {action}, treating as accept");
                    self.accept_message(description)?;
                }
            }
        } else {
            println!("Filter condition '{condition}' did not match, message passes through");
            // 条件不匹配，消息通过过滤器
        }

        Ok(())
    }

    /// 评估过滤条件
    fn evaluate_filter_condition(
        &self,
        condition: &str,
        frame_data: &[u8],
    ) -> Result<bool, ProtocolError> {
        match condition {
            "filter_cond" => {
                // 默认过滤条件，接受所有消息
                Ok(true)
            }
            "high_priority_only" => {
                // 仅接受高优先级消息
                self.check_high_priority(frame_data)
            }
            "specific_source" => {
                // 检查特定源
                self.check_specific_source(frame_data)
            }
            "valid_checksum" => {
                // 检查校验和是否有效
                self.check_valid_checksum(frame_data)
            }
            "size_limit" => {
                // 检查消息大小限制
                self.check_size_limit(frame_data)
            }
            "duplicate_filter" => {
                // 去除重复消息
                self.check_duplicate(frame_data)
            }
            _ => {
                // 解析自定义条件
                self.parse_custom_condition(condition, frame_data)
            }
        }
    }

    /// 检查是否为高优先级消息
    fn check_high_priority(&self, frame_data: &[u8]) -> Result<bool, ProtocolError> {
        // 简单检查：如果帧的前几个字节表示优先级且值较高
        if frame_data.len() >= 2 {
            let priority = (frame_data[0] as u16) << 8 | frame_data[1] as u16;
            Ok(priority > 0x8000) // 假设高优先级阈值为0x8000
        } else {
            Ok(false)
        }
    }

    /// 检查特定源
    fn check_specific_source(&self, frame_data: &[u8]) -> Result<bool, ProtocolError> {
        // 简单检查：检查特定的源地址或ID
        if frame_data.len() >= 4 {
            let source_id = ((frame_data[0] as u32) << 24)
                | ((frame_data[1] as u32) << 16)
                | ((frame_data[2] as u32) << 8)
                | (frame_data[3] as u32);

            // 假设特定源ID为0x12345678
            Ok(source_id == 0x12345678)
        } else {
            Ok(false)
        }
    }

    /// 检查校验和是否有效
    fn check_valid_checksum(&self, frame_data: &[u8]) -> Result<bool, ProtocolError> {
        // 简单校验和检查：最后两个字节是校验和
        if frame_data.len() < 2 {
            return Ok(false);
        }

        let received_checksum = ((frame_data[frame_data.len() - 2] as u16) << 8)
            | (frame_data[frame_data.len() - 1] as u16);

        let data_to_check = &frame_data[..frame_data.len() - 2];
        let calculated_checksum =
            crate::standard_units::frame_assembler::utils::calculate_simple_checksum(data_to_check);

        Ok(received_checksum == calculated_checksum)
    }

    /// 检查消息大小限制
    fn check_size_limit(&self, frame_data: &[u8]) -> Result<bool, ProtocolError> {
        // 假设最大消息大小为1024字节
        Ok(frame_data.len() <= 1024)
    }

    /// 检查是否为重复消息
    fn check_duplicate(&self, frame_data: &[u8]) -> Result<bool, ProtocolError> {
        // 这里可以实现重复检测逻辑
        // 简单示例：基于消息内容的哈希
        let message_hash = self.calculate_message_hash(frame_data);

        // TODO: 在实际应用中，这里会检查历史消息缓存
        // 在实际应用中，这里会检查历史消息缓存
        // 现在我们简单地返回true
        println!("Message hash: {message_hash:016X}, checking for duplicates");
        Ok(true) // 假设不是重复消息
    }

    /// 解析自定义条件
    fn parse_custom_condition(
        &self,
        condition: &str,
        frame_data: &[u8],
    ) -> Result<bool, ProtocolError> {
        // 检查条件是否包含比较操作符
        if condition.contains("==") {
            self.filter_parse_equality_condition(condition, frame_data)
        } else if condition.contains(">=") {
            self.parse_greater_equal_condition(condition, frame_data)
        } else if condition.contains(">") {
            self.filter_parse_greater_condition(condition, frame_data)
        } else if condition.contains("<=") {
            self.filter_parse_less_equal_condition(condition, frame_data)
        } else if condition.contains("<") {
            self.filter_parse_less_condition(condition, frame_data)
        } else if condition.contains("!=") {
            self.filter_parse_not_equal_condition(condition, frame_data)
        } else if condition.contains("contains") {
            self.filter_parse_contains_condition(condition, frame_data)
        } else {
            // 默认：如果条件字符串在帧数据中能找到，认为匹配
            let condition_bytes = condition.as_bytes();
            Ok(frame_data
                .windows(condition_bytes.len())
                .any(|window| window == condition_bytes))
        }
    }

    /// 解析相等条件
    fn filter_parse_equality_condition(
        &self,
        condition: &str,
        frame_data: &[u8],
    ) -> Result<bool, ProtocolError> {
        let parts: Vec<&str> = condition.split("==").collect();
        if parts.len() == 2 {
            let field_expr = parts[0].trim();
            let value_expr = parts[1].trim();

            // 解析字段表达式和值表达式
            let expected_value = self.filter_parse_value_expression(value_expr)?;
            let actual_value = self.extract_field_value(field_expr, frame_data)?;

            Ok(actual_value == expected_value)
        } else {
            Ok(false)
        }
    }

    /// 解析大于等于条件
    fn parse_greater_equal_condition(
        &self,
        condition: &str,
        frame_data: &[u8],
    ) -> Result<bool, ProtocolError> {
        let parts: Vec<&str> = condition.split(">=").collect();
        if parts.len() == 2 {
            let field_expr = parts[0].trim();
            let value_expr = parts[1].trim();

            let expected_value = self.filter_parse_value_expression(value_expr)?;
            let actual_value = self.extract_field_value(field_expr, frame_data)?;

            Ok(actual_value >= expected_value)
        } else {
            Ok(false)
        }
    }

    /// 解析大于条件
    fn filter_parse_greater_condition(
        &self,
        condition: &str,
        frame_data: &[u8],
    ) -> Result<bool, ProtocolError> {
        let parts: Vec<&str> = condition.split(">").collect();
        if parts.len() == 2 {
            let field_expr = parts[0].trim();
            let value_expr = parts[1].trim();

            let expected_value = self.filter_parse_value_expression(value_expr)?;
            let actual_value = self.extract_field_value(field_expr, frame_data)?;

            Ok(actual_value > expected_value)
        } else {
            Ok(false)
        }
    }

    /// 解析小于等于条件
    fn filter_parse_less_equal_condition(
        &self,
        condition: &str,
        frame_data: &[u8],
    ) -> Result<bool, ProtocolError> {
        let parts: Vec<&str> = condition.split("<=").collect();
        if parts.len() == 2 {
            let field_expr = parts[0].trim();
            let value_expr = parts[1].trim();

            let expected_value = self.filter_parse_value_expression(value_expr)?;
            let actual_value = self.extract_field_value(field_expr, frame_data)?;

            Ok(actual_value <= expected_value)
        } else {
            Ok(false)
        }
    }

    /// 解析小于条件
    fn filter_parse_less_condition(
        &self,
        condition: &str,
        frame_data: &[u8],
    ) -> Result<bool, ProtocolError> {
        let parts: Vec<&str> = condition.split("<").collect();
        if parts.len() == 2 {
            let field_expr = parts[0].trim();
            let value_expr = parts[1].trim();

            let expected_value = self.filter_parse_value_expression(value_expr)?;
            let actual_value = self.extract_field_value(field_expr, frame_data)?;

            Ok(actual_value < expected_value)
        } else {
            Ok(false)
        }
    }

    /// 解析不等于条件
    fn filter_parse_not_equal_condition(
        &self,
        condition: &str,
        frame_data: &[u8],
    ) -> Result<bool, ProtocolError> {
        let parts: Vec<&str> = condition.split("!=").collect();
        if parts.len() == 2 {
            let field_expr = parts[0].trim();
            let value_expr = parts[1].trim();

            let expected_value = self.filter_parse_value_expression(value_expr)?;
            let actual_value = self.extract_field_value(field_expr, frame_data)?;

            Ok(actual_value != expected_value)
        } else {
            Ok(false)
        }
    }

    /// 解析包含条件
    fn filter_parse_contains_condition(
        &self,
        condition: &str,
        frame_data: &[u8],
    ) -> Result<bool, ProtocolError> {
        if condition.contains("contains") {
            // 提取要搜索的模式
            if let Some(start) = condition.find('"') {
                if let Some(end) = condition[start + 1..].find('"').map(|i| i + start + 1) {
                    let pattern = condition[start + 1..end].as_bytes();
                    Ok(frame_data
                        .windows(pattern.len())
                        .any(|window| window == pattern))
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

    /// 执行接受消息操作
    fn accept_message(&self, description: &str) -> Result<(), ProtocolError> {
        println!("Accepting message: {description}");
        Ok(())
    }

    /// 执行拒绝消息操作
    fn reject_message(&self, description: &str) -> Result<(), ProtocolError> {
        println!("Rejecting message: {description}");
        Ok(())
    }

    /// 执行转发消息操作
    fn forward_message(&self, description: &str) -> Result<(), ProtocolError> {
        println!("Forwarding message: {description}");
        Ok(())
    }

    /// 执行丢弃消息操作
    fn drop_message(&self, description: &str) -> Result<(), ProtocolError> {
        println!("Dropping message: {description}");
        Ok(())
    }

    /// 执行修改消息操作
    fn modify_message(
        &mut self,
        frame_data: &mut [u8],
        description: &str,
    ) -> Result<(), ProtocolError> {
        println!("Modifying message: {description}");

        // 示例：在消息开头添加标记
        if !frame_data.is_empty() {
            frame_data[0] |= 0x80; // 设置最高位作为标记
        }

        Ok(())
    }

    /// 执行记录消息操作
    fn log_message(&self, frame_data: &[u8], description: &str) -> Result<(), ProtocolError> {
        println!(
            "Logging message: {}, data length: {} bytes",
            description,
            frame_data.len()
        );

        // 可能写入日志文件或系统
        Ok(())
    }

    /// 执行重定向消息操作
    fn redirect_message(&self, description: &str) -> Result<(), ProtocolError> {
        println!("Redirecting message: {description}");
        Ok(())
    }

    /// 解析值表达式
    fn filter_parse_value_expression(&self, expr: &str) -> Result<u64, ProtocolError> {
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

    /// 提取字段值
    fn extract_field_value(
        &self,
        field_expr: &str,
        frame_data: &[u8],
    ) -> Result<u64, ProtocolError> {
        // 简单实现：根据字段表达式提取值
        // TODO: 在实际应用中，这将根据具体的协议结构进行解析
        // 在实际应用中，这将根据具体的协议结构进行解析

        // 假设字段表达式是简单的索引，如 "field[0]", "field[1]" 等
        if field_expr.starts_with("field[") && field_expr.ends_with(']') {
            let index_str = &field_expr[6..field_expr.len() - 1]; // 提取 [ ] 中的内容
            if let Ok(index) = index_str.parse::<usize>() {
                if index < frame_data.len() {
                    Ok(frame_data[index] as u64)
                } else {
                    Ok(0) // 索引超出范围，返回0
                }
            } else {
                Ok(0) // 无法解析索引，返回0
            }
        } else {
            // 如果不是索引形式，返回第一个字节的值作为默认
            if frame_data.is_empty() {
                Ok(0)
            } else {
                Ok(frame_data[0] as u64)
            }
        }
    }

    /// 计算消息哈希
    fn calculate_message_hash(&self, data: &[u8]) -> u64 {
        let mut hash: u64 = 5381;
        for &byte in data {
            hash = ((hash << 5).wrapping_add(hash)).wrapping_add(byte as u64);
        }
        hash
    }
}
