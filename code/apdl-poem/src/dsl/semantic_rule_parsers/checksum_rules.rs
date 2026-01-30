//! 校验规则解析器
//!
//! 处理校验相关的语义规则解析

use apdl_core::{ChecksumAlgorithm, SemanticRule};

/// 解析校验和范围规则
pub fn parse_checksum_range(params: &str, rule_type: &str) -> Result<SemanticRule, String> {
    // 解析范围，例如 "field1 to field2" 或 "first to last_before_fieldX"
    let params = params.trim();
    let parts: Vec<&str> = params.split(" to ").collect();
    if parts.len() == 2 {
        Ok(SemanticRule::ChecksumRange {
            algorithm: if rule_type == "crc_range" {
                ChecksumAlgorithm::CRC16
            } else {
                ChecksumAlgorithm::XOR
            },
            start_field: parts[0].trim().to_string(),
            end_field: parts[1].trim().to_string(),
        })
    } else {
        Err("Invalid checksum range format, expected 'field1 to field2'".to_string())
    }
}
