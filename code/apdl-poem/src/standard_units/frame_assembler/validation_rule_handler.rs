//! 验证规则处理器
//!
//! 处理验证相关的语义规则

use apdl_core::ProtocolError;

use crate::standard_units::frame_assembler::core::FrameAssembler;

impl FrameAssembler {
    /// 应用验证规则
    pub fn apply_validation_rule(
        &mut self,
        field_name: &str,
        algorithm: &str,
        range_start: &str,
        range_end: &str,
        description: &str,
        frame_data: &mut [u8],
    ) -> Result<(), ProtocolError> {
        println!(
            "Applying validation rule: {description} with algorithm {algorithm} for range {range_start} to {range_end}"
        );

        // 验证字段值是否符合预期
        match algorithm {
            "crc16_verification" | "crc_verification" => {
                self.validate_crc16(field_name, range_start, range_end, frame_data)?;
            }
            "xor_verification" | "xor_validation" => {
                self.validate_xor(field_name, range_start, range_end, frame_data)?;
            }
            _ => {
                // 对于其他验证算法，简单检查字段是否存在
                if self.field_values.contains_key(field_name) {
                    println!("Field {field_name} exists, basic validation passed");
                } else {
                    println!("Warning: Field {field_name} not found for validation");
                }
            }
        }

        Ok(())
    }

    /// 验证CRC16校验和
    fn validate_crc16(
        &self,
        field_name: &str,
        range_start: &str,
        range_end: &str,
        frame_data: &[u8],
    ) -> Result<(), ProtocolError> {
        // 获取验证字段的期望校验和值
        let expected_checksum = if let Ok(field_value) = self.get_field_value(field_name) {
            self.bytes_to_u16(&field_value)
        } else {
            return Err(ProtocolError::FieldNotFound(format!(
                "Validation field {field_name} not found"
            )));
        };

        // 计算指定范围内数据的CRC16值
        let start_pos = self.get_field_position(range_start)?;
        let end_pos =
            self.get_field_position(range_end)? + self.get_field_size_by_name(range_end)?;

        if end_pos > frame_data.len() {
            return Err(ProtocolError::InvalidFrameFormat(
                "Validation range exceeds frame size".to_string(),
            ));
        }

        let data_to_validate = &frame_data[start_pos..end_pos];
        let calculated_checksum = self.calculate_crc16(data_to_validate);

        // 验证校验和是否匹配
        if calculated_checksum == expected_checksum {
            println!(
                "CRC16 validation passed for field {field_name}: expected=0x{expected_checksum:04X}, calculated=0x{calculated_checksum:04X}"
            );
            Ok(())
        } else {
            Err(ProtocolError::ValidationError(format!(
                "CRC16 validation failed for field {field_name}: expected=0x{expected_checksum:04X}, calculated=0x{calculated_checksum:04X}"
            )))
        }
    }

    /// 验证XOR校验和
    fn validate_xor(
        &self,
        field_name: &str,
        range_start: &str,
        range_end: &str,
        frame_data: &[u8],
    ) -> Result<(), ProtocolError> {
        // 获取验证字段的期望校验和值
        let expected_checksum = if let Ok(field_value) = self.get_field_value(field_name) {
            self.bytes_to_u16(&field_value)
        } else {
            return Err(ProtocolError::FieldNotFound(format!(
                "Validation field {field_name} not found"
            )));
        };

        // 计算指定范围内数据的XOR值
        let start_pos = self.get_field_position(range_start)?;
        let end_pos =
            self.get_field_position(range_end)? + self.get_field_size_by_name(range_end)?;

        if end_pos > frame_data.len() {
            return Err(ProtocolError::InvalidFrameFormat(
                "Validation range exceeds frame size".to_string(),
            ));
        }

        let data_to_validate = &frame_data[start_pos..end_pos];
        let calculated_checksum = self.calculate_xor(data_to_validate);

        // 验证校验和是否匹配
        if calculated_checksum == expected_checksum {
            println!(
                "XOR validation passed for field {field_name}: expected=0x{expected_checksum:04X}, calculated=0x{calculated_checksum:04X}"
            );
            Ok(())
        } else {
            Err(ProtocolError::ValidationError(format!(
                "XOR validation failed for field {field_name}: expected=0x{expected_checksum:04X}, calculated=0x{calculated_checksum:04X}"
            )))
        }
    }

    /// 将字节数组转换为u16
    fn bytes_to_u16(&self, bytes: &[u8]) -> u16 {
        if bytes.len() >= 2 {
            ((bytes[0] as u16) << 8) | (bytes[1] as u16)
        } else if bytes.len() == 1 {
            bytes[0] as u16
        } else {
            0
        }
    }
}
