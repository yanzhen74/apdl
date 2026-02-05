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
        // 直接获取内部存储的字节值，不进行字节序转换
        let clean_field_name = field_name.trim_start_matches("field: ").trim();
        let current_value = match self.field_values.get(clean_field_name) {
            Some(value) => crate::standard_units::frame_assembler::utils::bytes_to_u64_be(value),
            None => 0, // 如果字段不存在，默认从0开始
        };

        let new_value = match algorithm {
            "increment_seq" => current_value.wrapping_add(1),
            "seq_counter" => current_value.wrapping_add(1),
            "simple_increment" => current_value.wrapping_add(1),
            _ => current_value.wrapping_add(1), // 默认递增
        };

        // 将新值转换回大端字节序并直接设置到内部存储
        if let Some(&index) = self.field_index.get(field_name) {
            if let Some(field) = self.fields.get(index) {
                let field_size = self.get_field_size(field)?;
                let new_bytes = crate::standard_units::frame_assembler::utils::u64_to_bytes_be(
                    new_value, field_size,
                );

                // 直接插入内部存储，不进行额外的字节序转换
                self.field_values
                    .insert(clean_field_name.to_string(), new_bytes);

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

#[cfg(test)]
mod tests {
    use super::*;
    use apdl_core::{CoverDesc, LengthDesc, LengthUnit, ScopeDesc, SyntaxUnit, UnitType};

    #[test]
    fn test_apply_sequence_control_rule() {
        // 创建一个FrameAssembler实例
        let mut assembler = FrameAssembler::new();

        // 添加一个序列控制字段
        let seq_field = SyntaxUnit {
            field_id: "sequence_count".to_string(),
            unit_type: UnitType::Uint(16), // 16位无符号整数
            length: LengthDesc {
                size: 2, // 2字节
                unit: LengthUnit::Byte,
            },
            scope: ScopeDesc::Global("test".to_string()),
            cover: CoverDesc::EntireField,
            constraint: None,
            alg: None,
            associate: vec![],
            desc: "Sequence Count Field".to_string(),
        };
        assembler.add_field(seq_field);

        // 初始化序列号字段的值为0
        assembler
            .set_field_value("sequence_count", &[0, 0])
            .unwrap();

        // 应用序列控制规则
        let mut frame_data = vec![0; 10];
        let result = assembler.apply_sequence_control_rule(
            "sequence_count",
            "on_transmission",
            "increment_seq",
            "Increment sequence on transmission",
            &mut frame_data,
        );

        // 验证结果
        assert!(result.is_ok());

        // 验证序列号已递增
        let updated_value = assembler.get_field_value("sequence_count").unwrap();
        assert_eq!(updated_value, vec![0, 1]); // 应该从0增加到1

        // 再次应用规则，验证继续递增
        let result2 = assembler.apply_sequence_control_rule(
            "sequence_count",
            "on_transmission",
            "increment_seq",
            "Increment sequence on transmission",
            &mut frame_data,
        );

        assert!(result2.is_ok());

        let updated_value2 = assembler.get_field_value("sequence_count").unwrap();
        assert_eq!(updated_value2, vec![0, 2]); // 应该从1增加到2

        println!("Sequence control rule test passed!");
    }
}
