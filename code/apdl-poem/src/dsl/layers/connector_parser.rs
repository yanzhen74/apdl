//! 连接器解析器模块
//!
//! 处理连接器定义的解析

use apdl_core::{ConnectorConfig, ConnectorDefinition, FieldMappingEntry, HeaderPointerConfig};

/// 连接器解析器
pub struct ConnectorParser;

impl ConnectorParser {
    /// 解析连接器定义
    pub fn parse_connector_definition(dsl_text: &str) -> Result<ConnectorDefinition, String> {
        let dsl_text = dsl_text.trim();

        // 提取连接器名
        let connector_def_start = "connector ";
        if !dsl_text.starts_with(connector_def_start) {
            return Err("Not a connector definition".to_string());
        }

        let after_connector = &dsl_text[connector_def_start.len()..].trim_start();

        // 查找连接器名结束位置（空格或左花括号）
        let mut connector_name_end = 0;
        for (i, c) in after_connector.chars().enumerate() {
            if c.is_whitespace() || c == '{' {
                connector_name_end = i;
                break;
            }
        }

        let connector_name = after_connector[..connector_name_end].trim();
        let remaining = &after_connector[connector_name_end..].trim_start();

        // 确保以左花括号开始
        if !remaining.starts_with('{') {
            return Err("Connector definition must start with {".to_string());
        }

        // 找到匹配的右花括号
        let content = Self::extract_braced_content(remaining)?;

        // 解析连接器定义内容
        let mut connector_type = String::new();
        let mut source_package = String::new();
        let mut target_package = String::new();
        let mut description = String::new();
        let mut mappings_str = String::new();
        let mut header_pointers_str = String::new();

        // 解析各个字段
        let mut lines = content.lines().peekable();

        while let Some(line) = lines.next() {
            let line = line.trim();
            if line.is_empty() || line.starts_with("//") {
                continue;
            }

            // 移除行末的分号后再进行匹配
            let line_no_semicolon = line.trim_end().trim_end_matches(';');

            if line_no_semicolon.trim_start().starts_with("type:") {
                connector_type = Self::extract_simple_value(line)?;
            } else if line_no_semicolon
                .trim_start()
                .starts_with("source_package:")
            {
                source_package = Self::extract_quoted_value(line)?;
            } else if line_no_semicolon
                .trim_start()
                .starts_with("target_package:")
            {
                target_package = Self::extract_quoted_value(line)?;
            } else if line_no_semicolon.trim_start().starts_with("desc:") {
                description = Self::extract_quoted_value(line)?;
            } else if line_no_semicolon.trim_start().starts_with("config:") {
                // config 后面跟着一个花括号块，需要收集完整的内容
                let mut config_block = String::from(line);
                let mut brace_count = 0;

                // 计算当前行的花括号数量
                let mut in_string = false;
                let mut escape_next = false;

                for c in line.chars() {
                    if escape_next {
                        escape_next = false;
                        continue;
                    }

                    match c {
                        '\\' if in_string => escape_next = true,
                        '"' => in_string = !in_string,
                        '{' if !in_string => brace_count += 1,
                        '}' if !in_string => {
                            brace_count -= 1;
                            if brace_count == 0 {
                                // 找到了平衡的括号，跳出循环
                                break;
                            }
                        }
                        _ => {}
                    }
                }

                // 继续收集行直到括号平衡
                while brace_count > 0 {
                    if let Some(next_line) = lines.next() {
                        let next_trimmed = next_line;
                        config_block.push('\n');
                        config_block.push_str(next_line);

                        // 更新括号计数
                        let mut in_string = false;
                        let mut escape_next = false;

                        for c in next_trimmed.chars() {
                            if escape_next {
                                escape_next = false;
                                continue;
                            }

                            match c {
                                '\\' if in_string => escape_next = true,
                                '"' => in_string = !in_string,
                                '{' if !in_string => brace_count += 1,
                                '}' if !in_string => brace_count -= 1,
                                _ => {}
                            }
                        }
                    } else {
                        return Err("Unmatched braces in config section".to_string());
                    }
                }

                // 简化处理：直接提取 config 块的内容
                // 需要找到第一个 { 的位置，因为 config_block 包含了整行内容
                let content_without_prefix = config_block.trim_start();
                let start_brace_pos = content_without_prefix.find('{');

                let content_to_parse = if let Some(pos) = start_brace_pos {
                    &content_without_prefix[pos..]
                } else {
                    return Err("Could not find opening brace in config block".to_string());
                };

                let config_content = Self::extract_braced_content(content_to_parse)?;

                // 为正确处理跨多行的数组，我们需要从 config_content 整体中查找
                // 而不是逐行处理，因为 mappings: [ 可能跨越多行

                // 查找 mappings 数组
                if let Some(mappings_pos) = config_content.find("mappings:") {
                    // 从 mappings: 位置开始寻找完整的数组
                    let from_mappings = &config_content[mappings_pos..];

                    let mut bracket_count = 0;
                    let mut in_string = false;
                    let mut escape_next = false;
                    let mut array_start = None;
                    let mut array_end = None;

                    // 跳过 mappings: 部分，找到第一个 [
                    if let Some(bracket_pos) = from_mappings.find('[') {
                        for (i, c) in from_mappings[bracket_pos..].char_indices() {
                            let actual_i = i + bracket_pos;
                            if escape_next {
                                escape_next = false;
                                continue;
                            }

                            match c {
                                '\\' if in_string => escape_next = true,
                                '"' => in_string = !in_string,
                                '[' if !in_string => {
                                    if bracket_count == 0 {
                                        array_start = Some(actual_i + 1); // 跳过 '['
                                    }
                                    bracket_count += 1;
                                }
                                ']' if !in_string => {
                                    bracket_count -= 1;
                                    if bracket_count == 0 {
                                        array_end = Some(actual_i);
                                        break; // 找到匹配的 ']'
                                    }
                                }
                                _ => {}
                            }
                        }
                    }

                    if let (Some(start), Some(end)) = (array_start, array_end) {
                        mappings_str = from_mappings[start..end].to_string();
                    } else {
                        return Err("Could not extract mappings content".to_string());
                    }
                }

                // 查找 header_pointers 对象（如果存在）
                if let Some(hp_pos) = config_content.find("header_pointers:") {
                    let from_hp = &config_content[hp_pos..];

                    let mut brace_count = 0;
                    let mut in_string = false;
                    let mut escape_next = false;
                    let mut obj_start = None;
                    let mut obj_end = None;

                    // 跳过 header_pointers: 部分，找到第一个 {
                    if let Some(brace_pos) = from_hp.find('{') {
                        for (i, c) in from_hp[brace_pos..].char_indices() {
                            let actual_i = i + brace_pos;
                            if escape_next {
                                escape_next = false;
                                continue;
                            }

                            match c {
                                '\\' if in_string => escape_next = true,
                                '"' => in_string = !in_string,
                                '{' if !in_string => {
                                    if brace_count == 0 {
                                        obj_start = Some(actual_i + 1); // 跳过 '{'
                                    }
                                    brace_count += 1;
                                }
                                '}' if !in_string => {
                                    brace_count -= 1;
                                    if brace_count == 0 {
                                        obj_end = Some(actual_i);
                                        break; // 找到匹配的 '}'
                                    }
                                }
                                _ => {}
                            }
                        }
                    }

                    if let (Some(start), Some(end)) = (obj_start, obj_end) {
                        header_pointers_str = from_hp[start..end].to_string();
                    }
                }
            }
        }

        // 解析映射
        let mappings = if !mappings_str.is_empty() {
            Self::parse_mappings(&mappings_str)?
        } else {
            Vec::new()
        };

        // 解析导头指针配置
        let header_pointers = if !header_pointers_str.is_empty() {
            Some(Self::parse_header_pointers(&header_pointers_str)?)
        } else {
            None
        };

        let config = ConnectorConfig {
            mappings,
            header_pointers,
        };

        // 设置默认值
        let connector_type = if connector_type.is_empty() {
            "field_mapping".to_string()
        } else {
            connector_type
        };

        Ok(ConnectorDefinition {
            name: connector_name.to_string(),
            connector_type,
            source_package,
            target_package,
            config,
            description,
        })
    }

