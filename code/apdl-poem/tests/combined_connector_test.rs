//! 组合连接器测试
//!
//! 展示DSL解析器和运行时引擎如何协同工作的测试用例

use apdl_core::{CoverDesc, LengthDesc, LengthUnit, ScopeDesc, SemanticRule, SyntaxUnit, UnitType};
use apdl_poem::dsl::layers::connector_parser::ConnectorParser;
use apdl_poem::standard_units::connector::connector_engine::ConnectorEngine;

/// 测试DSL解析器和运行时引擎的组合使用
#[cfg(test)]
mod tests {
    use apdl_core::{DataPlacementConfig, DataPlacementStrategy};
    use apdl_poem::dsl::layers::connector_parser::ConnectorParser;
    use apdl_poem::standard_units::connector::connector_engine::ConnectorEngine;

    #[test]
    fn test_dsl_parser_and_runtime_engine_integration() {
        // 1. 定义DSL文本 - 描述连接器配置
        let connector_dsl = r#"
        connector test_field_mapping {
            type: "field_mapping";
            source_package: "telemetry_packet";
            target_package: "encapsulating_packet";
            config: {
                mappings: [
                    {
                        source_field: "tlm_source_id",
                        target_field: "apid",
                        logic: "hash_mod_2048",
                        default_value: "0"
                    },
                    {
                        source_field: "packet_sequence_control",
                        target_field: "sequence_count",
                        logic: "identity",
                        default_value: "1"
                    }
                ];
                placement_strategy: {
                    strategy: "pointer_based";
                    target_field: "data_field";
                    config: {
                        pointer_field: "first_header_ptr";
                        map_id: "map_id_field";
                    };
                };
            };
            desc: "Test field mapping from telemetry to encapsulating packet";
        }
        "#;

        // 2. 使用DSL解析器解析DSL文本
        let connector_definition = ConnectorParser::parse_connector_definition(connector_dsl)
            .expect("Failed to parse connector DSL");

        // 3. 验证解析结果
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
        assert_eq!(
            connector_definition.config.mappings[0].source_field,
            "tlm_source_id"
        );
        assert_eq!(connector_definition.config.mappings[0].target_field, "apid");

        // 验证数据放置配置
        assert!(connector_definition.config.data_placement.is_some());
        let placement_config = connector_definition.config.data_placement.unwrap();
        match placement_config.strategy {
            DataPlacementStrategy::PointerBased => {}
            _ => panic!("Expected PointerBased strategy"),
        }
        assert_eq!(placement_config.target_field, "data_field");

        // 4. 创建运行时引擎
        let mut engine = ConnectorEngine::new();

        // 5. 注意：由于当前的SemanticRule::FieldMapping构造方式限制，
        // 我们不能直接创建包含包名的FieldMapping规则
        // 因此，我们单独测试引擎的数据放置功能

        // 6. 创建模拟的语法单元用于测试
        let mock_source_package = vec![apdl_core::SyntaxUnit {
            field_id: "tlm_source_id".to_string(),
            unit_type: apdl_core::UnitType::Uint(16),
            length: apdl_core::LengthDesc {
                size: 2,
                unit: apdl_core::LengthUnit::Byte,
            },
            scope: apdl_core::ScopeDesc::Global("end2end".to_string()),
            cover: apdl_core::CoverDesc::EntireField,
            constraint: None,
            alg: None,
            associate: vec![],
            desc: "Mock telemetry source ID".to_string(),
        }];

        let mut mock_target_package = vec![apdl_core::SyntaxUnit {
            field_id: "apid".to_string(),
            unit_type: apdl_core::UnitType::Uint(16),
            length: apdl_core::LengthDesc {
                size: 2,
                unit: apdl_core::LengthUnit::Byte,
            },
            scope: apdl_core::ScopeDesc::Global("end2end".to_string()),
            cover: apdl_core::CoverDesc::EntireField,
            constraint: None,
            alg: None,
            associate: vec![],
            desc: "Mock APID field".to_string(),
        }];

        // 7. 使用引擎执行数据放置策略
        let result = engine.apply_data_placement(
            &mock_source_package,
            &mut mock_target_package,
            &DataPlacementConfig {
                strategy: DataPlacementStrategy::PointerBased,
                target_field: "apid".to_string(),
                config_params: vec![("pointer_field".to_string(), "first_header_ptr".to_string())],
            },
        );

        assert!(
            result.is_ok(),
            "Failed to apply data placement: {:?}",
            result.err()
        );

        // 8. 验证引擎的字段映射功能（使用简化的测试）
        // 注意：这里我们无法使用完整的FieldMapping语义规则，因为缺少包名参数
        // 在实际应用中，FieldMapping规则通常通过其他途径添加
        println!("Integration test completed successfully!");
        println!("DSL Parser correctly parsed the connector definition.");
        println!("Runtime Engine is ready to process field mappings and data placement.");
    }
}
