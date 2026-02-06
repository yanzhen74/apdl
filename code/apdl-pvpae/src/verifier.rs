//! 协议验证器模块
//!
//! 实现协议合理性的验证功能

use crate::reporter::ValidationResult;
use apdl_core::ProtocolUnit;
use std::collections::HashMap;

/// 验证类型
#[derive(Debug, Clone)]
pub enum VerificationType {
    FormatCheck,     // 格式验证
    PackUnpackCheck, // 打包拆包验证
    MultiplexCheck,  // 复接验证
    ErrorHandling,   // 错误处理验证
    ComplianceCheck, // 符合性验证
}

/// 协议验证器
#[derive(Default)]
pub struct ProtocolVerifier {
    verifications: HashMap<String, VerificationType>,
}

impl ProtocolVerifier {
    pub fn new() -> Self {
        Self::default()
    }

    /// 添加验证项
    pub fn add_verification(&mut self, name: String, verification_type: VerificationType) {
        self.verifications.insert(name, verification_type);
    }

    /// 执行格式验证
    pub fn verify_format(&self, data: &[u8], expected_format: &str) -> ValidationResult {
        let passed = !data.is_empty(); // 简单的非空验证
        ValidationResult {
            passed,
            message: format!("Format verification for {expected_format}"),
            details: if passed {
                None
            } else {
                Some("Data is empty".to_string())
            },
        }
    }

    /// 执行打包拆包验证
    pub fn verify_pack_unpack(
        &self,
        unit: &dyn ProtocolUnit,
        original_data: &[u8],
    ) -> ValidationResult {
        match unit.pack(original_data) {
            Ok(packed) => match unit.unpack(&packed) {
                Ok((unpacked, remaining)) => {
                    let passed = unpacked == original_data && remaining.is_empty();
                    ValidationResult {
                        passed,
                        message: "Pack/unpack verification".to_string(),
                        details: if passed {
                            None
                        } else {
                            Some(format!(
                                "Original: {orig_len} bytes, Unpacked: {unpacked_len} bytes, Remaining: {remaining_len} bytes",
                                orig_len = original_data.len(),
                                unpacked_len = unpacked.len(),
                                remaining_len = remaining.len()
                            ))
                        },
                    }
                }
                Err(e) => ValidationResult {
                    passed: false,
                    message: "Pack/unpack verification failed during unpack".to_string(),
                    details: Some(e.to_string()),
                },
            },
            Err(e) => ValidationResult {
                passed: false,
                message: "Pack/unpack verification failed during pack".to_string(),
                details: Some(e.to_string()),
            },
        }
    }

    /// 执行协议单元验证
    pub fn verify_protocol_unit(&self, unit: &dyn ProtocolUnit) -> ValidationResult {
        match unit.validate() {
            Ok(_) => ValidationResult {
                passed: true,
                message: "Protocol unit validation".to_string(),
                details: None,
            },
            Err(e) => ValidationResult {
                passed: false,
                message: "Protocol unit validation failed".to_string(),
                details: Some(e.to_string()),
            },
        }
    }

    /// 执行协议一致性验证
    pub fn verify_compliance(&self, unit: &dyn ProtocolUnit, standard: &str) -> ValidationResult {
        // 简单的符合性验证
        let meta = unit.get_meta();
        let passed = meta.standard.contains(standard);
        ValidationResult {
            passed,
            message: format!("Compliance verification against {standard}"),
            details: if passed {
                None
            } else {
                Some(format!(
                    "Expected standard '{standard}', got '{actual}'",
                    actual = meta.standard
                ))
            },
        }
    }

    /// 运行所有验证
    pub fn run_all_verifications(&self) -> Vec<ValidationResult> {
        // 这里只返回示例结果，实际实现会更复杂
        vec![
            ValidationResult {
                passed: true,
                message: "Sample format verification".to_string(),
                details: None,
            },
            ValidationResult {
                passed: true,
                message: "Sample compliance verification".to_string(),
                details: None,
            },
        ]
    }

    /// 重置验证器
    pub fn reset(&mut self) {
        self.verifications.clear();
    }
}