    /// 解析映射条目列表
    fn parse_mappings(mappings_content: &str) -> Result<Vec<FieldMappingEntry>, String> {
        let mut mappings = Vec::new();

        // mappings_content 已经是提取出的数组内容（不包含方括号）
        let mappings_array_content = mappings_content;

        // 按映射条目分割
        let mapping_defs = Self::split_mapping_definitions(mappings_array_content);

        for mapping_def in mapping_defs {
            let mapping = Self::parse_single_mapping(mapping_def)?;
            mappings.push(mapping);
        }

        Ok(mappings)
    }

    /// 解析单个映射条目
    fn parse_single_mapping(mapping_content: &str) -> Result<FieldMappingEntry, String> {
        let mut source_field = String::new();
        let mut target_field = String::new();
        let mut mapping_logic = String::new();
        let mut default_value = String::new();

        for line in mapping_content.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with("//") {
                continue;
            }

            // 移除行末的分号后再进行匹配
            let line_no_semicolon = line.trim_end().trim_end_matches(';');

            if line_no_semicolon.contains("source_field:") {
                source_field = Self::extract_quoted_value(line)?;
            } else if line_no_semicolon.contains("target_field:") {
                target_field = Self::extract_quoted_value(line)?;
            } else if line_no_semicolon.contains("logic:") {
                mapping_logic = Self::extract_quoted_value(line)?;
            } else if line_no_semicolon.contains("default_value:") {
                default_value = Self::extract_quoted_value(line)?;
            }
        }

