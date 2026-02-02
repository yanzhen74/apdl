//! 公共工具函数
//!
//! 包含多个规则处理器共享的工具函数

use apdl_core::SyntaxUnit;

/// 将字节数组转换为u64
pub fn bytes_to_u64(bytes: &[u8]) -> u64 {
    let mut value = 0u64;
    for (i, &byte) in bytes.iter().enumerate() {
        value |= (byte as u64) << (8 * i);
    }
    value
}

/// 将u64值转换为指定长度的字节数组
pub fn u64_to_bytes(value: u64, size: usize) -> Vec<u8> {
    let mut bytes = Vec::new();
    for i in 0..size {
        bytes.push(((value >> (8 * (size - 1 - i))) & 0xFF) as u8);
    }
    bytes
}

/// 判断是否为数据字段
pub fn is_data_field(field: &SyntaxUnit) -> bool {
    field.field_id.to_lowercase().contains("data")
        || field.field_id.to_lowercase().contains("payload")
        || field.field_id.to_lowercase().contains("message")
        || field.field_id.to_lowercase().contains("content")
}

/// 判断是否为头部字段
pub fn is_header_field(field: &SyntaxUnit) -> bool {
    field.field_id.to_lowercase().contains("sync")
        || field.field_id.to_lowercase().contains("version")
        || field.field_id.to_lowercase().contains("type")
        || field.field_id.to_lowercase().contains("apid")
        || field.field_id.to_lowercase().contains("seq")
}

/// 通配符匹配实现
pub fn wildcard_match(text: &str, pattern: &str) -> bool {
    let text_chars: Vec<char> = text.chars().collect();
    let pattern_chars: Vec<char> = pattern.chars().collect();
    let text_len = text_chars.len();
    let pattern_len = pattern_chars.len();

    // 使用动态规划实现通配符匹配
    let mut dp = vec![vec![false; pattern_len + 1]; text_len + 1];
    dp[0][0] = true;

    // 处理以*开头的情况
    for j in 1..=pattern_len {
        if pattern_chars[j - 1] == '*' {
            dp[0][j] = dp[0][j - 1];
        }
    }

    for i in 1..=text_len {
        for j in 1..=pattern_len {
            if pattern_chars[j - 1] == '*' {
                // *可以匹配任意长度的字符串
                dp[i][j] = dp[i][j - 1] || dp[i - 1][j];
            } else if pattern_chars[j - 1] == '?' || text_chars[i - 1] == pattern_chars[j - 1] {
                // ?匹配任意单个字符
                dp[i][j] = dp[i - 1][j - 1];
            }
        }
    }

    dp[text_len][pattern_len]
}

/// 计算数据的哈希值
pub fn calculate_hash(data: &[u8]) -> u64 {
    let mut hash: u64 = 5381;
    for &byte in data {
        hash = ((hash << 5).wrapping_add(hash)).wrapping_add(byte as u64);
    }
    hash
}

/// 计算CRC16校验和
pub fn calculate_crc16(data: &[u8]) -> u16 {
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

/// 计算简单校验和
pub fn calculate_simple_checksum(data: &[u8]) -> u16 {
    let mut sum: u16 = 0;
    for &byte in data {
        sum = sum.wrapping_add(byte as u16);
    }
    sum
}

/// 计算XOR校验和
pub fn calculate_xor(data: &[u8]) -> u16 {
    let mut xor: u16 = 0;
    for &byte in data {
        xor ^= byte as u16;
    }
    xor
}
