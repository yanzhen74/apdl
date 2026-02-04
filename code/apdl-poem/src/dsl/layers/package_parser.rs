//! 包解析器模块
//!
//! 处理包定义的解析

use apdl_core::{
    CoverDesc, LayerDefinition, LengthDesc, LengthUnit, PackageDefinition, ScopeDesc, SyntaxUnit,
    UnitType,
};

/// 包解析器
pub struct PackageParser;

impl PackageParser {
    /// 解析包定义
    pub fn parse_package_definition(dsl_text: &str) -> Result<PackageDefinition, String> {
        // 解析包定义的 DSL 文本
        // 格式: package <package_name> { ... }

        let dsl_text = dsl_text.trim();

        // 提取包名
        let package_def_start = "package ";
        if !dsl_text.starts_with(package_def_start) {
            return Err("Not a package definition".to_string());
        }

        let after_package = &dsl_text[package_def_start.len()..].trim_start();

        // 查找包名结束位置（空格或左花括号）
        let mut package_name_end = 0;
        for (i, c) in after_package.chars().enumerate() {
            if c.is_whitespace() || c == '{' {
                package_name_end = i;
                break;
            }
        }

        let package_name = after_package[..package_name_end].trim();
        let remaining = &after_package[package_name_end..].trim_start();

        // 确保以左花括号开始
        if !remaining.starts_with('{') {
            return Err("Package definition must start with {".to_string());
        }

        // 找到匹配的右花括号
        let content = Self::extract_braced_content(remaining)?;

        // 解析包定义内容
        let mut display_name = String::new();
        let mut package_type = String::new();
        let mut description = String::new();
        let mut layers_str = String::new();

        // 解析各个字段
        for line in content.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with("//") {
                continue;
            }

            if line.starts_with("name:") {
                display_name = Self::extract_quoted_value(line)?;
            } else if line.starts_with("type:") {
                package_type = Self::extract_simple_value(line)?;
            } else if line.starts_with("desc:") {
                description = Self::extract_quoted_value(line)?;
            } else if line.starts_with("layers:") {
                // 提取整个layers数组内容
                layers_str = Self::extract_array_content(line)?;
            }
        }

        // 解析层定义
        let layers = if !layers_str.is_empty() {
            Self::parse_layers(&layers_str)?
        } else {
            Vec::new()
        };

        // 设置默认值
        let display_name = if display_name.is_empty() {
            package_name.to_string()
        } else {
            display_name
        };
        let package_type = if package_type.is_empty() {
            "generic".to_string()
        } else {
            package_type
        };

