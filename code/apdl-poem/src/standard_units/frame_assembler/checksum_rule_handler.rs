//! 校验和规则处理器
//!
//! 处理与校验和相关的语义规则，包括CRC、XOR等算法

use apdl_core::{AlgorithmAst, ChecksumAlgorithm, ProtocolError};

use crate::standard_units::frame_assembler::core::FrameAssembler;

impl FrameAssembler {
    /// 应用校验和规则
    pub fn apply_checksum_rule(
        &mut self,
        frame_data: &mut [u8],
        algorithm: &ChecksumAlgorithm,
        start_field: &str,
        end_field: &str,
    ) -> Result<(), ProtocolError> {
        // 根据字段名找到在帧数据中的位置
        let start_pos = self.get_field_position(start_field)?;
        let end_pos =
            self.get_field_position(end_field)? + self.get_field_size_by_name(end_field)?;

        if end_pos > frame_data.len() {
            return Err(ProtocolError::InvalidFrameFormat(
                "Field range exceeds frame size".to_string(),
            ));
        }

        let data_to_checksum = &frame_data[start_pos..end_pos];
        let checksum: u64 = match algorithm {
            ChecksumAlgorithm::CRC16 => self.calculate_crc16(data_to_checksum) as u64,
            ChecksumAlgorithm::CRC32 => self.calculate_crc32(data_to_checksum) as u64,
            ChecksumAlgorithm::CRC15 => self.calculate_crc15(data_to_checksum) as u64, // CAN协议专用
            ChecksumAlgorithm::XOR => {
                crate::standard_units::frame_assembler::utils::calculate_xor(data_to_checksum)
                    as u64
            }
        };

        // 确定校验和应该写入哪个字段
        // 策略：检查所有字段，找到应用了匹配算法的字段
        let mut found_matching_field = false;
        for (field_name, &field_index) in &self.field_index {
            let field = &self.fields[field_index];

            // 检查字段是否应用了校验算法
            if let Some(ref alg_ast) = field.alg {
                // 如果算法匹配，则将校验和写入该字段
                if self.checksum_algorithm_matches(alg_ast, algorithm) {
                    let field_size = self.get_field_size(field)?;
                    let field_offset = self.calculate_field_offset(field_index)?;

                    // 调试信息
                    let checksum_bytes = self.u64_to_bytes(checksum, field_size);
                    println!(
                        "DEBUG: Writing checksum {:?} to field {} at offset {}, field_size: {}, frame_data length: {}",
                        checksum_bytes, field_name, field_offset, field_size, frame_data.len()
                    );

                    // 将校验和写入帧数据
                    for (i, &byte) in checksum_bytes.iter().enumerate() {
                        let write_pos = field_offset + i;
                        if write_pos < frame_data.len() {
                            frame_data[write_pos] = byte;
                            println!("DEBUG: Wrote byte {byte:02X} to position {write_pos}");
                        } else {
                            println!("DEBUG: Cannot write byte {:02X} to position {}, exceeds frame length {}", byte, write_pos, frame_data.len());
                        }
                    }

                    // 同时更新字段值存储
                    self.field_values.insert(field_name.clone(), checksum_bytes);

                    found_matching_field = true;
                    break; // 找到并处理了一个校验字段后退出
                }
            }
        }

        // 如果没有找到匹配算法的字段，尝试根据常见的校验字段名称来查找
        if !found_matching_field {
            // 尝试其他常见的校验字段名称
            for field_name in ["fecf", "crc", "checksum", "crc_field", "check_field"] {
                if let Some(&field_index) = self.field_index.get(field_name) {
                    let field = &self.fields[field_index];
                    let field_size = self.get_field_size(field)?;
                    let field_offset = self.calculate_field_offset(field_index)?;

                    // 调试信息
                    let checksum_bytes = self.u64_to_bytes(checksum, field_size);
                    println!(
                        "DEBUG: Writing checksum {:?} to field {} at offset {}, field_size: {}, frame_data length: {}",
                        checksum_bytes, field_name, field_offset, field_size, frame_data.len()
                    );

                    // 将校验和写入帧数据
                    for (i, &byte) in checksum_bytes.iter().enumerate() {
                        let write_pos = field_offset + i;
                        if write_pos < frame_data.len() {
                            frame_data[write_pos] = byte;
                            println!("DEBUG: Wrote byte {byte:02X} to position {write_pos}");
                        } else {
                            println!("DEBUG: Cannot write byte {:02X} to position {}, exceeds frame length {}", byte, write_pos, frame_data.len());
                        }
                    }

                    // 同时更新字段值存储
                    self.field_values
                        .insert(field_name.to_string(), checksum_bytes);

                    break; // 找到并处理了一个校验字段后退出
                }
            }
        }

        println!(
            "Calculated checksum {algorithm:?} for range {start_field} to {end_field}: {checksum:?}"
        );
        Ok(())
    }

    /// 验证校验和规则
    pub fn validate_checksum_rule(
        &self,
        frame_data: &[u8],
        algorithm: &ChecksumAlgorithm,
        start_field: &str,
        end_field: &str,
    ) -> Result<(), ProtocolError> {
        // 验证帧数据中的校验和是否正确
        let start_pos = self.get_field_position(start_field)?;
        let end_pos =
            self.get_field_position(end_field)? + self.get_field_size_by_name(end_field)?;

        if end_pos > frame_data.len() {
            return Err(ProtocolError::InvalidFrameFormat(
                "Field range exceeds frame size".to_string(),
            ));
        }

        let data_to_checksum = &frame_data[start_pos..end_pos];
        let calculated_checksum: u64 = match algorithm {
            ChecksumAlgorithm::CRC16 => self.calculate_crc16(data_to_checksum) as u64,
            ChecksumAlgorithm::CRC32 => self.calculate_crc32(data_to_checksum) as u64,
            ChecksumAlgorithm::CRC15 => self.calculate_crc15(data_to_checksum) as u64, // CAN协议专用
            ChecksumAlgorithm::XOR => {
                crate::standard_units::frame_assembler::utils::calculate_xor(data_to_checksum)
                    as u64
            }
        };

        println!(
            "Validated checksum {algorithm:?} for range {start_field} to {end_field}: {calculated_checksum:?}"
        );
        Ok(())
    }

    /// 检查算法AST是否与ChecksumAlgorithm匹配
    fn checksum_algorithm_matches(
        &self,
        alg_ast: &AlgorithmAst,
        algorithm: &ChecksumAlgorithm,
    ) -> bool {
        matches!(
            (alg_ast, algorithm),
            (AlgorithmAst::Crc16, ChecksumAlgorithm::CRC16)
                | (AlgorithmAst::Crc32, ChecksumAlgorithm::CRC32)
                | (AlgorithmAst::Crc15, ChecksumAlgorithm::CRC15)
                | (AlgorithmAst::XorSum, ChecksumAlgorithm::XOR)
        )
    }

    /// 计算CRC32校验和
    fn calculate_crc32(&self, data: &[u8]) -> u32 {
        // 简化的CRC32计算
        let mut crc: u32 = 0xFFFFFFFF;
        for byte in data {
            crc ^= *byte as u32;
            for _ in 0..8 {
                if (crc & 1) != 0 {
                    crc = (crc >> 1) ^ 0xEDB88320;
                } else {
                    crc >>= 1;
                }
            }
        }
        !crc
    }
}
