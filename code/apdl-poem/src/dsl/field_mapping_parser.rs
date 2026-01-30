//! 字段映射解析器模块
//!
//! 专门处理field_mapping语义规则的解析

use apdl_core::{FieldMappingEntry, SemanticRule};

/// 字段映射解析器
pub struct FieldMappingParser;

impl FieldMappingParser {
    /// 解析字段映射规则
    pub fn parse_field_mapping_rule(params: &str) -> Result<SemanticRule, String> {
        let params = params.trim();
        let mut source_package = String::new();
        let mut target_package = String::new();
        let mut mappings_str = String::new();
        let mut description = String::new();

        if params.contains("source_package:")
            && params.contains("target_package:")
            && params.contains("mappings:")
        {
            if let Some(src_start) = params.find("source_package:") {
                if let Some(semi_pos) = params[src_start..].find(';').map(|p| p + src_start) {
                    source_package = params[src_start + 15..semi_pos]
                        .trim()
                        .to_string()
                        .trim_matches('"')
                        .to_string();
                }
            }

            if let Some(tgt_start) = params.find("target_package:") {
                let remaining = &params[tgt_start + 15..];
                if let Some(semi_pos) = remaining.find(';').map(|p| p + tgt_start + 15) {
                    target_package = remaining[..semi_pos - (tgt_start + 15)]
                        .trim()
                        .to_string()
                        .trim_matches('"')
                        .to_string();
                } else {
                    target_package = remaining.trim().to_string().trim_matches('"').to_string();
                }
            }

            if let Some(map_start) = params.find("mappings:") {
                let remaining = &params[map_start + 9..];
                if let Some(semi_pos) = remaining.find(';').map(|p| p + map_start + 9) {
                    mappings_str = remaining[..semi_pos - (map_start + 9)].trim().to_string();
                } else {
                    mappings_str = remaining.trim().to_string();
                }
            }

            if let Some(desc_start) = params.find("desc:") {
                description = params[desc_start + 5..]
                    .trim()
                    .trim_matches('"')
                    .to_string();
            }
        }

        // 解析映射条目列表
        let mappings = Self::parse_field_mappings(&mappings_str)?;

        Ok(SemanticRule::FieldMapping {
            source_package,
            target_package,
            mappings,
            description,
        })
    }

    /// 解析映射条目
    pub fn parse_field_mappings(
        mappings_str: &str,
    ) -> Result<Vec<apdl_core::FieldMappingEntry>, String> {
        let mut mappings = Vec::new();

        // 首先去掉方括号
        let clean_str = mappings_str.trim();
        let content = if clean_str.starts_with('[') && clean_str.ends_with(']') {
            &clean_str[1..clean_str.len() - 1] // 去掉首尾的方括号
        } else {
            clean_str // 如果没有方括号，就使用原始内容
        };

        // 分割每个映射对象
        let mapping_objects = Self::split_mapping_objects(content);

        for obj_str in mapping_objects {
            let obj_str = obj_str.trim();
            if obj_str.starts_with('{') && obj_str.ends_with('}') {
                let content = &obj_str[1..obj_str.len() - 1]; // 去掉大括号

                let mut source_field = String::new();
                let mut target_field = String::new();
                let mut mapping_logic = String::new();
                let mut default_value = String::new();

                for pair in content.split(",").map(|s| s.trim()) {
                    let kv: Vec<&str> = pair.splitn(2, ':').map(|s| s.trim()).collect();
                    if kv.len() == 2 {
                        let key = kv[0].trim().trim_matches(|c| c == '"');
                        let value = kv[1]
                            .trim()
                            .trim_matches(|c| c == '"')
                            .trim_matches(|c| c == '"');

                        match key {
                            "source_field" => source_field = value.to_string(),
                            "target_field" => target_field = value.to_string(),
                            "mapping_logic" => mapping_logic = value.to_string(),
                            "default_value" => default_value = value.to_string(),
                            _ => {}
                        }
                    }
                }

                if !source_field.is_empty() && !target_field.is_empty() {
                    mappings.push(apdl_core::FieldMappingEntry {
                        source_field,
                        target_field,
                        mapping_logic,
                        default_value,
                    });
                }
            }
        }

        Ok(mappings)
    }

    /// 分割映射对象
    pub fn split_mapping_objects(content: &str) -> Vec<String> {
        let mut objects = Vec::new();
        let mut brace_count = 0;
        let mut current_object = String::new();
        let mut in_quotes = false;
        let mut quote_char = '"';

        for c in content.chars() {
            match c {
                '"' | '\'' => {
                    if !in_quotes {
                        in_quotes = true;
                        quote_char = c;
                    } else if c == quote_char {
                        in_quotes = false;
                    }
                    current_object.push(c);
                }
                '{' if !in_quotes => {
                    brace_count += 1;
                    current_object.push(c);
                }
                '}' if !in_quotes => {
                    brace_count -= 1;
                    current_object.push(c);

                    // 当括号平衡且到达对象结尾时，保存当前对象
                    if brace_count == 0 && current_object.trim_end().ends_with('}') {
                        objects.push(current_object.clone());
                        current_object.clear();
                    }
                }
                ',' if !in_quotes && brace_count == 0 => {
                    // 在顶层遇到逗号，说明是对象间的分隔符
                    if !current_object.trim().is_empty() {
                        objects.push(current_object.clone());
                        current_object.clear();
                    }
                }
                _ => {
                    current_object.push(c);
                }
            }
        }

        // 添加最后一个对象（如果存在）
        if !current_object.trim().is_empty() {
            objects.push(current_object);
        }

        objects
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_field_mapping_rule() {
        let params = r#"source_package: "lower_layer_packet"; target_package: "upper_layer_packet"; mappings: [{source_field: "src_id", target_field: "vcid", mapping_logic: "hash_mod_64", default_value: "0"}]; desc: "Map source ID to VCID""#;

        let result = FieldMappingParser::parse_field_mapping_rule(params);
        assert!(result.is_ok());

        if let Ok(SemanticRule::FieldMapping {
            source_package,
            target_package,
            mappings,
            description,
        }) = result
        {
            assert_eq!(source_package, "lower_layer_packet");
            assert_eq!(target_package, "upper_layer_packet");
            assert_eq!(description, "Map source ID to VCID");
            assert_eq!(mappings.len(), 1);

            let mapping = &mappings[0];
            assert_eq!(mapping.source_field, "src_id");
            assert_eq!(mapping.target_field, "vcid");
            assert_eq!(mapping.mapping_logic, "hash_mod_64");
            assert_eq!(mapping.default_value, "0");
        } else {
            panic!("Expected FieldMapping rule");
        }
    }
}
