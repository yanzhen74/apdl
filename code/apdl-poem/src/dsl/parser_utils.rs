//! 解析器工具模块
//!
//! 包含DSL解析器使用的通用辅助函数

use apdl_core::{
    AlgorithmAst, ChecksumAlgorithm, Constraint, CoverDesc, LengthDesc, LengthUnit, ScopeDesc,
    UnitType,
};

/// 解析单元类型
pub fn parse_unit_type(type_str: &str) -> Result<UnitType, String> {
    if type_str.starts_with("Uint") {
        let num_str = &type_str[4..];
        if let Ok(bits) = num_str.parse::<u8>() {
            Ok(UnitType::Uint(bits))
        } else {
            Err(format!("Invalid Uint type: {}", type_str))
        }
    } else if type_str.starts_with("Bit(") && type_str.ends_with(')') {
        let num_str = &type_str[4..type_str.len() - 1];
        if let Ok(bits) = num_str.parse::<u8>() {
            Ok(UnitType::Bit(bits))
        } else {
            Err(format!("Invalid Bit type: {}", type_str))
        }
    } else if type_str == "RawData" {
        Ok(UnitType::RawData)
    } else if type_str == "Ip6Addr" {
        Ok(UnitType::Ip6Addr)
    } else {
        Err(format!("Unknown type: {}", type_str))
    }
}

/// 解析长度描述
pub fn parse_length_desc(length_str: &str) -> Result<LengthDesc, String> {
    let length_str = length_str.trim();
    if length_str.ends_with("byte") {
        let num_str = length_str[..length_str.len() - 4].trim();
        if let Ok(size) = num_str.parse::<usize>() {
            Ok(LengthDesc {
                size,
                unit: LengthUnit::Byte,
            })
        } else {
            Err(format!("Invalid byte length: {}", length_str))
        }
    } else if length_str.ends_with("bit") {
        let num_str = length_str[..length_str.len() - 3].trim();
        if let Ok(size) = num_str.parse::<usize>() {
            Ok(LengthDesc {
                size,
                unit: LengthUnit::Bit,
            })
        } else {
            Err(format!("Invalid bit length: {}", length_str))
        }
    } else if length_str == "dynamic" {
        Ok(LengthDesc {
            size: 0,
            unit: LengthUnit::Dynamic,
        })
    } else if length_str.starts_with('(') && length_str.ends_with(')') {
        Ok(LengthDesc {
            size: 0,
            unit: LengthUnit::Expression(length_str.to_string()),
        })
    } else if let Ok(size) = length_str.parse::<usize>() {
        Ok(LengthDesc {
            size,
            unit: LengthUnit::Byte,
        }) // 默认单位
    } else {
        Ok(LengthDesc {
            size: 0,
            unit: LengthUnit::Expression(length_str.to_string()),
        })
    }
}

/// 解析作用域描述
pub fn parse_scope_desc(scope_str: &str) -> Result<ScopeDesc, String> {
    let scope_str = scope_str.trim();
    if scope_str.starts_with("layer(") && scope_str.ends_with(')') {
        let layer_name = &scope_str[6..scope_str.len() - 1];
        Ok(ScopeDesc::Layer(layer_name.to_string()))
    } else if scope_str.starts_with("cross_layer(") && scope_str.ends_with(')') {
        let layers = &scope_str[12..scope_str.len() - 1];
        if let Some(pos) = layers.find("→") {
            let first = layers[..pos].trim();
            let second = layers[pos + 1..].trim();
            Ok(ScopeDesc::CrossLayer(first.to_string(), second.to_string()))
        } else {
            Err(format!("Invalid cross_layer format: {}", scope_str))
        }
    } else if scope_str.starts_with("global(") && scope_str.ends_with(')') {
        let scope_name = &scope_str[7..scope_str.len() - 1];
        Ok(ScopeDesc::Global(scope_name.to_string()))
    } else {
        Err(format!("Invalid scope format: {}", scope_str))
    }
}

/// 解析覆盖描述
pub fn parse_cover_desc(cover_str: &str) -> Result<CoverDesc, String> {
    let cover_str = cover_str.trim();
    if cover_str == "entire_field" {
        Ok(CoverDesc::EntireField)
    } else if cover_str.contains('[') && cover_str.contains(']') {
        // 解析 field[offset..offset] 格式
        if let Some(open_bracket) = cover_str.find('[') {
            if let Some(close_bracket) = cover_str.find(']') {
                let field_part = &cover_str[..open_bracket];
                let range_part = &cover_str[open_bracket + 1..close_bracket];

                if let Some(double_dot) = range_part.find("..") {
                    let start_str = &range_part[..double_dot];
                    let end_str = &range_part[double_dot + 2..];

                    if let (Ok(start), Ok(end)) =
                        (start_str.parse::<usize>(), end_str.parse::<usize>())
                    {
                        return Ok(CoverDesc::Range(field_part.to_string(), start, end));
                    }
                }
            }
        }
        Ok(CoverDesc::Expression(cover_str.to_string()))
    } else {
        Ok(CoverDesc::Expression(cover_str.to_string()))
    }
}

