//! 序列号校验器
//!
//! 基于序列号连续性检测帧丢失

use std::collections::HashMap;

/// 序列号校验结果
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ValidationResult {
    /// 序列号正常
    Ok,
    /// 检测到帧丢失，参数为丢失的帧数
    FrameLost(usize),
    /// 检测到重复帧
    Duplicate,
    /// 序列号回绕（正常）
    Wraparound,
}

/// 序列号校验器
///
/// 用于检测CCSDS协议中的帧丢失和序列号异常
pub struct SequenceValidator {
    /// 各通道的最后序列号（channel_id -> last_sequence）
    last_sequence: HashMap<u16, u32>,
    /// 序列号模数（如CCSDS的14位序列号，模数为0x4000）
    modulo: u32,
}

impl SequenceValidator {
    /// 创建新的序列号校验器
    ///
    /// # 参数
    /// - `modulo`: 序列号模数（如CCSDS 14位序列号为0x4000，即16384）
    ///
    /// # 示例
    /// ```
    /// use apdl_lsk::demultiplex::SequenceValidator;
    ///
    /// // CCSDS Space Packet使用14位序列号
    /// let validator = SequenceValidator::new(0x4000);
    /// ```
    pub fn new(modulo: u32) -> Self {
        Self {
            last_sequence: HashMap::new(),
            modulo,
        }
    }

    /// 验证序列号
    ///
    /// # 参数
    /// - `channel_id`: 通道ID（VCID或APID）
    /// - `sequence`: 当前序列号
    ///
    /// # 返回
    /// - `ValidationResult`: 校验结果
    ///
    /// # 示例
    /// ```
    /// use apdl_lsk::demultiplex::{SequenceValidator, ValidationResult};
    ///
    /// let mut validator = SequenceValidator::new(0x4000);
    ///
    /// // 第一个帧
    /// let result = validator.validate(0, 0);
    /// assert!(matches!(result, ValidationResult::Ok));
    ///
    /// // 连续的第二个帧
    /// let result = validator.validate(0, 1);
    /// assert!(matches!(result, ValidationResult::Ok));
    ///
    /// // 跳过序列号2，检测到丢失1帧
    /// let result = validator.validate(0, 3);
    /// assert!(matches!(result, ValidationResult::FrameLost(1)));
    /// ```
    pub fn validate(&mut self, channel_id: u16, sequence: u32) -> ValidationResult {
        // 获取该通道的最后序列号
        if let Some(&last_seq) = self.last_sequence.get(&channel_id) {
            // 计算期望的序列号
            let expected = (last_seq + 1) % self.modulo;

            if sequence == expected {
                // 序列号正常连续
                self.last_sequence.insert(channel_id, sequence);
                ValidationResult::Ok
            } else if sequence == last_seq {
                // 重复帧
                ValidationResult::Duplicate
            } else {
                // 检测到帧丢失，计算丢失的帧数
                let lost = self.calculate_lost_count(last_seq, sequence);
                self.last_sequence.insert(channel_id, sequence);

                if lost > 0 {
                    ValidationResult::FrameLost(lost)
                } else {
                    // 序列号回绕
                    ValidationResult::Wraparound
                }
            }
        } else {
            // 第一次接收该通道的数据
            self.last_sequence.insert(channel_id, sequence);
            ValidationResult::Ok
        }
    }

    /// 计算丢失的帧数
    ///
    /// # 参数
    /// - `last_seq`: 最后接收的序列号
    /// - `current_seq`: 当前接收的序列号
    ///
    /// # 返回
    /// - 丢失的帧数
    fn calculate_lost_count(&self, last_seq: u32, current_seq: u32) -> usize {
        if current_seq > last_seq {
            // 正常情况：current > last
            (current_seq - last_seq - 1) as usize
        } else {
            // 序列号回绕：current < last
            // 例如：last=0x3FFE, current=0x0002, modulo=0x4000
            // lost = (0x4000 - 0x3FFE - 1) + 0x0002 = 1 + 2 = 3
            let to_wrap = (self.modulo - last_seq - 1) as usize;
            let after_wrap = current_seq as usize;
            to_wrap + after_wrap
        }
    }

    /// 重置指定通道的序列号状态
    pub fn reset_channel(&mut self, channel_id: u16) {
        self.last_sequence.remove(&channel_id);
    }

    /// 获取指定通道的最后序列号
    pub fn get_last_sequence(&self, channel_id: u16) -> Option<u32> {
        self.last_sequence.get(&channel_id).copied()
    }

