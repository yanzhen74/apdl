//! JSON解析器模块
//!
//! 用于解析JSON格式的协议定义

use apdl_core::{
    ConnectorDefinition, Constraint, CoverDesc, LayerDefinition, LengthDesc, LengthUnit,
    PackageDefinition, ProtocolStackDefinition, ScopeDesc, SemanticRule, SyntaxUnit, UnitType,
};
use serde_json::Value;

/// JSON解析器
pub struct JsonParser;

impl JsonParser {
    /// 解析包定义JSON
    pub fn parse_package(json_str: &str) -> Result<PackageDefinition, String> {
        serde_json::from_str(json_str).map_err(|e| format!("Failed to parse package JSON: {e}"))
    }

    /// 解析连接器定义JSON
    pub fn parse_connector(json_str: &str) -> Result<ConnectorDefinition, String> {
        serde_json::from_str(json_str).map_err(|e| format!("Failed to parse connector JSON: {e}"))
    }

    /// 解析语义规则JSON
    pub fn parse_semantic_rule(json_str: &str) -> Result<SemanticRule, String> {
        serde_json::from_str(json_str)
            .map_err(|e| format!("Failed to parse semantic rule JSON: {e}"))
    }

    /// 解析协议栈定义JSON
    pub fn parse_protocol_stack(json_str: &str) -> Result<ProtocolStackDefinition, String> {
        serde_json::from_str(json_str)
            .map_err(|e| format!("Failed to parse protocol stack JSON: {e}"))
    }

    /// 验证JSON格式
    pub fn validate_json(json_str: &str) -> Result<Value, String> {
        serde_json::from_str(json_str).map_err(|e| format!("Invalid JSON format: {e}"))
    }

    /// 解析标准 CCSDS JSON 格式并转换为内部 PackageDefinition 格式
    ///
    /// 此方法用于解析标准的 CCSDS 协议 JSON 定义文件（如 ccsds_packet_structure.json）
    /// 并将其转换为系统内部使用的 PackageDefinition 格式。
    ///
    /// # 参数
    /// - `standard_json`: 标准 CCSDS JSON 字符串
    /// - `package_name`: 包名称
    ///
    /// # 返回
    /// 返回转换后的 `PackageDefinition` 或错误信息
    ///
    /// # 示例
    /// ```no_run
    /// use apdl_poem::dsl::json_parser::JsonParser;
    /// use std::fs;
    ///
    /// let json = fs::read_to_string("ccsds_packet_structure.json").unwrap();
    /// let package = JsonParser::parse_standard_ccsds_json(&json, "space_packet").unwrap();
    /// ```
    pub fn parse_standard_ccsds_json(
        standard_json: &str,
        package_name: &str,
    ) -> Result<PackageDefinition, String> {
        let json: Value = serde_json::from_str(standard_json)
            .map_err(|e| format!("Failed to parse JSON: {e}"))?;

        // 提取字段定义
        let fields = json["fields"]
            .as_array()
            .ok_or_else(|| "Missing 'fields' array in JSON".to_string())?;

        let mut units = Vec::new();

        for field in fields {
            let field_name = field["name"]
                .as_str()
                .ok_or_else(|| "Missing 'name' field".to_string())?;
            let field_type = field["type"]
                .as_str()
                .ok_or_else(|| "Missing 'type' field".to_string())?;
            let length_str = field["length"]
                .as_str()
                .ok_or_else(|| "Missing 'length' field".to_string())?;

            // 解析字段类型
            let unit_type = if field_type.starts_with("Bit(") && field_type.ends_with(')') {
                // 解析 Bit(N) 格式
                let bits_str = field_type.trim_start_matches("Bit(").trim_end_matches(')');
                let bits: u8 = bits_str
                    .parse()
                    .map_err(|_| format!("Invalid bit count: {bits_str}"))?;
                UnitType::Bit(bits)
            } else {
                match field_type {
                    "Uint8" => UnitType::Uint(8),
                    "Uint16" => UnitType::Uint(16),
                    "Uint32" => UnitType::Uint(32),
                    "RawData" => UnitType::RawData,
                    _ => {
                        return Err(format!("Unknown field type: {field_type}"));
                    }
                }
            };

            // 解析长度
            let (size, unit) = if length_str == "dynamic" {
                (0, LengthUnit::Dynamic)
            } else if let Some(size_str) = length_str.strip_suffix("bit") {
                let size: usize = size_str
                    .parse()
                    .map_err(|_| format!("Invalid bit size: {size_str}"))?;
                (size, LengthUnit::Bit)
            } else if let Some(size_str) = length_str.strip_suffix("byte") {
                let size: usize = size_str
                    .parse()
                    .map_err(|_| format!("Invalid byte size: {size_str}"))?;
                (size, LengthUnit::Byte)
            } else {
                (0, LengthUnit::Dynamic)
            };

            // 解析作用域
            let scope_str = field
                .get("scope")
                .and_then(|s| s.as_str())
                .unwrap_or("layer(default)");
            let scope = if let Some(layer_name) = scope_str.strip_prefix("layer(") {
                let layer_name = layer_name.trim_end_matches(')').to_string();
                ScopeDesc::Global(layer_name)
            } else {
                ScopeDesc::Global("default".to_string())
            };

            // 解析约束（如果存在）
            let constraint = field
                .get("fixed_value")
                .and_then(|fixed_value| fixed_value.as_u64())
                .map(Constraint::FixedValue);

            // 创建字段定义
            let syntax_unit = SyntaxUnit {
                field_id: field_name.to_string(),
                unit_type,
                length: LengthDesc { size, unit },
                scope,
                cover: CoverDesc::EntireField,
                constraint,
                alg: None,
                associate: vec![],
                desc: field
                    .get("description")
                    .and_then(|d| d.as_str())
                    .unwrap_or("")
                    .to_string(),
            };

            units.push(syntax_unit);
        }

        // 解析规则定义（如果存在）
        let mut rules = Vec::new();
        if let Some(rules_array) = json.get("rules").and_then(|r| r.as_array()) {
            for rule_json in rules_array {
                if let Ok(rule) = Self::parse_standard_rule(rule_json) {
                    rules.push(rule);
                }
            }
        }

        // 创建包定义
        Ok(PackageDefinition {
            name: package_name.to_string(),
            display_name: json["name"].as_str().unwrap_or(package_name).to_string(),
            package_type: json["type"].as_str().unwrap_or("unknown").to_string(),
            description: json["description"].as_str().unwrap_or("").to_string(),
            layers: vec![LayerDefinition {
                name: "default_layer".to_string(),
                units,
                rules,
            }],
        })
    }