        Ok(FieldMappingEntry {
            source_field: if source_field.is_empty() {
                "unknown".to_string()
            } else {
                source_field
            },
            target_field: if target_field.is_empty() {
                "unknown".to_string()
            } else {
                target_field
            },
            mapping_logic: if mapping_logic.is_empty() {
                "identity".to_string()
            } else {
                mapping_logic
            },
            default_value,
            enum_mappings: None,
        })
    }

    /// 解析导头指针配置
    fn parse_header_pointers(pointers_content: &str) -> Result<HeaderPointerConfig, String> {
        let mut master_pointer = String::new();
        let mut secondary_pointers = Vec::new();
        let mut descriptor_field = String::new();

        for line in pointers_content.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with("//") {
                continue;
            }

            // 移除行末的分号后再进行匹配
            let line_no_semicolon = line.trim_end().trim_end_matches(';');

            if line_no_semicolon.starts_with("master_pointer:") {
                master_pointer = Self::extract_quoted_value(line)?;
            } else if line_no_semicolon.starts_with("secondary_pointers:") {
                let array_content = Self::extract_array_content(line)?;
                secondary_pointers = Self::parse_secondary_pointers(&array_content)?;
            } else if line_no_semicolon.starts_with("descriptor_field:") {
                descriptor_field = Self::extract_quoted_value(line)?;
            }
        }

        Ok(HeaderPointerConfig {
            master_pointer: if master_pointer.is_empty() {
                "unknown".to_string()
            } else {
                master_pointer
            },
            secondary_pointers,
            descriptor_field: if descriptor_field.is_empty() {
                "unknown".to_string()
            } else {
                descriptor_field
            },
        })
    }

    /// 解析副导头指针数组
    fn parse_secondary_pointers(array_content: &str) -> Result<Vec<String>, String> {
        let mut pointers = Vec::new();

        // 提取数组内容并分割字符串
        let content = Self::extract_bracket_content_from_array(array_content)?;

        // 按逗号分割并去除引号
        for item in content.split(',') {
            let item = item.trim();
            if item.starts_with('"') && item.ends_with('"') {
                let value = item[1..item.len() - 1].trim();
                pointers.push(value.to_string());
            }
        }

        Ok(pointers)
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

    /// 从数组格式中提取方括号内容
    fn extract_bracket_content_from_array(array_str: &str) -> Result<&str, String> {
        // 跳过开始的 "["
        let start_pos = if array_str.trim_start().starts_with('[') {
            array_str.find('[').unwrap() + 1
        } else {
            return Err("Array must start with [".to_string());
        };

        let content = &array_str[start_pos..].trim_start();

        // 查找对应的右方括号，注意处理引号内的方括号
        let mut bracket_count = 1; // 开始时有一个左方括号
        let mut in_string = false;
        let mut escape_next = false;
        let mut end_pos = 0;

        for (i, c) in content.char_indices() {
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
                '[' if !in_string => {
                    bracket_count += 1;
                }
                ']' if !in_string => {
                    bracket_count -= 1;
                    if bracket_count == 0 {
                        // 找到匹配的右方括号
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

        Ok(&content[..end_pos])
    }

    /// 分割映射定义
    fn split_mapping_definitions(content: &str) -> Vec<&str> {
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
                    if brace_count == 0 {
                        start = i; // 记录开始位置
                    }
                    brace_count += 1;
                }
                '}' if !in_string => {
                    brace_count -= 1;
                    if brace_count == 0 {
                        defs.push(&content[start..i + 1]);
                    }
                }
                _ => {}
            }
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
    fn test_parse_basic_connector() {
        let dsl = r#"
        connector test_connector {
            type: "field_mapping";
            source_package: "telemetry_packet";
            target_package: "encapsulating_packet";
            config: {
                mappings: [
                    {
                        source_field: "tlm_source_id",
                        target_field: "apid",
                        logic: "hash(tlm_source_id) % 2048",
                        default_value: "0"
                    }
                ];
            };
            desc: "Map telemetry to encapsulating packet";
        }
        "#;

        let result = ConnectorParser::parse_connector_definition(dsl);
        if result.is_err() {
            println!("Error parsing connector: {:?}", result.as_ref().err());
        }
        assert!(
            result.is_ok(),
            "Failed to parse connector: {:?}",
            result.as_ref().err()
        );

        let conn = result.unwrap();
        assert_eq!(conn.name, "test_connector");
        assert_eq!(conn.connector_type, "field_mapping");
        assert_eq!(conn.source_package, "telemetry_packet");
        assert_eq!(conn.target_package, "encapsulating_packet");
        assert_eq!(conn.description, "Map telemetry to encapsulating packet");
        assert_eq!(conn.config.mappings.len(), 1);
        assert_eq!(conn.config.mappings[0].source_field, "tlm_source_id");
        assert_eq!(conn.config.mappings[0].target_field, "apid");
    }
}
