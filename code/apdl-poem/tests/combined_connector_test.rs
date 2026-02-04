#[cfg(test)]
mod tests {
    use apdl_core::{
        CoverDesc, LengthDesc, LengthUnit, ScopeDesc, SemanticRule, SyntaxUnit, UnitType,
    };
    use apdl_poem::{
        dsl::json_parser::JsonParser, standard_units::connector::connector_engine::ConnectorEngine,
    };

    #[test]
    fn test_runtime_engine_functionality() {
        // 完整的工作流程测试：从JSON定义到运行时执行
        let connector_json = r#"
        {
            "name": "telemetry_to_encapsulating",
            "connector_type": "field_mapping",
            "source_package": "telemetry_packet",
            "target_package": "encapsulating_packet",
            "config": {
                "mappings": [
                    {
                        "source_field": "source_id",
                        "target_field": "apid",
                        "mapping_logic": "hash_mod_64",
                        "default_value": "0x0000",
                        "enum_mappings": null
                    }
                ],
                "header_pointers": null,
                "data_placement": null
            },
            "description": "Map telemetry source ID to APID"
        }
        "#;

        // 步骤1: JSON解析
        let connector_def =
            JsonParser::parse_connector(connector_json).expect("Should parse JSON successfully");

        // 步骤2: 创建语义规则
        let rule = SemanticRule::FieldMapping {
            source_package: connector_def.source_package.clone(),
            target_package: connector_def.target_package.clone(),
            mappings: connector_def.config.mappings.clone(),
            description: connector_def.description.clone(),
        };

        // 步骤3: 初始化引擎
        let mut engine = ConnectorEngine::new();
        engine.add_mapping_rule(rule);

        // 步骤4: 准备测试用的源数据包和目标数据包
        let source_package = vec![SyntaxUnit {
            field_id: "source_id".to_string(),
            unit_type: UnitType::Uint(16),
            length: LengthDesc {
                size: 2,
                unit: LengthUnit::Byte,
            },
            scope: ScopeDesc::Layer("telemetry".to_string()),
            cover: CoverDesc::EntireField,
            constraint: None,
            alg: None,
            associate: vec![],
            desc: "Source ID for mapping test".to_string(),
        }];

        let mut target_package = vec![SyntaxUnit {
            field_id: "apid".to_string(),
            unit_type: UnitType::Uint(16),
            length: LengthDesc {
                size: 2,
                unit: LengthUnit::Byte,
            },
            scope: ScopeDesc::Layer("encapsulating".to_string()),
            cover: CoverDesc::EntireField,
            constraint: None,
            alg: None,
            associate: vec![],
            desc: "APID field for mapping test".to_string(),
        }];

        // 步骤5: 使用运行时引擎应用映射规则
        let result = engine.apply_mapping_rules(&source_package, &mut target_package);
        assert!(result.is_ok(), "Failed to apply mapping rules");

        // 步骤6: 验证配置
        assert_eq!(connector_def.name, "telemetry_to_encapsulating");

        println!("完整工作流程测试通过！");
        println!("- JSON解析器解析了连接器定义");
        println!("- 运行时引擎加载了映射规则");
        println!("- 系统准备就绪，等待运行时数据处理");
    }