    /// 清空所有通道的序列号状态
    pub fn clear(&mut self) {
        self.last_sequence.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sequence_validator_normal() {
        let mut validator = SequenceValidator::new(0x4000);

        // 正常连续序列
        assert!(matches!(validator.validate(0, 0), ValidationResult::Ok));
        assert!(matches!(validator.validate(0, 1), ValidationResult::Ok));
        assert!(matches!(validator.validate(0, 2), ValidationResult::Ok));
        assert!(matches!(validator.validate(0, 3), ValidationResult::Ok));
    }

    #[test]
    fn test_sequence_validator_frame_lost() {
        let mut validator = SequenceValidator::new(0x4000);

        // 正常序列
        validator.validate(0, 0);
        validator.validate(0, 1);

        // 跳过序列号2，丢失1帧
        let result = validator.validate(0, 3);
        assert_eq!(result, ValidationResult::FrameLost(1));

        // 跳过序列号4、5、6，丢失3帧
        let result = validator.validate(0, 7);
        assert_eq!(result, ValidationResult::FrameLost(3));
    }

    #[test]
    fn test_sequence_validator_duplicate() {
        let mut validator = SequenceValidator::new(0x4000);

        validator.validate(0, 0);
        validator.validate(0, 1);

        // 重复接收序列号1
        let result = validator.validate(0, 1);
        assert_eq!(result, ValidationResult::Duplicate);
    }

    #[test]
    fn test_sequence_validator_wraparound() {
        let mut validator = SequenceValidator::new(0x4000);

        // 接近序列号上限
        validator.validate(0, 0x3FFE);

        // 正常接收0x3FFF
        assert!(matches!(
            validator.validate(0, 0x3FFF),
            ValidationResult::Ok
        ));

        // 序列号回绕到0（正常）
        assert!(matches!(
            validator.validate(0, 0),
            ValidationResult::Ok
        ));

        // 继续正常序列
        assert!(matches!(
            validator.validate(0, 1),
            ValidationResult::Ok
        ));
    }

    #[test]
    fn test_sequence_validator_wraparound_with_loss() {
        let mut validator = SequenceValidator::new(0x4000);

        // 序列号在0x3FFE
        validator.validate(0, 0x3FFE);

        // 跳过0x3FFF，直接回绕到0x0001（丢失2帧：0x3FFF和0x0000）
        let result = validator.validate(0, 0x0001);
        assert_eq!(result, ValidationResult::FrameLost(2));
    }

    #[test]
    fn test_multiple_channels() {
        let mut validator = SequenceValidator::new(0x4000);

        // 通道0正常序列
        validator.validate(0, 0);
        validator.validate(0, 1);

        // 通道1独立的序列
        validator.validate(1, 10);
        validator.validate(1, 11);

        // 通道0跳过序列号
        let result = validator.validate(0, 3);
        assert_eq!(result, ValidationResult::FrameLost(1));

        // 通道1继续正常
        assert!(matches!(
            validator.validate(1, 12),
            ValidationResult::Ok
        ));
    }

    #[test]
    fn test_reset_channel() {
        let mut validator = SequenceValidator::new(0x4000);

        validator.validate(0, 0);
        validator.validate(0, 1);

        // 重置通道0
        validator.reset_channel(0);

        // 从任意序列号开始都应该是Ok（因为是第一次）
        assert!(matches!(
            validator.validate(0, 100),
            ValidationResult::Ok
        ));
    }

    #[test]
    fn test_get_last_sequence() {
        let mut validator = SequenceValidator::new(0x4000);

        // 初始状态
        assert_eq!(validator.get_last_sequence(0), None);

        // 接收第一个帧
        validator.validate(0, 42);
        assert_eq!(validator.get_last_sequence(0), Some(42));

        // 继续接收
        validator.validate(0, 43);
        assert_eq!(validator.get_last_sequence(0), Some(43));
    }

    #[test]
    fn test_clear() {
        let mut validator = SequenceValidator::new(0x4000);

        validator.validate(0, 0);
        validator.validate(1, 10);

        // 清空所有通道
        validator.clear();

        assert_eq!(validator.get_last_sequence(0), None);
        assert_eq!(validator.get_last_sequence(1), None);
    }

    #[test]
    fn test_ccsds_sequence_modulo() {
        // CCSDS Space Packet使用14位序列号
        let mut validator = SequenceValidator::new(0x4000); // 2^14 = 16384

        // 测试序列号上限
        validator.validate(0, 0x3FFF); // 最大值16383
        assert!(matches!(
            validator.validate(0, 0),
            ValidationResult::Ok
        ));
    }
}