        Ok(PackageDefinition {
            name: package_name.to_string(),
            display_name,
            package_type,
            layers,
            description,
        })
    }

    /// 解析层定义列表
    fn parse_layers(layers_content: &str) -> Result<Vec<LayerDefinition>, String> {
        let mut layers = Vec::new();

        // 提取数组内容
        let layers_array_content = Self::extract_braced_content_from_array(layers_content)?;

        // 按层分割（寻找 }, 匹配来分隔不同的层）
        let layer_defs = Self::split_layer_definitions(&layers_array_content);

        for layer_def in layer_defs {
            let layer = Self::parse_single_layer(layer_def)?;
            layers.push(layer);
        }

        Ok(layers)
    }

    /// 解析单个层定义
    fn parse_single_layer(layer_content: &str) -> Result<LayerDefinition, String> {
        let mut name = String::new();
        let mut units_str = String::new();
        let mut rules_str = String::new();

        // 解析层内容
        for line in layer_content.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with("//") {
                continue;
            }

            if line.starts_with("name:") {
                name = Self::extract_quoted_value(line)?;
            } else if line.starts_with("units:") {
                units_str = Self::extract_array_content(line)?;
            } else if line.starts_with("rules:") {
                rules_str = Self::extract_array_content(line)?;
            }
        }

        // 解析语法单元
        let units = if !units_str.is_empty() {
            Self::parse_syntax_units(&units_str)?
        } else {
            Vec::new()
        };

        // 解析语义规则
        let rules = if !rules_str.is_empty() {
            // 这里需要调用现有的语义规则解析器来解析rules数组
            // 为简化，暂时返回空向量，实际实现需要更复杂的解析
            Vec::new()
        } else {
            Vec::new()
        };

        Ok(LayerDefinition {
            name: if name.is_empty() {
                "default_layer".to_string()
            } else {
                name
            },
            units,
            rules,
        })
    }

    /// 解析语法单元列表
    fn parse_syntax_units(units_content: &str) -> Result<Vec<SyntaxUnit>, String> {
        let mut units = Vec::new();

        // 提取数组内容
        let units_array_content = Self::extract_braced_content_from_array(units_content)?;

        // 按语法单元分割（寻找 }, 匹配来分隔不同的语法单元）
        let unit_defs = Self::split_unit_definitions(&units_array_content);

        for unit_def in unit_defs {
            let unit = Self::parse_single_syntax_unit(unit_def)?;
            units.push(unit);
        }

        Ok(units)
    }

    /// 解析单个语法单元定义
    fn parse_single_syntax_unit(unit_content: &str) -> Result<SyntaxUnit, String> {
        let mut field_id = String::new();
        let mut unit_type_str = String::new();
        let mut length_str = String::new();
        let mut scope_str = String::new();
        let mut cover_str = String::new();
        let mut constraint_str = String::new();
        let mut alg_str = String::new();
        let mut associate_str = String::new();
        let mut desc_str = String::new();

        // 解析语法单元内容
        for line in unit_content.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with("//") {
                continue;
            }

            if line.starts_with("field:") {
                field_id = Self::extract_simple_value(line)?;
            } else if line.starts_with("type:") {
                unit_type_str = Self::extract_simple_value(line)?;
            } else if line.starts_with("length:") {
                length_str = Self::extract_simple_value(line)?;
            } else if line.starts_with("scope:") {
                scope_str = Self::extract_simple_value(line)?;
            } else if line.starts_with("cover:") {
                cover_str = Self::extract_simple_value(line)?;
            } else if line.starts_with("constraint:") {
                constraint_str = Self::extract_simple_value(line)?;
            } else if line.starts_with("alg:") {
                alg_str = Self::extract_simple_value(line)?;
            } else if line.starts_with("associate:") {
                associate_str = Self::extract_simple_value(line)?;
            } else if line.starts_with("desc:") {
                desc_str = Self::extract_quoted_value(line)?;
            }
        }

        // 解析各个组件
        let unit_type = Self::parse_unit_type(&unit_type_str)?;
        let length = Self::parse_length_desc(&length_str)?;
        let scope = Self::parse_scope_desc(&scope_str)?;
        let cover = Self::parse_cover_desc(&cover_str)?;

        let constraint = if !constraint_str.is_empty() {
            Some(Self::parse_constraint(&constraint_str)?)
        } else {
            None
        };

        let alg = if !alg_str.is_empty() {
            Some(Self::parse_algorithm(&alg_str)?)
        } else {
            None
        };

        let associate = if !associate_str.is_empty() {
            associate_str
                .split(',')
                .map(|s| s.trim().to_string())
                .collect()
        } else {
            Vec::new()
        };

        Ok(SyntaxUnit {
            field_id: if field_id.is_empty() {
                return Err("Missing field_id in syntax unit".to_string());
            } else {
                field_id
            },
            unit_type,
            length,
            scope,
            cover,
            constraint,
            alg,
            associate,
            desc: desc_str,
        })
    }

    // 从dsl/parser_utils.rs复制的相关解析函数
    /// 解析单元类型
    fn parse_unit_type(type_str: &str) -> Result<UnitType, String> {
        if let Some(num_str) = type_str.strip_prefix("Uint") {
            if let Ok(bits) = num_str.parse::<u8>() {
                Ok(UnitType::Uint(bits))
            } else {
                Err(format!("Invalid Uint type: {type_str}"))
            }
        } else if type_str.starts_with("Bit(") && type_str.ends_with(')') {
            let num_str = &type_str[4..type_str.len() - 1];
            if let Ok(bits) = num_str.parse::<u8>() {
                Ok(UnitType::Bit(bits))
            } else {
                Err(format!("Invalid Bit type: {type_str}"))
            }
        } else if type_str == "RawData" {
            Ok(UnitType::RawData)
        } else if type_str == "Ip6Addr" {
            Ok(UnitType::Ip6Addr)
        } else {
            Err(format!("Unknown type: {type_str}"))
        }
    }

    /// 解析长度描述
    fn parse_length_desc(length_str: &str) -> Result<LengthDesc, String> {
        let length_str = length_str.trim();
        if length_str.ends_with("byte") {
            let num_str = length_str[..length_str.len() - 4].trim();
            if let Ok(size) = num_str.parse::<usize>() {
                Ok(LengthDesc {
                    size,
                    unit: LengthUnit::Byte,
                })
            } else {
                Err(format!("Invalid byte length: {length_str}"))
            }
        } else if length_str.ends_with("bit") {
            let num_str = length_str[..length_str.len() - 3].trim();
            if let Ok(size) = num_str.parse::<usize>() {
                Ok(LengthDesc {
                    size,
                    unit: LengthUnit::Bit,
                })
            } else {
                Err(format!("Invalid bit length: {length_str}"))
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
    fn parse_scope_desc(scope_str: &str) -> Result<ScopeDesc, String> {
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
                Err(format!("Invalid cross_layer format: {scope_str}"))
            }
        } else if scope_str.starts_with("global(") && scope_str.ends_with(')') {
            let scope_name = &scope_str[7..scope_str.len() - 1];
            Ok(ScopeDesc::Global(scope_name.to_string()))
        } else {
            Err(format!("Invalid scope format: {scope_str}"))
        }
    }

    /// 解析覆盖描述
    fn parse_cover_desc(cover_str: &str) -> Result<CoverDesc, String> {
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
    fn parse_constraint(constraint_str: &str) -> Result<apdl_core::Constraint, String> {
        let constraint_str = constraint_str.trim();
        if constraint_str.starts_with("fixed(") && constraint_str.ends_with(')') {
            let value_str = &constraint_str[6..constraint_str.len() - 1];
            // 尝试解析十进制或十六进制值
            let value = if value_str.starts_with("0x") || value_str.starts_with("0X") {
                u64::from_str_radix(&value_str[2..], 16)
                    .map_err(|_| format!("Invalid hex value: {value_str}"))?
            } else {
                value_str
                    .parse::<u64>()
                    .map_err(|_| format!("Invalid decimal value: {value_str}"))?
            };
            Ok(apdl_core::Constraint::FixedValue(value))
        } else if constraint_str.starts_with("range(") && constraint_str.ends_with(')') {
            let range_str = &constraint_str[6..constraint_str.len() - 1];
            if let Some(double_dot_eq) = range_str.find("..=") {
                let start_str = &range_str[..double_dot_eq];
                let end_str = &range_str[double_dot_eq + 3..];

                // 解析起始值
                let start = if start_str.starts_with("0x") || start_str.starts_with("0X") {
                    u64::from_str_radix(&start_str[2..], 16)
                        .map_err(|_| format!("Invalid hex start value: {start_str}"))?
                } else {
                    start_str
                        .parse::<u64>()
                        .map_err(|_| format!("Invalid decimal start value: {start_str}"))?
                };

                // 解析结束值
                let end = if end_str.starts_with("0x") || end_str.starts_with("0X") {
                    u64::from_str_radix(&end_str[2..], 16)
                        .map_err(|_| format!("Invalid hex end value: {end_str}"))?
                } else {
                    end_str
                        .parse::<u64>()
                        .map_err(|_| format!("Invalid decimal end value: {end_str}"))?
                };

                Ok(apdl_core::Constraint::Range(start, end))
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
                            .map_err(|_| format!("Invalid hex enum value: {value_str}"))?
                    } else {
                        value_str
                            .parse::<u64>()
                            .map_err(|_| format!("Invalid decimal enum value: {value_str}"))?
                    };
                    enums.push((pair_parts[0].trim().to_string(), value));
                } else {
                    return Err("Invalid enum format".to_string());
                }
            }
            Ok(apdl_core::Constraint::Enum(enums))
        } else {
            Ok(apdl_core::Constraint::Custom(constraint_str.to_string()))
        }
    }

    /// 解析算法
    fn parse_algorithm(alg_str: &str) -> Result<apdl_core::AlgorithmAst, String> {
        let alg_str = alg_str.trim();
        match alg_str {
            "crc16" => Ok(apdl_core::AlgorithmAst::Crc16),
            "crc32" => Ok(apdl_core::AlgorithmAst::Crc32),
            "crc15" => Ok(apdl_core::AlgorithmAst::Crc15), // CAN协议专用
            "xor_sum" => Ok(apdl_core::AlgorithmAst::XorSum),
            _ => Ok(apdl_core::AlgorithmAst::Custom(alg_str.to_string())),
        }
    }

    /// 分割语法单元定义
    fn split_unit_definitions(content: &str) -> Vec<&str> {
        let mut defs = Vec::new();
        let mut brace_count = 0;
        let mut in_string = false;
        let mut start = 0;

        for (i, c) in content.char_indices() {
            match c {
                '"' => {
                    in_string = !in_string;
                }
                '{' if !in_string => {
                    brace_count += 1;
                }
                '}' if !in_string => {
                    brace_count -= 1;
                }
                ',' if !in_string && brace_count == 0 => {
                    defs.push(content[start..i].trim());
                    start = i + 1;
                }
                _ => {}
            }
        }

        // 添加最后一个定义
        if start < content.len() {
            defs.push(content[start..].trim());
        }

        defs
    }

    /// 提取花括号内容
    fn extract_braced_content(text: &str) -> Result<&str, String> {
        if !text.starts_with('{') {
            return Err("Content must start with {".to_string());
        }

        let mut brace_count = 0;
        let mut in_string = false;
        let mut escape_next = false;
        let mut end_pos = 0;

        for (i, c) in text.char_indices() {
            if escape_next {
                escape_next = false;
                continue;
            }

            match c {
                '\\' if in_string => {
                    escape_next = true;
                }
                '"' => {
                    in_string = !in_string;
                }
                '{' if !in_string => {
                    brace_count += 1;
                }
                '}' if !in_string => {
                    brace_count -= 1;
                    if brace_count == 0 {
                        end_pos = i + 1;
                        break;
                    }
                }
                _ => {}
            }
        }

        if brace_count != 0 {
            return Err("Unmatched braces".to_string());
        }

        Ok(&text[1..end_pos - 1])
    }

    /// 从数组格式中提取花括号内容
    fn extract_braced_content_from_array(array_str: &str) -> Result<String, String> {
        // 去除开头的 "layers:", "units:" 或 "rules:" 等
        let content = array_str.trim();
        let content_after_colon = if let Some(colon_pos) = content.find(':') {
            content[colon_pos + 1..].trim()
        } else {
            content
        };

        // 现在处理数组内容，可能包含 [ { ... }, { ... } ] 这样的结构
        if content_after_colon.starts_with('[') {
            // 找到匹配的方括号
            let mut bracket_count = 0;
            let mut in_string = false;
            let mut escape_next = false;
            let mut start_pos = None;
            let mut end_pos = 0;

            for (i, c) in content_after_colon.char_indices() {
                if escape_next {
                    escape_next = false;
                    continue;
                }

                match c {
                    '\\' if in_string => escape_next = true,
                    '"' => in_string = !in_string,
                    '[' if !in_string => {
                        if bracket_count == 0 {
                            start_pos = Some(i + 1); // 跳过左方括号
                        }
                        bracket_count += 1;
                    }
                    ']' if !in_string => {
                        bracket_count -= 1;
                        if bracket_count == 0 {
                            end_pos = i;
                            break;
                        }
                    }
                    _ => {}
                }
            }

            if bracket_count != 0 {
                return Err("Unmatched brackets in array".to_string());
            }

            if let Some(start) = start_pos {
                Ok(content_after_colon[start..end_pos].to_string())
            } else {
                Err("Could not find array content".to_string())
            }
        } else {
            // 如果不是数组格式，直接返回内容
            Ok(content_after_colon.to_string())
        }
    }

    /// 分割层定义
    fn split_layer_definitions(content: &str) -> Vec<&str> {
        let mut defs = Vec::new();
        let mut brace_count = 0;
        let mut in_string = false;
        let mut start = 0;
        let mut _last_comma = 0;

        for (i, c) in content.char_indices() {
            match c {
                '"' => {
                    in_string = !in_string;
                }
                '{' if !in_string => {
                    brace_count += 1;
                }
                '}' if !in_string => {
                    brace_count -= 1;
                }
                ',' if !in_string && brace_count == 0 => {
                    defs.push(content[start..i].trim());
                    start = i + 1;
                    _last_comma = i;
                }
                _ => {}
            }
        }

        // 添加最后一个定义
        if start < content.len() {
            defs.push(content[start..].trim());
        }

        defs
    }

    /// 提取引号值
    fn extract_quoted_value(line: &str) -> Result<String, String> {
        let colon_pos = line.find(':').ok_or("Missing colon in line")?;

        let value_part = line[colon_pos + 1..].trim();

        // 查找引号包围的值
        if let Some(start) = value_part.find('"') {
            if let Some(end) = value_part[start + 1..].find('"') {
                let quoted_content = &value_part[start + 1..start + 1 + end];
                // 检查引号后面是否有分号或其他内容
                let final_value = if start + 1 + end + 1 < value_part.len() {
                    // 引号后还有内容，可能是分号
                    let after_quote = &value_part[start + 1 + end + 1..];
                    if after_quote.find(';').is_some() {
                        // 在分号处分割
                        &quoted_content
                    } else {
                        // 没有分号，直接返回引号内容
                        quoted_content
                    }
                } else {
                    // 没有更多内容，直接返回引号内容
                    quoted_content
                };
                Ok(final_value.trim().to_string())
            } else {
                Err("Unmatched quote".to_string())
            }
        } else {
            Err("No quoted value found".to_string())
        }
    }

    /// 提取简单值（可能包含引号）
    fn extract_simple_value(line: &str) -> Result<String, String> {
        let colon_pos = line.find(':').ok_or("Missing colon in line")?;

        let mut value_part = line[colon_pos + 1..].trim();

        // 先移除分号及之后的内容
        if let Some(pos) = value_part.find(';') {
            value_part = &value_part[..pos];
        }
        value_part = value_part.trim();

        // 检查是否是带引号的值
        if value_part.starts_with('"') && value_part.ends_with('"') && value_part.len() > 1 {
            // 去掉首尾引号
            Ok(value_part[1..value_part.len() - 1].to_string())
        } else {
            Ok(value_part.to_string())
        }
    }

    /// 提取数组内容
    fn extract_array_content(line: &str) -> Result<String, String> {
        let colon_pos = line.find(':').ok_or("Missing colon in line")?;

        let value_part = line[colon_pos + 1..].trim();
        Ok(value_part.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_basic_package() {
        let dsl = r#"
        package test_package {
            name: "Test Package";
            type: "telemetry";
            desc: "A test package definition";
        }
        "#;

        let result = PackageParser::parse_package_definition(dsl);
        assert!(result.is_ok());

        let pkg = result.unwrap();
        assert_eq!(pkg.name, "test_package");
        assert_eq!(pkg.display_name, "Test Package");
        assert_eq!(pkg.package_type, "telemetry");
        assert_eq!(pkg.description, "A test package definition");
    }
}