    /// 解析标准 JSON 格式的单个规则
    ///
    /// 将标准 JSON 规则转换为内部 SemanticRule 格式
    fn parse_standard_rule(rule_json: &Value) -> Result<SemanticRule, String> {
        let rule_type = rule_json["type"]
            .as_str()
            .ok_or_else(|| "Missing rule type".to_string())?;

        match rule_type {
            // 依赖规则
            "dependency" => {
                let source_field = rule_json["source_field"]
                    .as_str()
                    .ok_or_else(|| "Missing source_field".to_string())?
                    .to_string();
                let target_field = rule_json["target_field"]
                    .as_str()
                    .ok_or_else(|| "Missing target_field".to_string())?
                    .to_string();
                Ok(SemanticRule::Dependency {
                    dependent_field: source_field,
                    dependency_field: target_field,
                })
            }

            // 序列控制规则
            "sequence_control" => {
                let field_name = rule_json["field"]
                    .as_str()
                    .ok_or_else(|| "Missing field".to_string())?
                    .to_string();
                let logic = rule_json["logic"]
                    .as_str()
                    .unwrap_or("monotonic_increase")
                    .to_string();
                let description = rule_json["description"].as_str().unwrap_or("").to_string();

                // 构造触发条件（如果有modulo信息）
                let trigger_condition = if let Some(modulo) = rule_json.get("modulo") {
                    format!("modulo:{modulo}")
                } else {
                    "always".to_string()
                };

                Ok(SemanticRule::SequenceControl {
                    field_name,
                    trigger_condition,
                    algorithm: logic,
                    description,
                })
            }

            // 约束验证规则
            "constraint_validation" => {
                let field_name = rule_json["field"]
                    .as_str()
                    .ok_or_else(|| "Missing field".to_string())?
                    .to_string();
                let constraint = rule_json["constraint"].as_str().unwrap_or("").to_string();
                let description = rule_json["description"].as_str().unwrap_or("").to_string();

                Ok(SemanticRule::Validation {
                    field_name,
                    algorithm: "constraint_check".to_string(),
                    range_start: String::new(),
                    range_end: constraint,
                    description,
                })
            }

            // 固定值验证规则
            "fixed_value_validation" => {
                let field_name = rule_json["field"]
                    .as_str()
                    .ok_or_else(|| "Missing field".to_string())?
                    .to_string();
                let expected_value = rule_json["expected_value"]
                    .as_str()
                    .unwrap_or("")
                    .to_string();
                let description = rule_json["description"].as_str().unwrap_or("").to_string();

                Ok(SemanticRule::Validation {
                    field_name,
                    algorithm: "fixed_value".to_string(),
                    range_start: expected_value,
                    range_end: String::new(),
                    description,
                })
            }

            // 校验和验证规则
            "checksum_validation" => {
                let field_name = rule_json["field"]
                    .as_str()
                    .ok_or_else(|| "Missing field".to_string())?
                    .to_string();
                let algorithm = rule_json["algorithm"]
                    .as_str()
                    .unwrap_or("crc16")
                    .to_string();
                let range_start = rule_json
                    .get("range")
                    .and_then(|r| r.get("start"))
                    .and_then(|s| s.as_str())
                    .unwrap_or("")
                    .to_string();
                let range_end = rule_json
                    .get("range")
                    .and_then(|r| r.get("end"))
                    .and_then(|e| e.as_str())
                    .unwrap_or("")
                    .to_string();
                let description = rule_json["description"].as_str().unwrap_or("").to_string();

                Ok(SemanticRule::Validation {
                    field_name,
                    algorithm,
                    range_start,
                    range_end,
                    description,
                })
            }

            // 边界检测规则（转换为长度规则）
            "boundary_detection" => {
                let length_field = rule_json["length_field"]
                    .as_str()
                    .ok_or_else(|| "Missing length_field".to_string())?
                    .to_string();
                let calculation = rule_json["calculation"].as_str().unwrap_or("").to_string();

                Ok(SemanticRule::LengthRule {
                    field_name: length_field,
                    expression: calculation,
                })
            }

            // 字段映射规则
            "field_mapping" => {
                let source_field = rule_json["source_field"].as_str().unwrap_or("").to_string();
                let description = rule_json["description"].as_str().unwrap_or("").to_string();

                // 简化处理：将映射信息存入描述
                let mapping_desc = format!("{description} - {source_field}");

                Ok(SemanticRule::RoutingDispatch {
                    fields: vec![source_field],
                    algorithm: "field_mapping".to_string(),
                    description: mapping_desc,
                })
            }

            // 枚举映射规则
            "enum_mapping" => {
                let field_name = rule_json["field"]
                    .as_str()
                    .ok_or_else(|| "Missing field".to_string())?
                    .to_string();
                let _description = rule_json["description"].as_str().unwrap_or("").to_string();

                Ok(SemanticRule::Algorithm {
                    field_name,
                    algorithm: "enum_mapping".to_string(),
                })
            }

            // 长度验证规则
            "length_validation" => {
                let field_name = rule_json["field"]
                    .as_str()
                    .ok_or_else(|| "Missing field".to_string())?
                    .to_string();
                let condition = rule_json["condition"].as_str().unwrap_or("").to_string();
                let description = rule_json["description"].as_str().unwrap_or("").to_string();

                Ok(SemanticRule::LengthValidation {
                    field_name,
                    condition,
                    description,
                })
            }

            // 同步规则
            "synchronization" => {
                let field_name = rule_json["field"]
                    .as_str()
                    .ok_or_else(|| "Missing field".to_string())?
                    .to_string();
                let algorithm = rule_json["algorithm"]
                    .as_str()
                    .unwrap_or("sync_detect")
                    .to_string();
                let description = rule_json["description"].as_str().unwrap_or("").to_string();

                Ok(SemanticRule::Synchronization {
                    field_name,
                    algorithm,
                    description,
                })
            }

            // 未知规则类型，跳过
            _ => Err(format!("Unsupported rule type: {rule_type}")),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_package_json() {
        let json_str = r#"
        {
            "name": "telemetry_packet",
            "display_name": "Telemetry Packet",
            "package_type": "telemetry",
            "description": "Telemetry packet with version, APID, length and data",
            "layers": [
                {
                    "name": "telemetry_layer",
                    "units": [
                        {
                            "field_id": "version",
                            "unit_type": {
                                "Uint": 8
                            },
                            "length": {
                                "size": 1,
                                "unit": "Byte"
                            },
                            "scope": {
                                "Global": "telemetry"
                            },
                            "cover": "EntireField",
                            "constraint": {
                                "Range": [0, 255]
                            },
                            "alg": null,
                            "associate": [],
                            "desc": "Version number"
                        }
                    ],
                    "rules": []
                }
            ]
        }
        "#;

        let result = JsonParser::parse_package(json_str);
        assert!(
            result.is_ok(),
            "Failed to parse package JSON: {:?}",
            result.err()
        );
    }

    #[test]
    fn test_parse_connector_json() {
        let json_str = r#"
        {
            "name": "telemetry_to_encap_connector",
            "connector_type": "field_mapping",
            "source_package": "telemetry_packet",
            "target_package": "encapsulating_packet",
            "config": {
                "mappings": [
                    {
                        "source_field": "apid",
                        "target_field": "vcid",
                        "mapping_logic": "identity",
                        "default_value": "0",
                        "enum_mappings": null
                    }
                ],
                "header_pointers": null,
                "data_placement": null
            },
            "description": "Maps telemetry packet fields to encap packet fields"
        }
        "#;

        let result = JsonParser::parse_connector(json_str);
        assert!(
            result.is_ok(),
            "Failed to parse connector JSON: {:?}",
            result.err()
        );
    }
}
