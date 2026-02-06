//! 优先级处理规则处理器
//!
//! 处理优先级处理相关的语义规则

use apdl_core::ProtocolError;

use crate::standard_units::frame_assembler::core::FrameAssembler;

impl FrameAssembler {
    /// 应用优先级处理规则
    pub fn apply_priority_processing_rule(
        &mut self,
        field_name: &str,
        algorithm: &str,
        description: &str,
        frame_data: &mut [u8],
    ) -> Result<(), ProtocolError> {
        println!(
            "Applying priority processing rule: {description} for field {field_name} with algorithm {algorithm}"
        );

        match algorithm {
            "priority_arb" => {
                self.process_priority_arbitration(field_name, frame_data)?;
            }
            "high_priority_first" => {
                self.process_high_priority_first(field_name, frame_data)?;
            }
            "round_robin" => {
                self.process_round_robin(field_name, frame_data)?;
            }
            "fifo_priority" => {
                self.process_fifo_priority(field_name, frame_data)?;
            }
            "weighted_round_robin" => {
                self.process_weighted_round_robin(field_name, frame_data)?;
            }
            _ => {
                // 默认优先级处理
                self.process_default_priority(field_name, frame_data)?;
            }
        }

        Ok(())
    }

    /// 优先级仲裁处理
    fn process_priority_arbitration(
        &mut self,
        field_name: &str,
        _frame_data: &mut [u8],
    ) -> Result<(), ProtocolError> {
        // 获取字段值作为优先级值
        let priority_value = if let Ok(value) = self.get_field_value(field_name) {
            self.bytes_to_u64(&value)
        } else {
            return Err(ProtocolError::FieldNotFound(format!(
                "Priority field {field_name} not found"
            )));
        };

        // 根据优先级值进行处理
        println!("Priority arbitration for field {field_name} with value {priority_value}");

        // TODO: 在实际应用中，这里可能会根据优先级调整处理顺序
        // 在实际应用中，这里可能会根据优先级调整处理顺序
        // 当前我们只是记录优先级值
        Ok(())
    }

    /// 高优先级优先处理
    fn process_high_priority_first(
        &mut self,
        field_name: &str,
        _frame_data: &mut [u8],
    ) -> Result<(), ProtocolError> {
        // 获取字段值作为优先级值
        let priority_value = if let Ok(value) = self.get_field_value(field_name) {
            self.bytes_to_u64(&value)
        } else {
            return Err(ProtocolError::FieldNotFound(format!(
                "Priority field {field_name} not found"
            )));
        };

        // 高数值通常表示高优先级
        if priority_value > 0 {
            println!("High priority processing for field {field_name} with value {priority_value}");
            // TODO: 在实际应用中，这里可能会提前处理高优先级数据
            // 在实际应用中，这里可能会提前处理高优先级数据
        } else {
            println!("Low priority processing for field {field_name} with value {priority_value}");
        }

        Ok(())
    }

    /// 循环处理
    fn process_round_robin(
        &mut self,
        field_name: &str,
        _frame_data: &mut [u8],
    ) -> Result<(), ProtocolError> {
        // 获取字段值用于循环处理
        let round_value = if let Ok(value) = self.get_field_value(field_name) {
            self.bytes_to_u64(&value) % 100 // 限制在0-99范围内
        } else {
            return Err(ProtocolError::FieldNotFound(format!(
                "Round robin field {field_name} not found"
            )));
        };

        println!("Round robin processing for field {field_name} with round value {round_value}");

        // TODO: 在实际应用中，这里可能会根据轮次值进行循环调度
        // 在实际应用中，这里可能会根据轮次值进行循环调度
        Ok(())
    }

    /// FIFO优先级处理
    fn process_fifo_priority(
        &mut self,
        field_name: &str,
        _frame_data: &mut [u8],
    ) -> Result<(), ProtocolError> {
        // FIFO处理主要关注到达顺序，而不是字段值
        println!("FIFO priority processing for field {field_name}");

        // TODO: 在实际应用中，这里可能会维护队列来确保先进先出
        // 在实际应用中，这里可能会维护队列来确保先进先出
        Ok(())
    }

    /// 加权循环处理
    fn process_weighted_round_robin(
        &mut self,
        field_name: &str,
        _frame_data: &mut [u8],
    ) -> Result<(), ProtocolError> {
        // 获取字段值作为权重
        let weight_value = if let Ok(value) = self.get_field_value(field_name) {
            self.bytes_to_u64(&value)
        } else {
            return Err(ProtocolError::FieldNotFound(format!(
                "Weight field {field_name} not found"
            )));
        };

        println!(
            "Weighted round robin processing for field {field_name} with weight {weight_value}"
        );

        // TODO: 在实际应用中，这里会根据权重分配处理时间片
        // 在实际应用中，这里会根据权重分配处理时间片
        Ok(())
    }

    /// 默认优先级处理
    fn process_default_priority(
        &mut self,
        field_name: &str,
        _frame_data: &mut [u8],
    ) -> Result<(), ProtocolError> {
        // 默认处理方式：记录优先级信息
        let priority_value = if let Ok(value) = self.get_field_value(field_name) {
            self.bytes_to_u64(&value)
        } else {
            return Err(ProtocolError::FieldNotFound(format!(
                "Priority field {field_name} not found"
            )));
        };

        println!("Default priority processing for field {field_name} with value {priority_value}");

        Ok(())
    }
}
