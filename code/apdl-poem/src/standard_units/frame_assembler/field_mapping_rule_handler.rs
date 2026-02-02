//! 字段映射规则处理器
//!
//! 处理字段映射相关的语义规则

use apdl_core::{FieldMappingEntry, ProtocolError};
use hex;

use crate::standard_units::frame_assembler::core::FrameAssembler;

impl FrameAssembler {
    /// 应用字段映射规则
    pub fn apply_field_mapping_rule(
        &mut self,
        source_package: &str,
        target_package: &str,
        mappings: &[FieldMappingEntry],
        description: &str,
        frame_data: &mut Vec<u8>,
    ) -> Result<(), ProtocolError> {
        println!(
            "Applying field mapping rule: {description} from {source_package} to {target_package}"
        );

        // 对每个字段映射进行处理
        for mapping in mappings {
            self.process_single_field_mapping(mapping, frame_data)?;
        }

        Ok(())
    }

    /// 处理单个字段映射
    fn process_single_field_mapping(
        &mut self,
        mapping: &FieldMappingEntry,
        _frame_data: &mut Vec<u8>,
    ) -> Result<(), ProtocolError> {
        println!(
            "Processing field mapping: {} -> {} with logic '{}'",
            mapping.source_field, mapping.target_field, mapping.mapping_logic
        );

        // 获取源字段值
        let source_value = if let Ok(value) = self.get_field_value(&mapping.source_field) {
            value
        } else {
            // 如果源字段不存在，使用默认值
            println!(
                "Source field {} not found, using default value",
                mapping.source_field
            );
            self.parse_default_value(&mapping.default_value)
        };

        // 根据映射逻辑处理源值
        let mapped_value = self.apply_mapping_logic(
            &source_value,
            &mapping.mapping_logic,
            &mapping.enum_mappings,
        )?;

        // 将映射后的值设置到目标字段
        self.set_field_value(&mapping.target_field, &mapped_value)?;

        println!(
            "Mapped field {} with value {:?} to field {} with value {:?}",
            mapping.source_field, source_value, mapping.target_field, mapped_value
        );

        Ok(())
    }

    /// 应用映射逻辑
    fn apply_mapping_logic(
        &self,
        source_value: &[u8],
        mapping_logic: &str,
        enum_mappings: &Option<Vec<apdl_core::EnumMappingEntry>>,
    ) -> Result<Vec<u8>, ProtocolError> {
        // 首先检查是否有枚举映射
        if let Some(enum_mappings) = enum_mappings {
            if let Some(mapped_value) = self.apply_enum_mapping(source_value, enum_mappings) {
                return Ok(mapped_value);
            }
        }

        // 如果没有枚举映射或枚举映射不匹配，则应用常规映射逻辑
        match mapping_logic {
            "identity" | "direct" | "passthrough" => {
                // 恒等映射，直接返回源值
                Ok(source_value.to_vec())
            }
            logic if logic.contains("hash") => {
                // 哈希映射逻辑，例如 "hash_mod_64", "hash(x) % 64"
                self.apply_hash_mapping(source_value, logic)
            }
            logic if logic.contains("shift") => {
                // 位移映射逻辑
                self.apply_shift_mapping(source_value, logic)
            }
            logic if logic.contains("scale") => {
                // 缩放映射逻辑
                self.apply_scale_mapping(source_value, logic)
            }
            logic if logic.contains("mask") => {
                // 掩码映射逻辑
                self.apply_mask_mapping(source_value, logic)
            }
            _ => {
                // 默认使用恒等映射
                Ok(source_value.to_vec())
            }
        }
    }

    /// 应用枚举映射
    fn apply_enum_mapping(
        &self,
        source_value: &[u8],
        enum_mappings: &[apdl_core::EnumMappingEntry],
    ) -> Option<Vec<u8>> {
        // 将源值转换为字符串进行匹配
        let source_str = String::from_utf8_lossy(source_value).to_string();

        for enum_mapping in enum_mappings {
            // 使用通配符匹配算法
            if crate::standard_units::frame_assembler::utils::wildcard_match(
                &source_str,
                &enum_mapping.source_enum,
            ) {
                // 返回目标枚举值
                return Some(enum_mapping.target_enum.as_bytes().to_vec());
            }
        }

        None
    }

    /// 应用哈希映射
    fn apply_hash_mapping(
        &self,
        source_value: &[u8],
        logic: &str,
    ) -> Result<Vec<u8>, ProtocolError> {
        // 计算源值的哈希
        let hash_value =
            crate::standard_units::frame_assembler::utils::calculate_hash(source_value);

        // 根据逻辑表达式计算最终值
        let result_value = if logic.contains("%") {
            // 提取模数
            if let Some(mod_pos) = logic.find('%') {
                let mod_str = logic[mod_pos + 1..].trim();
                if let Ok(mod_value) = mod_str.parse::<u64>() {
                    hash_value % mod_value
                } else {
                    hash_value
                }
            } else {
                hash_value
            }
        } else {
            hash_value
        };

        // 转换为字节数组
        Ok(crate::standard_units::frame_assembler::utils::u64_to_bytes(
            result_value,
            source_value.len().max(1),
        ))
    }

