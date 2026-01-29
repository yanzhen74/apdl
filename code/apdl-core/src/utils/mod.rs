//! 工具模块
//!
//! 提供APDL系统中常用的工具函数

/// CRC-16校验算法
pub fn calculate_ccsds_crc(data: &[u8]) -> u16 {
    let mut crc = 0xFFFF;
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

/// 将字节数组转换为十六进制字符串
pub fn bytes_to_hex(bytes: &[u8]) -> String {
    bytes
        .iter()
        .map(|b| format!("{:02X}", b))
        .collect::<Vec<_>>()
        .join(" ")
}

/// 将十六进制字符串转换为字节数组
pub fn hex_to_bytes(hex_str: &str) -> Result<Vec<u8>, std::num::ParseIntError> {
    let clean_str = hex_str.replace(" ", "");
    let mut bytes = Vec::new();
    for i in (0..clean_str.len()).step_by(2) {
        let byte_str = &clean_str[i..i + 2];
        let byte = u8::from_str_radix(byte_str, 16)?;
        bytes.push(byte);
    }
    Ok(bytes)
}

/// 位操作工具
pub mod bit_ops {
    /// 从字节数组中提取指定范围的位
    pub fn extract_bits(data: &[u8], start_bit: usize, bit_count: usize) -> u64 {
        let mut result = 0u64;
        for i in 0..bit_count {
            let bit_pos = start_bit + i;
            let byte_idx = bit_pos / 8;
            let bit_idx = 7 - (bit_pos % 8); // MSB first

            if byte_idx < data.len() {
                let byte = data[byte_idx];
                let bit = (byte >> bit_idx) & 1;
                result |= (bit as u64) << (bit_count - 1 - i);
            }
        }
        result
    }

    /// 将值设置到位数组的指定位置
    pub fn set_bits(data: &mut [u8], start_bit: usize, bit_count: usize, value: u64) {
        for i in 0..bit_count {
            let bit_pos = start_bit + i;
            let byte_idx = bit_pos / 8;
            let bit_idx = 7 - (bit_pos % 8); // MSB first

            if byte_idx < data.len() {
                let bit_val = (value >> (bit_count - 1 - i)) & 1;
                if bit_val == 1 {
                    data[byte_idx] |= 1 << bit_idx;
                } else {
                    data[byte_idx] &= !(1 << bit_idx);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calculate_ccsds_crc() {
        let data = [0x01, 0x02, 0x03];
        let crc = calculate_ccsds_crc(&data);
        assert!(crc != 0); // 简单测试，确保函数正常工作
    }

    #[test]
    fn test_bytes_to_hex() {
        let bytes = [0xAB, 0xCD, 0xEF];
        let hex = bytes_to_hex(&bytes);
        assert_eq!(hex, "AB CD EF");
    }

    #[test]
    fn test_hex_to_bytes() {
        let hex = "AB CD EF";
        let bytes = hex_to_bytes(hex).unwrap();
        assert_eq!(bytes, [0xAB, 0xCD, 0xEF]);
    }

    #[test]
    fn test_extract_bits() {
        let data = [0b11001010, 0b10110101];
        let extracted = bit_ops::extract_bits(&data, 4, 4);
        assert_eq!(extracted, 0b1010); // 从第4位开始提取4位
    }
}
