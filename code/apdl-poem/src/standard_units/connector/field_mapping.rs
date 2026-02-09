//! 字段映射功能模块

use crate::standard_units::frame_assembler::core::FrameAssembler;
use apdl_core::FieldMappingEntry;

/// 应用映射逻辑
pub(super) fn apply_mapping_logic(
    source_value: &[u8],
    mapping_logic: &str,
    default_value: &str,
    mask_table: Option<&[apdl_core::MaskMappingEntry]>,
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
        "mask_table" => {
            // 使用掩码映射表
            if let Some(table) = mask_table {
                apply_mask_mapping_table(source_value, table, default_value)
            } else {
                // 没有提供掩码映射表，使用默认值
                parse_default_value(default_value)
            }
        }
        _ => {
            // 如果映射逻辑无法识别，使用默认值
            parse_default_value(default_value)
        }
    }
}

/// 应用掩码映射表查找
///
/// 根据掩码映射表，将源值应用掩码后查找对应的目标值
///
/// # 参数
/// - `source_value`: 源字段值，如 [0x04, 0x81]
/// - `mask_table`: 掩码映射表
/// - `default_value`: 未匹配时的默认值
///
/// # 返回
/// 匹配的目标值或默认值
pub(super) fn apply_mask_mapping_table(
    source_value: &[u8],
    mask_table: &[apdl_core::MaskMappingEntry],
    default_value: &str,
) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    // 遍历掩码映射表
    for entry in mask_table {
        // 检查长度是否匹配
        if entry.mask.len() != source_value.len() {
            continue;
        }

        // 应用掩码
        let masked_value: Vec<u8> = source_value
            .iter()
            .zip(entry.mask.iter())
            .map(|(src, mask)| src & mask)
            .collect();

        // 检查是否匹配期望的掩码值
        if masked_value == entry.src_masked {
            println!(
                "Mask mapping matched: source={:02X?} & mask={:02X?} = {:02X?} -> dst={:02X?}",
                source_value, entry.mask, masked_value, entry.dst
            );
            return Ok(entry.dst.clone());
        }
    }

    // 未匹配，使用默认值
    println!(
        "Mask mapping not matched for source={source_value:02X?}, using default={default_value}"
    );
    parse_default_value(default_value)
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
            // 统一应用映射逻辑
            let mapped_value = apply_mapping_logic(
                &source_value,
                &mapping.mapping_logic,
                &mapping.default_value,
                mapping.mask_mapping_table.as_deref(),
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
