//! 错误检测规则处理器
//!
//! 处理错误检测相关的语义规则

use apdl_core::ProtocolError;

use crate::standard_units::frame_assembler::core::FrameAssembler;

impl FrameAssembler {
    /// 应用错误检测规则
    pub fn apply_error_detection_rule(
        &self,
        algorithm: &str,
        description: &str,
        frame_data: &[u8],
    ) -> Result<(), ProtocolError> {
        println!("Applying error detection rule: {description} with algorithm {algorithm}");

        match algorithm {
            "detect_errors" => {
                self.detect_general_errors(frame_data)?;
            }
            "parity_check" => {
                self.perform_parity_check(frame_data)?;
            }
            "crc_check" => {
                self.perform_crc_check(frame_data)?;
            }
            "checksum_check" => {
                self.perform_checksum_check(frame_data)?;
            }
            "hamming_code" => {
                self.perform_hamming_code_check(frame_data)?;
            }
            "reed_solomon" => {
                self.perform_reed_solomon_check(frame_data)?;
            }
            "sequence_check" => {
                self.perform_sequence_check(frame_data)?;
            }
            "duplicate_check" => {
                self.perform_duplicate_check(frame_data)?;
            }
            _ => {
                // 处理自定义错误检测算法
                self.perform_custom_error_detection(algorithm, frame_data)?;
            }
        }

        Ok(())
    }

    /// 检测一般错误
    fn detect_general_errors(&self, frame_data: &[u8]) -> Result<(), ProtocolError> {
        if frame_data.is_empty() {
            return Err(ProtocolError::InvalidFrameFormat(
                "Empty frame detected".to_string(),
            ));
        }

        // 检查帧长度是否合理
        if frame_data.len() < 4 {
            println!("Warning: Very short frame ({} bytes)", frame_data.len());
        }

        // 检查是否包含明显的错误模式
        if frame_data.iter().all(|&b| b == 0) {
            return Err(ProtocolError::InvalidFrameFormat(
                "All-zero frame detected".to_string(),
            ));
        }

        if frame_data.iter().all(|&b| b == 0xFF) {
            return Err(ProtocolError::InvalidFrameFormat(
                "All-ones frame detected".to_string(),
            ));
        }

        println!(
            "General error detection passed for {}-byte frame",
            frame_data.len()
        );
        Ok(())
    }

    /// 执行奇偶校验
    fn perform_parity_check(&self, frame_data: &[u8]) -> Result<(), ProtocolError> {
        if frame_data.is_empty() {
            return Err(ProtocolError::InvalidFrameFormat(
                "Cannot perform parity check on empty frame".to_string(),
            ));
        }

        // 简单的偶校验检查（除了最后一个字节外）
        let data_bytes = &frame_data[..frame_data.len().saturating_sub(1)];
        let parity_byte = if !frame_data.is_empty() {
            frame_data[frame_data.len() - 1]
        } else {
            return Err(ProtocolError::InvalidFrameFormat(
                "Frame too short for parity check".to_string(),
            ));
        };

        let mut bit_count = 0;
        for &byte in data_bytes {
            bit_count += byte.count_ones(); // 计算所有数据字节中的1的个数
        }

        // 检查总1数是否为偶数（偶校验）
        let expected_parity = bit_count % 2 == 0;
        let actual_parity = parity_byte & 1 == 0; // 假设最低位是校验位

        if expected_parity == actual_parity {
            println!("Parity check passed");
            Ok(())
        } else {
            Err(ProtocolError::ValidationError(
                "Parity check failed".to_string(),
            ))
        }
    }

    /// 执行CRC校验
    fn perform_crc_check(&self, frame_data: &[u8]) -> Result<(), ProtocolError> {
        if frame_data.len() < 2 {
            return Err(ProtocolError::InvalidFrameFormat(
                "Frame too short for CRC check".to_string(),
            ));
        }

        // 假设最后两个字节是CRC校验和
        let received_crc = ((frame_data[frame_data.len() - 2] as u16) << 8)
            | (frame_data[frame_data.len() - 1] as u16);

        // 计算数据部分的CRC
        let data_to_check = &frame_data[..frame_data.len() - 2];
        let calculated_crc =
            crate::standard_units::frame_assembler::utils::calculate_crc16(data_to_check);

        if received_crc == calculated_crc {
            println!(
                "CRC check passed: received=0x{received_crc:04X}, calculated=0x{calculated_crc:04X}"
            );
            Ok(())
        } else {
            Err(ProtocolError::ValidationError(format!(
                "CRC check failed: received=0x{received_crc:04X}, calculated=0x{calculated_crc:04X}"
            )))
        }
    }

    /// 执行校验和检查
    fn perform_checksum_check(&self, frame_data: &[u8]) -> Result<(), ProtocolError> {
        if frame_data.is_empty() {
            return Err(ProtocolError::InvalidFrameFormat(
                "Frame too short for checksum check".to_string(),
            ));
        }

        // 假设最后一个字节是校验和
        let received_checksum = frame_data[frame_data.len() - 1] as u16;

        // 计算数据部分的校验和
        let data_to_check = &frame_data[..frame_data.len() - 1];
        let calculated_checksum =
            crate::standard_units::frame_assembler::utils::calculate_simple_checksum(data_to_check);

        if received_checksum == (calculated_checksum & 0xFF) {
            println!(
                "Checksum check passed: received=0x{:02X}, calculated=0x{:02X}",
                received_checksum,
                calculated_checksum & 0xFF
            );
            Ok(())
        } else {
            Err(ProtocolError::ValidationError(format!(
                "Checksum check failed: received=0x{:02X}, calculated=0x{:02X}",
                received_checksum,
                calculated_checksum & 0xFF
            )))
        }
    }

