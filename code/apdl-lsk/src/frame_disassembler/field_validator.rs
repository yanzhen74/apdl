//! 字段校验器
//!
//! 负责验证提取的字段值是否满足约束条件

use apdl_core::{Constraint, ProtocolError};

/// 字段校验器
pub struct FieldValidator;

impl FieldValidator {
    /// 验证字段值是否满足约束
    ///
    /// # 参数
    /// - `field_name`: 字段名称
    /// - `value`: 字段值（字节数组）
    /// - `constraint`: 约束条件
    ///
    /// # 返回
    /// - `Ok(())`: 验证通过
    /// - `Err(ProtocolError)`: 验证失败
    pub fn validate(
        field_name: &str,
        value: &[u8],
        constraint: &Constraint,
    ) -> Result<(), ProtocolError> {
        match constraint {
            Constraint::FixedValue(expected) => {
                let actual = Self::bytes_to_u64(value);
                if actual != *expected {
                    return Err(ProtocolError::ValidationError(format!(
                        "Field '{}' fixed value mismatch: expected {}, got {}",
                        field_name, expected, actual
                    )));
                }
            }
            Constraint::Range(min, max) => {
                let actual = Self::bytes_to_u64(value);
                if actual < *min || actual > *max {
                    return Err(ProtocolError::ValueOutOfRange(format!(
                        "Field '{}' out of range: expected [{}, {}], got {}",
                        field_name, min, max, actual
                    )));
                }
            }
            Constraint::Enum(valid_values) => {
                let actual = Self::bytes_to_u64(value);
                // Enum约束包含(name, value)对，我们只比较value部分
                let valid_nums: Vec<u64> = valid_values.iter().map(|(_, v)| *v).collect();
                if !valid_nums.contains(&actual) {
                    return Err(ProtocolError::ValidationError(format!(
                        "Field '{}' invalid enum value: expected {:?}, got {}",
                        field_name, valid_nums, actual
                    )));
                }
            }
            Constraint::Custom(_) => {
                // 自定义约束暂不支持，直接通过
            }
        }

        Ok(())
    }

    /// 将字节数组转换为u64值
    fn bytes_to_u64(bytes: &[u8]) -> u64 {
        let mut value = 0u64;
        for &byte in bytes.iter().take(8) {
            value = (value << 8) | (byte as u64);
        }
        value
    }

    /// 验证CRC16校验和
    ///
    /// # 参数
    /// - `data`: 数据区域
    /// - `expected_crc`: 期望的CRC值
    ///
    /// # 返回
    /// - `Ok(())`: 校验通过
    /// - `Err(ProtocolError)`: 校验失败
    pub fn verify_crc16(data: &[u8], expected_crc: u16) -> Result<(), ProtocolError> {
        let calculated_crc = Self::calculate_crc16(data);
        if calculated_crc != expected_crc {
            return Err(ProtocolError::ChecksumError(format!(
                "CRC16 mismatch: expected 0x{:04X}, got 0x{:04X}",
                expected_crc, calculated_crc
            )));
        }
        Ok(())
    }

    /// 计算CRC16校验和（CCITT多项式）
    fn calculate_crc16(data: &[u8]) -> u16 {
        let mut crc: u16 = 0xFFFF;
        for &byte in data {
            crc ^= (byte as u16) << 8;
            for _ in 0..8 {
                if (crc & 0x8000) != 0 {
                    crc = (crc << 1) ^ 0x1021;
                } else {
                    crc <<= 1;
                }
            }
        }
        crc
    }

    /// 验证简单校验和
    pub fn verify_simple_checksum(data: &[u8], expected: u16) -> Result<(), ProtocolError> {
        let calculated: u16 = data.iter().map(|&b| b as u16).sum();
        if calculated != expected {
            return Err(ProtocolError::ChecksumError(format!(
                "Checksum mismatch: expected {}, got {}",
                expected, calculated
            )));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_fixed_value() {
        // 测试固定值约束
        let constraint = Constraint::FixedValue(0x1234);

        // 正确的值
        let value = vec![0x12, 0x34];
        assert!(FieldValidator::validate("test_field", &value, &constraint).is_ok());

        // 错误的值
        let value = vec![0x12, 0x35];
        assert!(FieldValidator::validate("test_field", &value, &constraint).is_err());
    }

    #[test]
    fn test_validate_range() {
        // 测试范围约束
        let constraint = Constraint::Range(10, 20);

        // 范围内的值
        let value = vec![15];
        assert!(FieldValidator::validate("test_field", &value, &constraint).is_ok());

        // 边界值
        let value = vec![10];
        assert!(FieldValidator::validate("test_field", &value, &constraint).is_ok());
        let value = vec![20];
        assert!(FieldValidator::validate("test_field", &value, &constraint).is_ok());

        // 范围外的值
        let value = vec![9];
        assert!(FieldValidator::validate("test_field", &value, &constraint).is_err());
        let value = vec![21];
        assert!(FieldValidator::validate("test_field", &value, &constraint).is_err());
    }

    #[test]
    fn test_validate_enum() {
        // 测试枚举约束（注意：Enum格式为Vec<(String, u64)>）
        let constraint = Constraint::Enum(vec![
            ("zero".to_string(), 0),
            ("one".to_string(), 1),
            ("three".to_string(), 3),
        ]);

        // 有效的枚举值
        assert!(FieldValidator::validate("test_field", &[0], &constraint).is_ok());
        assert!(FieldValidator::validate("test_field", &[1], &constraint).is_ok());
        assert!(FieldValidator::validate("test_field", &[3], &constraint).is_ok());

        // 无效的枚举值
        assert!(FieldValidator::validate("test_field", &[2], &constraint).is_err());
        assert!(FieldValidator::validate("test_field", &[4], &constraint).is_err());
    }

    #[test]
    fn test_crc16() {
        // 测试CRC16计算
        let data = b"123456789";
        let crc = FieldValidator::calculate_crc16(data);

        // 验证计算的CRC
        assert!(FieldValidator::verify_crc16(data, crc).is_ok());

        // 验证错误的CRC
        assert!(FieldValidator::verify_crc16(data, crc + 1).is_err());
    }

    #[test]
    fn test_simple_checksum() {
        // 测试简单校验和
        let data = vec![1, 2, 3, 4, 5];
        let expected_sum: u16 = 15;

        assert!(FieldValidator::verify_simple_checksum(&data, expected_sum).is_ok());
        assert!(FieldValidator::verify_simple_checksum(&data, 14).is_err());
    }
}
