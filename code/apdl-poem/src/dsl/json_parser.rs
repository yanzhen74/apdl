//! JSON解析器模块
//!
//! 用于解析JSON格式的协议定义

use apdl_core::{ConnectorDefinition, PackageDefinition, ProtocolStackDefinition, SemanticRule};
use serde_json::Value;

/// JSON解析器
pub struct JsonParser;

impl JsonParser {
    /// 解析包定义JSON
    pub fn parse_package(json_str: &str) -> Result<PackageDefinition, String> {
        serde_json::from_str(json_str).map_err(|e| format!("Failed to parse package JSON: {}", e))
    }

    /// 解析连接器定义JSON
    pub fn parse_connector(json_str: &str) -> Result<ConnectorDefinition, String> {
        serde_json::from_str(json_str).map_err(|e| format!("Failed to parse connector JSON: {}", e))
    }

    /// 解析语义规则JSON
    pub fn parse_semantic_rule(json_str: &str) -> Result<SemanticRule, String> {
        serde_json::from_str(json_str)
            .map_err(|e| format!("Failed to parse semantic rule JSON: {}", e))
    }

    /// 解析协议栈定义JSON
    pub fn parse_protocol_stack(json_str: &str) -> Result<ProtocolStackDefinition, String> {
        serde_json::from_str(json_str)
            .map_err(|e| format!("Failed to parse protocol stack JSON: {}", e))
    }

    /// 验证JSON格式
    pub fn validate_json(json_str: &str) -> Result<Value, String> {
        serde_json::from_str(json_str).map_err(|e| format!("Invalid JSON format: {}", e))
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