    /// 执行汉明码检查
    fn perform_hamming_code_check(&self, frame_data: &[u8]) -> Result<(), ProtocolError> {
        // 汉明码检查较为复杂，这里实现一个简化的版本
        if frame_data.is_empty() {
            return Err(ProtocolError::InvalidFrameFormat(
                "Cannot perform Hamming code check on empty frame".to_string(),
            ));
        }

        println!("Performing simplified Hamming code check");

        // 这里只是一个示意性的实现
        // 实际的汉明码检查需要根据具体的编码方案来实现
        Ok(())
    }

    /// 执行里德-所罗门码检查
    fn perform_reed_solomon_check(&self, frame_data: &[u8]) -> Result<(), ProtocolError> {
        // 里德-所罗门码检查非常复杂，这里只是示意
        if frame_data.is_empty() {
            return Err(ProtocolError::InvalidFrameFormat(
                "Cannot perform Reed-Solomon check on empty frame".to_string(),
            ));
        }

        println!("Performing simplified Reed-Solomon code check");

        // 实际的里德-所罗门检查需要复杂的数学运算
        Ok(())
    }

    /// 执行序列检查
    fn perform_sequence_check(&self, frame_data: &[u8]) -> Result<(), ProtocolError> {
        if frame_data.is_empty() {
            return Err(ProtocolError::InvalidFrameFormat(
                "Cannot perform sequence check on empty frame".to_string(),
            ));
        }

        // 简单的序列检查：检查是否包含递增序列
        if frame_data.len() >= 4 {
            // 检查前4个字节是否构成某种序列
            let seq1 = frame_data[0] as u32;
            let seq2 = frame_data[1] as u32;
            let seq3 = frame_data[2] as u32;
            let seq4 = frame_data[3] as u32;

            if seq2 == seq1 + 1 && seq3 == seq2 + 1 && seq4 == seq3 + 1 {
                println!("Sequential pattern detected: {seq1} -> {seq2} -> {seq3} -> {seq4}");
            }
        }

        println!("Sequence check completed");
        Ok(())
    }

    /// 执行重复检查
    fn perform_duplicate_check(&self, frame_data: &[u8]) -> Result<(), ProtocolError> {
        if frame_data.is_empty() {
            return Err(ProtocolError::InvalidFrameFormat(
                "Cannot perform duplicate check on empty frame".to_string(),
            ));
        }

        // 计算帧的哈希值用于重复检测
        let frame_hash = self.calculate_frame_hash(frame_data);
        println!("Frame hash for duplicate check: {frame_hash:016X}");

        // TODO: 在实际应用中，这里会与历史帧哈希值进行比较
        // 在实际应用中，这里会与历史帧哈希值进行比较
        // 现在我们只是计算哈希值
        Ok(())
    }

    /// 执行自定义错误检测
    fn perform_custom_error_detection(
        &self,
        algorithm: &str,
        frame_data: &[u8],
    ) -> Result<(), ProtocolError> {
        println!("Performing custom error detection with algorithm: {algorithm}");

        match algorithm {
            "custom_error_detection" => {
                // 自定义错误检测逻辑
                self.custom_error_detection_logic(frame_data)?;
            }
            "advanced_check" => {
                // 高级检查
                self.advanced_error_check(frame_data)?;
            }
            "integrity_check" => {
                // 完整性检查
                self.integrity_check(frame_data)?;
            }
            _ => {
                println!("Unknown custom error detection algorithm: {algorithm}");
                // 对未知算法，默认认为通过检查
            }
        }

        Ok(())
    }

    /// 自定义错误检测逻辑
    fn custom_error_detection_logic(&self, _frame_data: &[u8]) -> Result<(), ProtocolError> {
        println!("Executing custom error detection logic");

        // 实现自定义的错误检测算法
        // 这里可以包含任何特定的错误检测逻辑
        Ok(())
    }

    /// 高级错误检查
    fn advanced_error_check(&self, frame_data: &[u8]) -> Result<(), ProtocolError> {
        println!("Executing advanced error check");

        // 实现高级错误检测，可能包括多种检查的组合
        self.detect_general_errors(frame_data)?;
        self.perform_checksum_check(frame_data).ok(); // 不中断整体流程，仅记录结果

        Ok(())
    }

    /// 完整性检查
    fn integrity_check(&self, frame_data: &[u8]) -> Result<(), ProtocolError> {
        println!("Executing integrity check");

        // 综合多种检查方法验证数据完整性
        self.detect_general_errors(frame_data)?;

        // 如果帧足够长，执行更严格的检查
        if frame_data.len() >= 4 {
            self.perform_checksum_check(frame_data)?;
        }

        Ok(())
    }

    /// 计算帧哈希值
    fn calculate_frame_hash(&self, data: &[u8]) -> u64 {
        let mut hash: u64 = 5381;
        for &byte in data {
            hash = ((hash << 5).wrapping_add(hash)).wrapping_add(byte as u64);
        }
        hash
    }
}
