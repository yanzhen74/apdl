//! 连接器功能测试
//!
//! 验证字段映射连接器的正确性和功能

use apdl_core::{
    Constraint, CoverDesc, FieldMappingEntry, LengthDesc, LengthUnit, SemanticRule, SyntaxUnit,
    UnitType,
};
use apdl_poem::dsl::parser::DslParserImpl;

#[test]
fn test_field_mapping_rule_parsing() {
    let parser = DslParserImpl::new();

    // 测试字段映射规则解析
    let dsl = r#"rule: field_mapping(source_package: "lower_layer_packet"; target_package: "upper_layer_packet"; mappings: [{source_field: "src_id", target_field: "vcid", mapping_logic: "hash_mod_64", default_value: "0"}]; desc: "Map source ID to VCID")"#;

    let result = parser.parse_semantic_rules(dsl);
    assert!(
        result.is_ok(),
        "Failed to parse field mapping rule: {:?}",
        result.err()
    );

    let rules = result.unwrap();
    assert_eq!(rules.len(), 1);

    if let SemanticRule::FieldMapping {
        source_package,
        target_package,
        mappings,
        description,
    } = &rules[0]
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

#[test]
fn test_connector_engine_basic_functionality() {
    use apdl_poem::standard_units::connector::{ConnectorEngine, FieldMapper};

    // 创建连接器引擎
    let mut engine = ConnectorEngine::new();

    // 创建字段映射器
    let mapper = FieldMapper::new();

    // 验证基本功能
    assert_eq!(mapper.mapping_functions.len(), 4); // identity, hash_mod_64, hash_mod_2048, shift_right_8

    // 测试映射功能
    let input = vec![0x12, 0x34];
    let result = mapper.map_field(&input, "identity").unwrap();
    assert_eq!(result, input);

    let result = mapper.map_field(&input, "hash_mod_64").unwrap();
    assert_eq!(result.len(), 1);
    assert!(result[0] < 64);
}

#[test]
fn test_complete_connector_workflow() {
    use apdl_poem::standard_units::connector::{ConnectorEngine, FieldMapper};

    // 创建源包和目标包的语法单元定义
    let source_package = vec![
        SyntaxUnit {
            field_id: "src_id".to_string(),
            unit_type: UnitType::Uint(16),
            length: LengthDesc {
                size: 2,
                unit: LengthUnit::Byte,
            },
            scope: apdl_core::ScopeDesc::Layer("data".to_string()),
            cover: CoverDesc::EntireField,
            constraint: Some(Constraint::Range(0, 65535)),
            alg: None,
            associate: vec![],
            desc: "Source identifier".to_string(),
        },
        SyntaxUnit {
            field_id: "src_type".to_string(),
            unit_type: UnitType::Uint(8),
            length: LengthDesc {
                size: 1,
                unit: LengthUnit::Byte,
            },
            scope: apdl_core::ScopeDesc::Layer("data".to_string()),
            cover: CoverDesc::EntireField,
            constraint: Some(Constraint::Range(0, 255)),
            alg: None,
            associate: vec![],
            desc: "Source type".to_string(),
        },
    ];

    let mut target_package = vec![
        SyntaxUnit {
            field_id: "vcid".to_string(),
            unit_type: UnitType::Bit(6),
            length: LengthDesc {
                size: 1,
                unit: LengthUnit::Byte,
            },
            scope: apdl_core::ScopeDesc::Layer("header".to_string()),
            cover: CoverDesc::EntireField,
            constraint: Some(Constraint::Range(0, 63)),
            alg: None,
            associate: vec![],
            desc: "Virtual Channel ID".to_string(),
        },
        SyntaxUnit {
            field_id: "apid".to_string(),
            unit_type: UnitType::Bit(11),
            length: LengthDesc {
                size: 2,
                unit: LengthUnit::Byte,
            },
            scope: apdl_core::ScopeDesc::Layer("header".to_string()),
            cover: CoverDesc::EntireField,
            constraint: Some(Constraint::Range(0, 2047)),
            alg: None,
            associate: vec![],
            desc: "Application Process ID".to_string(),
        },
    ];

    // 创建映射规则
    let mapping_rule = SemanticRule::FieldMapping {
        source_package: "telemetry_packet".to_string(),
        target_package: "encapsulating_packet".to_string(),
        mappings: vec![
            FieldMappingEntry {
                source_field: "src_id".to_string(),
                target_field: "vcid".to_string(),
                mapping_logic: "hash_mod_64".to_string(),
                default_value: "0".to_string(),
            },
            FieldMappingEntry {
                source_field: "src_id".to_string(),
                target_field: "apid".to_string(),
                mapping_logic: "hash_mod_2048".to_string(),
                default_value: "0".to_string(),
            },
        ],
        description: "Map telemetry source to VCID and APID".to_string(),
    };

    // 创建连接器引擎并添加规则
    let mut engine = ConnectorEngine::new();
    engine.add_mapping_rule(mapping_rule);

    // 应用映射规则
    let result = engine.apply_mapping_rules(&source_package, &mut target_package);
    assert!(
        result.is_ok(),
        "Failed to apply mapping rules: {:?}",
        result.err()
    );
}
