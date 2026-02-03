//! 连接器定义解析器
//!
//! 用于解析连接器定义的DSL文本

use apdl_core::{
    ConnectorConfig, ConnectorDefinition, DataPlacementConfig, DataPlacementStrategy,
    FieldMappingEntry, HeaderPointerConfig,
};
use std::collections::HashMap;

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

        // 解析属性
        let properties = Self::parse_properties(content)?;

        // 提取必需属性
        let connector_type = properties
            .get("type")
            .ok_or("Missing 'type' property")?
            .clone();
        let source_package = properties
            .get("source_package")
            .ok_or("Missing 'source_package' property")?
            .clone();
        let target_package = properties
            .get("target_package")
            .ok_or("Missing 'target_package' property")?
            .clone();

        // 提取可选属性
        let description = properties
            .get("desc")
            .unwrap_or(&"No description".to_string())
            .clone();

        // 解析配置部分
        let config_content = properties.get("config").ok_or("Missing 'config' section")?;
        //println!("DEBUG: config_content: {}", config_content);

        let config_obj = Self::parse_object(config_content.as_str())?;
        //println!("DEBUG: config_obj keys: {:?}", config_obj.keys());
        //if let Some(mappings_val) = config_obj.get("mappings") {
        //    println!("DEBUG: mappings value: '{}'", mappings_val);
        //}

        // 解析映射规则
        let mappings = if let Some(mappings_str) = config_obj.get("mappings") {
            //println!("DEBUG: Found mappings string: '{}'", mappings_str);
            let parsed_mappings = Self::parse_mappings(mappings_str)?;
            //println!("DEBUG: Parsed {} mappings", parsed_mappings.len());
            parsed_mappings
        } else {
            //println!("DEBUG: No mappings found in config_obj");
            Vec::new()
        };

        // 解析头部指针配置
        let header_pointers = if let Some(header_ptrs_str) = config_obj.get("header_pointers") {
            Some(Self::parse_header_pointers(header_ptrs_str)?)
        } else {
            None
        };

        // 解析数据放置配置
        let data_placement = if let Some(placement_str) = config_obj.get("placement_strategy") {
            Some(Self::parse_data_placement(placement_str)?)
        } else {
            None
        };

        let config = ConnectorConfig {
            mappings,
            header_pointers,
            data_placement,
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

    /// 解析属性列表
    fn parse_properties(text: &str) -> Result<HashMap<String, String>, String> {
        let mut properties = HashMap::new();
        let mut current_key = String::new();
        let mut current_value = String::new();
        let mut in_key = true;
        let mut in_value = false;
        let mut in_string = false;
        let mut escape_next = false;
        let mut brace_depth = 0;
        let mut bracket_depth = 0;
        let mut paren_depth = 0;
        let mut last_char_was_colon = false;

        let mut chars = text.chars().peekable();
        while let Some(c) = chars.next() {
            if escape_next {
                if in_value {
                    current_value.push(c);
                } else if in_key {
                    current_key.push(c);
                }
                escape_next = false;
                continue;
            }

            match c {
                '\\' if in_string => {
                    escape_next = true;
                    if in_value {
                        current_value.push(c);
                    } else if in_key {
                        current_key.push(c);
                    }
                    continue;
                }
                '"' => {
                    if in_value {
                        // 检查是否是字符串的开始或结束
                        if !in_string {
                            // 开始字符串
                            in_string = true;
                        } else {
                            // 结束字符串
                            in_string = false;
                        }
                        // 不将引号添加到值中，除非它是转义的
                        // （我们已经处理了转义情况）
                    } else if in_key {
                        current_key.push(c);
                    }
                }
                '{' if !in_string => {
                    brace_depth += 1;
                    if in_value {
                        current_value.push(c);
                    } else if in_key {
                        current_key.push(c);
                    }
                }
                '}' if !in_string => {
                    if brace_depth > 0 {
                        brace_depth -= 1;
                    }
                    if in_value {
                        current_value.push(c);
                    } else if in_key {
                        current_key.push(c);
                    }
                }
                '[' if !in_string => {
                    bracket_depth += 1;
                    if in_value {
                        current_value.push(c);
                    } else if in_key {
                        current_key.push(c);
                    }
                }
                ']' if !in_string => {
                    if bracket_depth > 0 {
                        bracket_depth -= 1;
                    }
                    if in_value {
                        current_value.push(c);
                    } else if in_key {
                        current_key.push(c);
                    }
                }
                '(' if !in_string => {
                    paren_depth += 1;
                    if in_value {
                        current_value.push(c);
                    } else if in_key {
                        current_key.push(c);
                    }
                }
                ')' if !in_string => {
                    if paren_depth > 0 {
                        paren_depth -= 1;
                    }
                    if in_value {
                        current_value.push(c);
                    } else if in_key {
                        current_key.push(c);
                    }
                }
                ':' if !in_string && brace_depth == 0 && bracket_depth == 0 && paren_depth == 0 => {
                    if in_key {
                        in_key = false;
                        in_value = true;
                        last_char_was_colon = true;
                    } else if in_value {
                        // 如果在值中又遇到冒号，且不在字符串中且外部无嵌套，则添加到值中
                        current_value.push(c);
                    }
                    continue;
                }
                ';' if !in_string && brace_depth == 0 && bracket_depth == 0 && paren_depth == 0 => {
                    if in_value && !current_key.is_empty() && !current_value.is_empty() {
                        // 对值进行处理：去除可能的外部引号
                        let processed_value = Self::process_value_string(current_value.trim());
                        properties.insert(current_key.trim().to_string(), processed_value);
                        current_key.clear();
                        current_value.clear();
                        in_key = true;
                        in_value = false;
                        in_string = false; // 重置字符串状态
                    }
                    continue;
                }
                ',' if !in_string && brace_depth == 0 && bracket_depth == 0 && paren_depth == 0 => {
                    if in_value && !current_key.is_empty() && !current_value.is_empty() {
                        // 对值进行处理：去除可能的外部引号
                        let processed_value = Self::process_value_string(current_value.trim());
                        properties.insert(current_key.trim().to_string(), processed_value);
                        current_key.clear();
                        current_value.clear();
                        in_key = true;
                        in_value = false;
                        in_string = false; // 重置字符串状态
                    }
                    continue;
                }
                _ => {
                    if in_key && !last_char_was_colon {
                        current_key.push(c);
                    } else if in_value {
                        current_value.push(c);
                    }
                }
            }

            if last_char_was_colon && !c.is_whitespace() {
                last_char_was_colon = false;
            }
        }

        // 添加最后一个属性
        if !current_key.is_empty() && !current_value.is_empty() {
            let processed_value = Self::process_value_string(current_value.trim());
            properties.insert(current_key.trim().to_string(), processed_value);
        }

        Ok(properties)
    }

    /// 处理值字符串，去除外部引号（如果存在）
    fn process_value_string(value: &str) -> String {
        let trimmed = value.trim();

        // 检查是否是字符串字面量（以引号开头和结尾）
        if trimmed.len() >= 2 && trimmed.starts_with('"') && trimmed.ends_with('"') {
            // 去除外部引号
            trimmed[1..trimmed.len() - 1].to_string()
        } else if trimmed.len() >= 2 && trimmed.starts_with('\'') && trimmed.ends_with('\'') {
            // 处理单引号字符串
            trimmed[1..trimmed.len() - 1].to_string()
        } else {
            // 不是字符串字面量，返回原值
            trimmed.to_string()
        }
    }

    /// 解析对象（花括号内容）
    fn parse_object(obj_text: &str) -> Result<HashMap<String, String>, String> {
        let content = Self::extract_braced_content(obj_text)?;
        Self::parse_properties(content)
    }

    /// 解析映射规则数组
    fn parse_mappings(mappings_text: &str) -> Result<Vec<FieldMappingEntry>, String> {
        // 检查是否是数组格式 [ ... ]
        let trimmed = mappings_text.trim();

        if trimmed.starts_with('[') && trimmed.ends_with(']') {
            // 提取方括号内容
            let content = Self::extract_array_content(trimmed)?;
            Self::parse_mappings_from_content(&content)
        } else if trimmed.starts_with('{') && trimmed.ends_with('}') {
            // 如果是花括号格式，提取内容并解析
            let content = Self::extract_braced_content(trimmed)?;
            Self::parse_mappings_from_content(&content)
        } else {
            // 尝试两种方式提取内容
            if let Ok(content) = Self::extract_array_content(trimmed) {
                Self::parse_mappings_from_content(&content)
            } else if let Ok(content) = Self::extract_braced_content(trimmed) {
                Self::parse_mappings_from_content(&content)
            } else {
                // 如果都不是，则直接解析原始内容
                Self::parse_mappings_from_content(trimmed)
            }
        }
    }

    /// 从内容解析映射规则
    fn parse_mappings_from_content(content: &str) -> Result<Vec<FieldMappingEntry>, String> {
        let mut mappings = Vec::new();

        // 解析数组内容
        let items = Self::parse_array_items(content)?;

        for item in items {
            // 检查item是否已经是用大括号包围的对象格式
            let clean_item = item.trim();
            if clean_item.is_empty() {
                continue; // 跳过空项
            }

            // 尝试解析为对象
            let obj_props = if clean_item.starts_with('{') && clean_item.ends_with('}') {
                Self::parse_object(clean_item)?
            } else {
                // 如果不是完整的对象格式，尝试包装
                Self::parse_object(&format!("{{{}}}", clean_item))?
            };

            let source_field = obj_props
                .get("source_field")
                .ok_or("Missing 'source_field' in mapping entry")?
                .clone();
            let target_field = obj_props
                .get("target_field")
                .ok_or("Missing 'target_field' in mapping entry")?
                .clone();
            let mapping_logic = obj_props
                .get("logic")
                .ok_or("Missing 'logic' in mapping entry")?
                .clone();
            let default_value = obj_props
                .get("default_value")
                .unwrap_or(&"0".to_string())
                .clone();

            // 解析枚举映射（如果存在）
            let enum_mappings = if let Some(enum_mappings_str) = obj_props.get("enum_mappings") {
                Some(Self::parse_enum_mappings(enum_mappings_str)?)
            } else {
                None
            };

            mappings.push(FieldMappingEntry {
                source_field,
                target_field,
                mapping_logic,
                default_value,
                enum_mappings,
            });
        }

        Ok(mappings)
    }

    /// 提取方括号内容
    fn extract_array_content(text: &str) -> Result<&str, String> {
        let mut bracket_count = 0;
        let mut in_string = false;
        let mut escape_next = false;
        let mut start_pos = 0;
        let mut end_pos = 0;

        for (i, c) in text.char_indices() {
            if escape_next {
                escape_next = false;
                continue;
            }

            match c {
                '\\' if in_string => escape_next = true,
                '"' => in_string = !in_string,
                '[' if !in_string => {
                    if bracket_count == 0 {
                        start_pos = i + 1; // 跳过左方括号
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
            return Err("Unmatched brackets".to_string());
        }

        if end_pos <= start_pos {
            return Ok("");
        }

        Ok(&text[start_pos..end_pos])
    }

    /// 解析数组项
    fn parse_array_items(array_content: &str) -> Result<Vec<String>, String> {
        let mut items = Vec::new();
        let mut current_item = String::new();
        let mut brace_count = 0;
        let mut bracket_count = 0;
        let mut in_string = false;
        let mut escape_next = false;
        let mut in_item = false;

        for c in array_content.chars() {
            if escape_next {
                current_item.push(c);
                escape_next = false;
                continue;
            }

            match c {
                '\\' if in_string => {
                    escape_next = true;
                    continue;
                }
                '"' => {
                    in_string = !in_string;
                    current_item.push(c);
                }
                '{' if !in_string => {
                    brace_count += 1;
                    current_item.push(c);
                    in_item = true;
                }
                '}' if !in_string => {
                    brace_count -= 1;
                    current_item.push(c);
                    if brace_count == 0 && !in_string {
                        // 当前项结束
                        items.push(current_item.clone());
                        current_item.clear();
                        in_item = false;
                    }
                }
                '[' if !in_string => {
                    bracket_count += 1;
                    current_item.push(c);
                    in_item = true;
                }
                ']' if !in_string => {
                    bracket_count -= 1;
                    current_item.push(c);
                    if bracket_count == 0 && !in_string {
                        // 当前项结束
                        items.push(current_item.clone());
                        current_item.clear();
                        in_item = false;
                    }
                }
                ',' if !in_string && brace_count == 0 && bracket_count == 0 => {
                    if in_item {
                        current_item.push(c);
                    } else {
                        // 当前项结束（如果是简单值）
                        if !current_item.trim().is_empty() {
                            items.push(current_item.trim().to_string());
                            current_item.clear();
                        }
                    }
                }
                ';' if !in_string && brace_count == 0 && bracket_count == 0 => {
                    if in_item {
                        current_item.push(c);
                    } else {
                        // 当前项结束（如果是简单值）- 在数组中也支持分号分隔
                        if !current_item.trim().is_empty() {
                            items.push(current_item.trim().to_string());
                            current_item.clear();
                        }
                    }
                }
                _ => {
                    current_item.push(c);
                    if !c.is_whitespace() {
                        in_item = true;
                    }
                }
            }
        }

        // 添加最后一个项（如果不是空的）
        if !current_item.trim().is_empty() {
            items.push(current_item.trim().to_string());
        }

        Ok(items)
    }

    /// 解析枚举映射
    fn parse_enum_mappings(
        enum_mappings_text: &str,
    ) -> Result<Vec<apdl_core::EnumMappingEntry>, String> {
        let content = Self::extract_braced_content(enum_mappings_text)?;
        let mut mappings = Vec::new();

        // 解析数组内容
        let items = Self::parse_array_items(content)?;

        for item in items {
            let obj_props = Self::parse_object(&item)?;

            let source_enum = obj_props
                .get("source_enum")
                .ok_or("Missing 'source_enum' in enum mapping entry")?
                .clone();
            let target_enum = obj_props
                .get("target_enum")
                .ok_or("Missing 'target_enum' in enum mapping entry")?
                .clone();

            mappings.push(apdl_core::EnumMappingEntry {
                source_enum,
                target_enum,
            });
        }

        Ok(mappings)
    }

    /// 解析头部指针配置
    fn parse_header_pointers(header_ptrs_text: &str) -> Result<HeaderPointerConfig, String> {
        let content = Self::extract_braced_content(header_ptrs_text)?;
        let properties = Self::parse_properties(content)?;

        let master_pointer = properties
            .get("master_pointer")
            .unwrap_or(&"".to_string())
            .clone();

        let secondary_pointers = if let Some(ptrs_str) = properties.get("secondary_pointers") {
            Self::parse_string_array(ptrs_str)?
        } else {
            Vec::new()
        };

        let descriptor_field = properties
            .get("descriptor_field")
            .unwrap_or(&"".to_string())
            .clone();

        Ok(HeaderPointerConfig {
            master_pointer,
            secondary_pointers,
            descriptor_field,
        })
    }

    /// 解析数据放置配置
    fn parse_data_placement(placement_text: &str) -> Result<DataPlacementConfig, String> {
        let content = Self::extract_braced_content(placement_text)?;
        let properties = Self::parse_properties(content)?;

        let strategy_str = properties
            .get("strategy")
            .ok_or("Missing 'strategy' in placement config")?
            .clone();

        let target_field = properties
            .get("target_field")
            .unwrap_or(&"".to_string())
            .clone();

        // 解析策略类型
        let strategy = match strategy_str.as_str() {
            "direct" => DataPlacementStrategy::Direct,
            "pointer_based" => DataPlacementStrategy::PointerBased,
            "stream_based" => DataPlacementStrategy::StreamBased,
            custom => DataPlacementStrategy::Custom(custom.to_string()),
        };

        // 解析配置参数
        let config_params = if let Some(config_str) = properties.get("config") {
            let config_obj = Self::parse_object(config_str)?;
            config_obj.into_iter().collect()
        } else {
            Vec::new()
        };

        Ok(DataPlacementConfig {
            strategy,
            target_field,
            config_params,
        })
    }

    /// 解析字符串数组
    fn parse_string_array(array_text: &str) -> Result<Vec<String>, String> {
        let content = Self::extract_braced_content(array_text)?;
        let mut items = Vec::new();
        let mut current_item = String::new();
        let mut in_string = false;
        let mut escape_next = false;

        for c in content.chars() {
            if escape_next {
                current_item.push(c);
                escape_next = false;
                continue;
            }

            match c {
                '\\' => {
                    escape_next = true;
                    continue;
                }
                '"' => {
                    in_string = !in_string;
                    if in_string {
                        // 跳过开始引号
                        continue;
                    } else {
                        // 结束引号，添加项
                        items.push(current_item.clone());
                        current_item.clear();
                        continue;
                    }
                }
                ',' if !in_string => {
                    if !current_item.trim().is_empty() {
                        items.push(current_item.trim().to_string());
                        current_item.clear();
                    }
                    continue;
                }
                _ => {
                    if in_string || (!c.is_whitespace()) {
                        current_item.push(c);
                    }
                }
            }
        }

        // 添加最后一个项
        if !current_item.trim().is_empty() {
            items.push(current_item.trim().to_string());
        }

        Ok(items)
    }

    /// 提取花括号内容
    fn extract_braced_content(text: &str) -> Result<&str, String> {
        let mut brace_count = 0;
        let mut in_string = false;
        let mut escape_next = false;
        let mut start_pos = 0;
        let mut end_pos = 0;

        for (i, c) in text.char_indices() {
            if escape_next {
                escape_next = false;
                continue;
            }

            match c {
                '\\' if in_string => escape_next = true,
                '"' => in_string = !in_string,
                '{' if !in_string => {
                    if brace_count == 0 {
                        start_pos = i + 1; // 跳过左花括号
                    }
                    brace_count += 1;
                }
                '}' if !in_string => {
                    brace_count -= 1;
                    if brace_count == 0 {
                        end_pos = i;
                        break;
                    }
                }
                _ => {}
            }
        }

        if brace_count != 0 {
            return Err("Unmatched braces".to_string());
        }

        if end_pos <= start_pos {
            return Ok("");
        }

        Ok(&text[start_pos..end_pos])
    }

    /// 提取简单值（非对象或数组的值）
    #[allow(dead_code)]
    fn extract_simple_value(line: &str) -> Result<String, String> {
        let parts: Vec<&str> = line.split(':').collect();
        if parts.len() < 2 {
            return Err("Invalid property format".to_string());
        }

        let value = parts[1].trim();
        // 移除可能的分号
        let value = value.trim_end_matches(';').trim();

        // 如果值被引号包围，移除引号
        if value.starts_with('"') && value.ends_with('"') {
            Ok(value[1..value.len() - 1].to_string())
        } else {
            Ok(value.to_string())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_connector_definition_with_data_placement() {
        let dsl = r#"
        connector test_connector {
            type: "field_mapping";
            source_package: "source_pkt";
            target_package: "target_pkt";
            config: {
                mappings: [
                    {
                        source_field: "src_field";
                        target_field: "tgt_field";
                        logic: "identity";
                        default_value: "0";
                    }
                ];
                placement_strategy: {
                    strategy: "pointer_based";
                    target_field: "data_field";
                    config: {
                        pointer_field: "ptr_field";
                        map_id: "map_id";
                    };
                };
            };
            desc: "Test connector with data placement";
        }
        "#;

        let result = ConnectorParser::parse_connector_definition(dsl);
        assert!(
            result.is_ok(),
            "Failed to parse connector: {:?}",
            result.err()
        );

        let conn = result.unwrap();
        assert_eq!(conn.name, "test_connector");
        assert_eq!(conn.connector_type, "field_mapping");
        assert_eq!(conn.source_package, "source_pkt");
        assert_eq!(conn.target_package, "target_pkt");
        assert_eq!(conn.description, "Test connector with data placement");

        // 检查映射规则
        assert_eq!(conn.config.mappings.len(), 1);
        assert_eq!(conn.config.mappings[0].source_field, "src_field");
        assert_eq!(conn.config.mappings[0].target_field, "tgt_field");

        // 检查数据放置配置
        assert!(conn.config.data_placement.is_some());
        let placement = conn.config.data_placement.unwrap();
        match placement.strategy {
            DataPlacementStrategy::PointerBased => {}
            _ => panic!("Expected PointerBased strategy"),
        }
        assert_eq!(placement.target_field, "data_field");
        assert!(!placement.config_params.is_empty());
    }

    #[test]
    fn test_parse_connector_definition_basic() {
        let dsl = r#"
        connector test_field_mapping {
            type: "field_mapping";
            source_package: "telemetry_packet";
            target_package: "encapsulating_packet";
            config: {
                mappings: [
                    {
                        source_field: "tlm_source_id";
                        target_field: "apid";
                        logic: "hash_mod_2048";
                        default_value: "0";
                    },
                    {
                        source_field: "packet_sequence_control";
                        target_field: "sequence_count";
                        logic: "identity";
                        default_value: "1";
                    }
                ];
            };
            desc: "Test field mapping from telemetry to encapsulating packet";
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

        let connector_definition = result.unwrap();
        assert_eq!(connector_definition.name, "test_field_mapping");
        assert_eq!(connector_definition.connector_type, "field_mapping");
        assert_eq!(connector_definition.source_package, "telemetry_packet");
        assert_eq!(connector_definition.target_package, "encapsulating_packet");
        assert_eq!(
            connector_definition.description,
            "Test field mapping from telemetry to encapsulating packet"
        );

        // 验证映射规则
        assert_eq!(connector_definition.config.mappings.len(), 2);

        let first_mapping = &connector_definition.config.mappings[0];
        assert_eq!(first_mapping.source_field, "tlm_source_id");
        assert_eq!(first_mapping.target_field, "apid");
        assert_eq!(first_mapping.mapping_logic, "hash_mod_2048");
        assert_eq!(first_mapping.default_value, "0");

        let second_mapping = &connector_definition.config.mappings[1];
        assert_eq!(second_mapping.source_field, "packet_sequence_control");
        assert_eq!(second_mapping.target_field, "sequence_count");
        assert_eq!(second_mapping.mapping_logic, "identity");
        assert_eq!(second_mapping.default_value, "1");
    }
}
