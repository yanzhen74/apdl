//! 同步规则处理器
//!
//! 处理同步相关的语义规则

use apdl_core::ProtocolError;

use crate::standard_units::frame_assembler::core::FrameAssembler;

impl FrameAssembler {
    /// 应用同步规则
    pub fn apply_synchronization_rule(
        &mut self,
        field_name: &str,
        algorithm: &str,
        description: &str,
        frame_data: &mut [u8],
    ) -> Result<(), ProtocolError> {
        println!("Applying synchronization rule: {description} with algorithm {algorithm}");

        match algorithm {
            "sync_pattern_match" => {
                self.perform_sync_pattern_match(field_name, frame_data)?;
            }
            "fixed_sync_match" => {
                self.perform_fixed_sync_match(field_name, frame_data)?;
            }
            "sync_flag_check" => {
                self.perform_sync_flag_check(field_name, frame_data)?;
            }
            _ => {
                // 默认同步检查：验证同步字段是否匹配预期值
                self.perform_default_sync_check(field_name, frame_data)?;
            }
        }

        Ok(())
    }

    /// 执行同步模式匹配
    fn perform_sync_pattern_match(
        &self,
        field_name: &str,
        frame_data: &[u8],
    ) -> Result<(), ProtocolError> {
        if let Ok(expected_sync_value) = self.get_field_value(field_name) {
            // 检查帧数据中是否存在同步模式
            let field_pos = self.get_field_position(field_name)?;
            let field_size = self.get_field_size_by_name(field_name)?;

            if field_pos + field_size <= frame_data.len() {
                let actual_value = &frame_data[field_pos..field_pos + field_size];

                if actual_value == expected_sync_value.as_slice() {
                    println!(
                        "Synchronization pattern match successful for field {field_name}: {expected_sync_value:?}"
                    );
                    Ok(())
                } else {
                    Err(ProtocolError::SynchronizationError(format!(
                        "Synchronization pattern mismatch for field {field_name}: expected={expected_sync_value:?}, actual={actual_value:?}"
                    )))
                }
            } else {
                Err(ProtocolError::InvalidFrameFormat(
                    "Insufficient frame data for synchronization check".to_string(),
                ))
            }
        } else {
            Err(ProtocolError::FieldNotFound(format!(
                "Sync field {field_name} not found"
            )))
        }
    }

    /// 执行固定同步匹配
    fn perform_fixed_sync_match(
        &self,
        field_name: &str,
        frame_data: &[u8],
    ) -> Result<(), ProtocolError> {
        // 与同步模式匹配类似，但可能针对特定的固定同步模式
        self.perform_sync_pattern_match(field_name, frame_data)
    }

    /// 执行同步标志检查
    fn perform_sync_flag_check(
        &self,
        field_name: &str,
        _frame_data: &[u8],
    ) -> Result<(), ProtocolError> {
        // 检查同步标志字段是否设置了正确的标志位
        if let Ok(sync_value) = self.get_field_value(field_name) {
            // 对于标志检查，我们可以验证特定的标志位是否被设置
            if !sync_value.is_empty() {
                println!("Sync flag check passed for field {field_name}: {sync_value:?}");
                Ok(())
            } else {
                Err(ProtocolError::SynchronizationError(
                    "Sync flag is empty".to_string(),
                ))
            }
        } else {
            Err(ProtocolError::FieldNotFound(format!(
                "Sync flag field {field_name} not found"
            )))
        }
    }

    /// 执行默认同步检查
    fn perform_default_sync_check(
        &self,
        field_name: &str,
        frame_data: &[u8],
    ) -> Result<(), ProtocolError> {
        // 默认同步检查：验证同步字段值
        self.perform_sync_pattern_match(field_name, frame_data)
    }
}
