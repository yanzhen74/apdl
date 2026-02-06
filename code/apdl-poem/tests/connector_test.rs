//! 连接器功能测试
//!
//! 验证字段映射连接器的正确性和功能

use apdl_core::SemanticRule;
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
        assert!(mapping.enum_mappings.is_none());
    } else {
        panic!("Expected FieldMapping rule");
    }
}

#[test]
fn test_connector_engine_basic_functionality() {
    use apdl_core::EnumMappingEntry;
    use apdl_poem::standard_units::connector::{
        connector_engine::ConnectorEngine, field_mapper::FieldMapper,
    };

    // 创建连接器引擎
    let _engine = ConnectorEngine::new();

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

    // 测试枚举映射功能
    let enum_mappings = vec![
        EnumMappingEntry {
            source_enum: "data_type_a".to_string(),
            target_enum: "vcid_0".to_string(),
        },
        EnumMappingEntry {
            source_enum: "data_type_b".to_string(),
            target_enum: "vcid_1".to_string(),
        },
        EnumMappingEntry {
            source_enum: "*".to_string(), // 通配符匹配
            target_enum: "default_vcid".to_string(),
        },
    ];

    let result = mapper.map_enum("data_type_a", Some(&enum_mappings));
    assert_eq!(result, Some("vcid_0".to_string()));

    let result = mapper.map_enum("data_type_b", Some(&enum_mappings));
    assert_eq!(result, Some("vcid_1".to_string()));

    let result = mapper.map_enum("other_type", Some(&enum_mappings));
    assert_eq!(result, Some("default_vcid".to_string()));

    // 测试通配符匹配
    let wildcard_mappings = vec![EnumMappingEntry {
        source_enum: "type_*".to_string(), // 通配符模式
        target_enum: "general_type".to_string(),
    }];

    let result = mapper.map_enum("type_123", Some(&wildcard_mappings));
    assert_eq!(result, Some("general_type".to_string()));

    let result = mapper.map_enum("type_data", Some(&wildcard_mappings));
    assert_eq!(result, Some("general_type".to_string()));
}