    /// 应用位移映射
    fn apply_shift_mapping(
        &self,
        source_value: &[u8],
        logic: &str,
    ) -> Result<Vec<u8>, ProtocolError> {
        let source_num = crate::standard_units::frame_assembler::utils::bytes_to_u64(source_value);

        let result_value = if logic.contains("left") || logic.contains("<<") {
            // 左移操作
            if let Some(shift_pos) = logic.find(|c: char| c.is_ascii_digit()) {
                if let Ok(shift_amount) = logic[shift_pos..]
                    .chars()
                    .take_while(|c| c.is_ascii_digit())
                    .collect::<String>()
                    .parse::<u32>()
                {
                    source_num << shift_amount.min(63) // 限制在合理范围内
                } else {
                    source_num
                }
            } else {
                source_num
            }
        } else if logic.contains("right") || logic.contains(">>") {
            // 右移操作
            if let Some(shift_pos) = logic.find(|c: char| c.is_ascii_digit()) {
                if let Ok(shift_amount) = logic[shift_pos..]
                    .chars()
                    .take_while(|c| c.is_ascii_digit())
                    .collect::<String>()
                    .parse::<u32>()
                {
                    source_num >> shift_amount.min(63) // 限制在合理范围内
                } else {
                    source_num
                }
            } else {
                source_num
            }
        } else {
            source_num
        };

        // 转换为字节数组
        Ok(crate::standard_units::frame_assembler::utils::u64_to_bytes(
            result_value,
            source_value.len().max(1),
        ))
    }

    /// 应用缩放映射
    fn apply_scale_mapping(
        &self,
        source_value: &[u8],
        logic: &str,
    ) -> Result<Vec<u8>, ProtocolError> {
        let source_num = crate::standard_units::frame_assembler::utils::bytes_to_u64(source_value);

        // 简单缩放逻辑：查找乘数或除数
        let result_value = if logic.contains("*") {
            // 乘法缩放
            if let Some(mul_pos) = logic.find('*') {
                let factor_str = logic[mul_pos + 1..].trim();
                if let Ok(factor) = factor_str.parse::<u64>() {
                    source_num.saturating_mul(factor)
                } else {
                    source_num
                }
            } else {
                source_num
            }
        } else if logic.contains("/") {
            // 除法缩放
            if let Some(div_pos) = logic.find('/') {
                let factor_str = logic[div_pos + 1..].trim();
                if let Ok(factor) = factor_str.parse::<u64>() {
                    if factor != 0 {
                        source_num / factor
                    } else {
                        source_num
                    }
                } else {
                    source_num
                }
            } else {
                source_num
            }
        } else {
            source_num
        };

        // 转换为字节数组
        Ok(crate::standard_units::frame_assembler::utils::u64_to_bytes(
            result_value,
            source_value.len().max(1),
        ))
    }

    /// 应用掩码映射
    fn apply_mask_mapping(
        &self,
        source_value: &[u8],
        logic: &str,
    ) -> Result<Vec<u8>, ProtocolError> {
        let source_num = crate::standard_units::frame_assembler::utils::bytes_to_u64(source_value);

        // 简单掩码逻辑：查找掩码值
        let result_value = if logic.contains("&") {
            // 与操作掩码
            if let Some(mask_pos) = logic.find('&') {
                let mask_str = logic[mask_pos + 1..].trim();
                if mask_str.starts_with("0x") || mask_str.starts_with("0X") {
                    if let Ok(mask) = u64::from_str_radix(&mask_str[2..], 16) {
                        source_num & mask
                    } else {
                        source_num
                    }
                } else if let Ok(mask) = mask_str.parse::<u64>() {
                    source_num & mask
                } else {
                    source_num
                }
            } else {
                source_num
            }
        } else if logic.contains("|") {
            // 或操作掩码
            if let Some(mask_pos) = logic.find('|') {
                let mask_str = logic[mask_pos + 1..].trim();
                if mask_str.starts_with("0x") || mask_str.starts_with("0X") {
                    if let Ok(mask) = u64::from_str_radix(&mask_str[2..], 16) {
                        source_num | mask
                    } else if let Ok(mask) = mask_str.parse::<u64>() {
                        source_num | mask
                    } else {
                        source_num
                    }
                } else if let Ok(mask) = mask_str.parse::<u64>() {
                    source_num | mask
                } else {
                    source_num
                }
            } else {
                source_num
            }
        } else {
            source_num
        };

        // 转换为字节数组
        Ok(crate::standard_units::frame_assembler::utils::u64_to_bytes(
            result_value,
            source_value.len().max(1),
        ))
    }

    /// 解析默认值
    fn parse_default_value(&self, default_value: &str) -> Vec<u8> {
        // 尝试将默认值解析为字节数组
        if default_value.starts_with("0x") || default_value.starts_with("0X") {
            // 解析十六进制
            let hex_str = &default_value[2..];
            hex::decode(hex_str).unwrap_or_else(|_| default_value.as_bytes().to_vec())
        } else {
            // 作为普通字符串处理
            default_value.as_bytes().to_vec()
        }
    }
}
