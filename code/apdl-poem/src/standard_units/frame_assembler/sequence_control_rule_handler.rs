//! 序列控制规则处理器
//!
//! 处理序列控制相关的语义规则

use apdl_core::ProtocolError;

use crate::standard_units::frame_assembler::core::FrameAssembler;

impl FrameAssembler {
    /// 应用序列控制规则
    pub fn apply_sequence_control_rule(
        &mut self,
        field_name: &str,
        trigger_condition: &str,
        algorithm: &str,
        description: &str,
        _frame_data: &mut [u8],
    ) -> Result<(), ProtocolError> {
        println!(
            "Applying sequence control rule: {description} with trigger {trigger_condition} and algorithm {algorithm}"
        );

        // 根据触发条件和算法更新序列号
        match trigger_condition {
            "on_transmission" => {
                // 在传输时更新序列号
                self.increment_sequence_number(field_name, algorithm)?;
            }
            "on_change" => {
                // 在值改变时更新序列号
                self.update_sequence_on_change(field_name, algorithm)?;
            }
            _ => {
                // 默认行为：更新序列号
                self.increment_sequence_number(field_name, algorithm)?;
            }
        }

        Ok(())
    }

    /// 增加序列号
    fn increment_sequence_number(
        &mut self,
        field_name: &str,
        algorithm: &str,
    ) -> Result<(), ProtocolError> {
        let current_value = match self.get_field_value(field_name) {
            Ok(value) => self.bytes_to_u64(&value),
            Err(_) => 0, // 如果字段不存在，默认从0开始
        };

        let new_value = match algorithm {
            "increment_seq" => current_value.wrapping_add(1),
            "seq_counter" => current_value.wrapping_add(1),
            "simple_increment" => current_value.wrapping_add(1),
            _ => current_value.wrapping_add(1), // 默认递增
        };

        // 将新值转换回字节数组并设置字段值
        if let Some(&index) = self.field_index.get(field_name) {
            if let Some(field) = self.fields.get(index) {
                let field_size = self.get_field_size(field)?;
                let new_bytes = self.u64_to_bytes(new_value, field_size);

                self.field_values.insert(field_name.to_string(), new_bytes);

                println!("Updated {field_name} from {current_value} to {new_value}");
            }
        }

        Ok(())
    }

    /// 在值改变时更新序列号
    fn update_sequence_on_change(
        &mut self,
        field_name: &str,
        algorithm: &str,
    ) -> Result<(), ProtocolError> {
        // 对于序列控制字段，我们总是递增它
        self.increment_sequence_number(field_name, algorithm)
    }
}