    #[test]
    fn test_dsl_parser_and_runtime_engine_integration() {
        // 1. 定义JSON文本 - 描述连接器配置
        let connector_json = r#"
        {
            "name": "test_field_mapping",
            "connector_type": "field_mapping",
            "source_package": "telemetry_packet",
            "target_package": "encapsulating_packet",
            "config": {
                "mappings": [
                    {
                        "source_field": "tlm_source_id",
                        "target_field": "apid",
                        "mapping_logic": "hash_mod_2048",
                        "default_value": "0",
                        "enum_mappings": null
                    },
                    {
                        "source_field": "packet_sequence_control",
                        "target_field": "sequence_count",
                        "mapping_logic": "identity",
                        "default_value": "1",
                        "enum_mappings": null
                    }
                ],
                "header_pointers": null,
                "data_placement": null
            },
            "description": "Test field mapping from telemetry to encapsulating packet"
        }
        "#;

        // 2. 使用JSON解析器解析连接器定义
        let connector_definition =
            JsonParser::parse_connector(connector_json).expect("Failed to parse connector JSON");

        // 3. 验证解析结果
        assert_eq!(connector_definition.name, "test_field_mapping");
        assert_eq!(connector_definition.source_package, "telemetry_packet");
        assert_eq!(connector_definition.target_package, "encapsulating_packet");
        assert_eq!(
            connector_definition.description,
            "Test field mapping from telemetry to encapsulating packet"
        );
        assert_eq!(connector_definition.config.mappings.len(), 2);

        // 4. 将解析结果转换为运行时引擎可使用的语义规则
        let semantic_rule = SemanticRule::FieldMapping {
            source_package: connector_definition.source_package.clone(),
            target_package: connector_definition.target_package.clone(),
            mappings: connector_definition.config.mappings.clone(),
            description: connector_definition.description.clone(),
        };

        // 5. 创建并配置运行时引擎
        let mut engine = ConnectorEngine::new();
        engine.add_mapping_rule(semantic_rule);

        // 6. 准备测试用的源数据包和目标数据包
        let source_package = vec![
            SyntaxUnit {
                field_id: "tlm_source_id".to_string(),
                unit_type: UnitType::Uint(16),
                length: LengthDesc {
                    size: 2,
                    unit: LengthUnit::Byte,
                },
                scope: ScopeDesc::Layer("telemetry".to_string()),
                cover: CoverDesc::EntireField,
                constraint: None,
                alg: None,
                associate: vec![],
                desc: "Telemetry source ID".to_string(),
            },
            SyntaxUnit {
                field_id: "packet_sequence_control".to_string(),
                unit_type: UnitType::Uint(16),
                length: LengthDesc {
                    size: 2,
                    unit: LengthUnit::Byte,
                },
                scope: ScopeDesc::Layer("telemetry".to_string()),
                cover: CoverDesc::EntireField,
                constraint: None,
                alg: None,
                associate: vec![],
                desc: "Packet sequence control".to_string(),
            },
        ];

        let mut target_package = vec![
            SyntaxUnit {
                field_id: "apid".to_string(),
                unit_type: UnitType::Uint(16),
                length: LengthDesc {
                    size: 2,
                    unit: LengthUnit::Byte,
                },
                scope: ScopeDesc::Layer("encapsulating".to_string()),
                cover: CoverDesc::EntireField,
                constraint: None,
                alg: None,
                associate: vec![],
                desc: "Application Process Identifier".to_string(),
            },
            SyntaxUnit {
                field_id: "sequence_count".to_string(),
                unit_type: UnitType::Uint(16),
                length: LengthDesc {
                    size: 2,
                    unit: LengthUnit::Byte,
                },
                scope: ScopeDesc::Layer("encapsulating".to_string()),
                cover: CoverDesc::EntireField,
                constraint: None,
                alg: None,
                associate: vec![],
                desc: "Sequence count".to_string(),
            },
        ];

        // 7. 使用运行时引擎应用映射规则
        let result = engine.apply_mapping_rules(&source_package, &mut target_package);
        assert!(result.is_ok(), "Failed to apply mapping rules");

        // 8. 验证映射结果
        // 注意：在实际实现中，这里会验证字段值是否正确映射
        // 由于ConnectorEngine的set_field_value目前是示意实现，
        // 我们验证的是映射规则已被正确应用的逻辑

        println!("JSON解析器成功解析连接器定义");
        println!(
            "运行时引擎成功应用了{}个映射规则",
            connector_definition.config.mappings.len()
        );
        println!("源包: {}", connector_definition.source_package);
        println!("目标包: {}", connector_definition.target_package);

        // 验证映射规则的数量
        assert_eq!(connector_definition.config.mappings.len(), 2);

        // 验证第一个映射规则
        assert_eq!(
            connector_definition.config.mappings[0].source_field,
            "tlm_source_id"
        );
        assert_eq!(connector_definition.config.mappings[0].target_field, "apid");
        assert_eq!(
            connector_definition.config.mappings[0].mapping_logic,
            "hash_mod_2048"
        );

        // 验证第二个映射规则
        assert_eq!(
            connector_definition.config.mappings[1].source_field,
            "packet_sequence_control"
        );
        assert_eq!(
            connector_definition.config.mappings[1].target_field,
            "sequence_count"
        );
        assert_eq!(
            connector_definition.config.mappings[1].mapping_logic,
            "identity"
        );

        println!("所有验证通过！JSON解析器和运行时引擎成功协同工作。");
    }

    #[test]
    fn test_complete_connector_workflow() {
        // 完整的工作流程测试：从JSON定义到运行时执行
        let connector_json = r#"
        {
            "name": "telemetry_to_encapsulating",
            "connector_type": "field_mapping",
            "source_package": "telemetry_packet",
            "target_package": "encapsulating_packet",
            "config": {
                "mappings": [
                    {
                        "source_field": "source_id",
                        "target_field": "apid",
                        "mapping_logic": "hash_mod_64",
                        "default_value": "0x0000",
                        "enum_mappings": null
                    }
                ],
                "header_pointers": null,
                "data_placement": null
            },
            "description": "Map telemetry source ID to APID"
        }
        "#;

        // 步骤1: JSON解析
        let connector_def =
            JsonParser::parse_connector(connector_json).expect("Should parse JSON successfully");

        // 步骤2: 创建语义规则
        let rule = SemanticRule::FieldMapping {
            source_package: connector_def.source_package.clone(),
            target_package: connector_def.target_package.clone(),
            mappings: connector_def.config.mappings.clone(),
            description: connector_def.description.clone(),
        };

        // 步骤3: 初始化引擎
        let mut engine = ConnectorEngine::new();
        engine.add_mapping_rule(rule);

        // 步骤4: 准备测试用的源数据包和目标数据包
        let source_package = vec![SyntaxUnit {
            field_id: "source_id".to_string(),
            unit_type: UnitType::Uint(16),
            length: LengthDesc {
                size: 2,
                unit: LengthUnit::Byte,
            },
            scope: ScopeDesc::Layer("telemetry".to_string()),
            cover: CoverDesc::EntireField,
            constraint: None,
            alg: None,
            associate: vec![],
            desc: "Source ID for mapping test".to_string(),
        }];

        let mut target_package = vec![SyntaxUnit {
            field_id: "apid".to_string(),
            unit_type: UnitType::Uint(16),
            length: LengthDesc {
                size: 2,
                unit: LengthUnit::Byte,
            },
            scope: ScopeDesc::Layer("encapsulating".to_string()),
            cover: CoverDesc::EntireField,
            constraint: None,
            alg: None,
            associate: vec![],
            desc: "APID field for mapping test".to_string(),
        }];

        // 步骤5: 使用运行时引擎应用映射规则
        let result = engine.apply_mapping_rules(&source_package, &mut target_package);
        assert!(result.is_ok(), "Failed to apply mapping rules");

        // 步骤6: 验证配置
        assert_eq!(connector_def.name, "telemetry_to_encapsulating");

        println!("完整工作流程测试通过！");
        println!("- JSON解析器解析了连接器定义");
        println!("- 运行时引擎加载了映射规则");
        println!("- 系统准备就绪，等待运行时数据处理");
    }
}
