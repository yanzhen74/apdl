//! 字段映射功能模块

use crate::standard_units::frame_assembler::core::FrameAssembler;
use apdl_core::FieldMappingEntry;

/// 应用映射逻辑
pub(super) fn apply_mapping_logic(
    source_value: &[u8],
    mapping_logic: &str,
    default_value: &str,
) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    match mapping_logic {
        "identity" => Ok(source_value.to_vec()),
        "hash_mod_64" => {
            // 简单的哈希实现
            let hash_value = simple_hash(source_value);
            let result = hash_value % 64;
            Ok(vec![(result & 0xFF) as u8])
        }
        "hash_mod_2048" => {
            // 用于APID的哈希实现
            let hash_value = simple_hash(source_value);
            let result = hash_value % 2048;
            Ok(vec![((result >> 8) & 0xFF) as u8, (result & 0xFF) as u8])
        }
        _ => {
            // 如果映射逻辑无法识别，使用默认值
            parse_default_value(default_value)
        }
    }
}

/// 简单的哈希函数
fn simple_hash(data: &[u8]) -> u64 {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    let mut hasher = DefaultHasher::new();
    data.hash(&mut hasher);
    hasher.finish()
}

/// 解析默认值
fn parse_default_value(default_value: &str) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    if let Some(hex_str) = default_value.strip_prefix("0x") {
        let value = u64::from_str_radix(hex_str, 16)
            .map_err(|_| format!("Invalid hex value: {default_value}"))?;
        Ok(value.to_be_bytes().to_vec())
    } else {
        let value = default_value
            .parse::<u64>()
            .map_err(|_| format!("Invalid decimal value: {default_value}"))?;
        Ok(value.to_be_bytes().to_vec())
    }
}

/// 应用字段映射规则到FrameAssembler
pub(super) fn apply_field_mapping_rules(
    source_assembler: &FrameAssembler,
    target_assembler: &mut FrameAssembler,
    channel: &str,
    mappings: &[FieldMappingEntry],
) -> Result<String, Box<dyn std::error::Error>> {
    let mut dispatch_flag = String::new();
    for mapping in mappings {
        // channel单独处理
        if mapping.source_field == "channel" {
            target_assembler
                .set_field_value(&mapping.target_field, channel.as_bytes())
                .map_err(Box::new)?;
            dispatch_flag.push_str(channel);
            continue;
        }
        // 获取源字段值
        if let Ok(source_value) = source_assembler.get_field_value(&mapping.source_field) {
            // 应用映射逻辑
            let mapped_value = apply_mapping_logic(
                &source_value,
                &mapping.mapping_logic,
                &mapping.default_value,
            )?;

            // 设置目标字段值
            target_assembler
                .set_field_value(&mapping.target_field, &mapped_value)
                .map_err(Box::new)?;
            println!(
                "Mapped {} to {} with value {:?} using logic {}",
                mapping.source_field, mapping.target_field, source_value, mapping.mapping_logic
            );
            // 将目标字段的值添加到dispatch_flag
            let target_value = target_assembler
                .get_field_value(&mapping.target_field)
                .unwrap_or_else(|_| mapped_value.clone());
            dispatch_flag.push_str(&format!("{target_value:?}"));
        }
    }
    Ok(dispatch_flag)
}