/// 解析约束条件
pub fn parse_constraint(constraint_str: &str) -> Result<Constraint, String> {
    let constraint_str = constraint_str.trim();
    if constraint_str.starts_with("fixed(") && constraint_str.ends_with(')') {
        let value_str = &constraint_str[6..constraint_str.len() - 1];
        // 尝试解析十进制或十六进制值
        let value = if value_str.starts_with("0x") || value_str.starts_with("0X") {
            u64::from_str_radix(&value_str[2..], 16)
                .map_err(|_| format!("Invalid hex value: {}", value_str))?
        } else {
            value_str
                .parse::<u64>()
                .map_err(|_| format!("Invalid decimal value: {}", value_str))?
        };
        Ok(Constraint::FixedValue(value))
    } else if constraint_str.starts_with("range(") && constraint_str.ends_with(')') {
        let range_str = &constraint_str[6..constraint_str.len() - 1];
        if let Some(double_dot_eq) = range_str.find("..=") {
            let start_str = &range_str[..double_dot_eq];
            let end_str = &range_str[double_dot_eq + 3..];

            // 解析起始值
            let start = if start_str.starts_with("0x") || start_str.starts_with("0X") {
                u64::from_str_radix(&start_str[2..], 16)
                    .map_err(|_| format!("Invalid hex start value: {}", start_str))?
            } else {
                start_str
                    .parse::<u64>()
                    .map_err(|_| format!("Invalid decimal start value: {}", start_str))?
            };

            // 解析结束值
            let end = if end_str.starts_with("0x") || end_str.starts_with("0X") {
                u64::from_str_radix(&end_str[2..], 16)
                    .map_err(|_| format!("Invalid hex end value: {}", end_str))?
            } else {
                end_str
                    .parse::<u64>()
                    .map_err(|_| format!("Invalid decimal end value: {}", end_str))?
            };

            Ok(Constraint::Range(start, end))
        } else {
            Err("Invalid range format, expected x..=y".to_string())
        }
    } else if constraint_str.starts_with("enum(") && constraint_str.ends_with(')') {
        let enum_str = &constraint_str[5..constraint_str.len() - 1];
        let mut enums = Vec::new();
        for pair_str in enum_str.split(',') {
            let pair_parts: Vec<&str> = pair_str.split('=').collect();
            if pair_parts.len() == 2 {
                let value_str = pair_parts[1].trim();
                let value = if value_str.starts_with("0x") || value_str.starts_with("0X") {
                    u64::from_str_radix(&value_str[2..], 16)
                        .map_err(|_| format!("Invalid hex enum value: {}", value_str))?
                } else {
                    value_str
                        .parse::<u64>()
                        .map_err(|_| format!("Invalid decimal enum value: {}", value_str))?
                };
                enums.push((pair_parts[0].trim().to_string(), value));
            } else {
                return Err("Invalid enum format".to_string());
            }
        }
        Ok(Constraint::Enum(enums))
    } else {
        Ok(Constraint::Custom(constraint_str.to_string()))
    }
}

/// 解析算法
pub fn parse_algorithm(alg_str: &str) -> Result<AlgorithmAst, String> {
    let alg_str = alg_str.trim();
    match alg_str {
        "crc16" => Ok(AlgorithmAst::Crc16),
        "crc32" => Ok(AlgorithmAst::Crc32),
        "crc15" => Ok(AlgorithmAst::Crc15), // CAN协议专用
        "xor_sum" => Ok(AlgorithmAst::XorSum),
        _ => Ok(AlgorithmAst::Custom(alg_str.to_string())),
    }
}

/// 解析校验和算法
pub fn parse_checksum_algorithm(alg_str: &str) -> Result<ChecksumAlgorithm, String> {
    match alg_str {
        "CRC16" => Ok(ChecksumAlgorithm::CRC16),
        "CRC32" => Ok(ChecksumAlgorithm::CRC32),
        "CRC15" => Ok(ChecksumAlgorithm::CRC15),
        "XOR" => Ok(ChecksumAlgorithm::XOR),
        _ => Err(format!("Unknown checksum algorithm: {}", alg_str)),
    }
}
