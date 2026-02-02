//! 枚举映射功能测试
//!
//! 验证字段映射规则中枚举映射功能的正确性和功能

use apdl_core::{EnumMappingEntry, FieldMappingEntry, SemanticRule};
use apdl_poem::dsl::parser::DslParserImpl;

#[test]
fn test_field_mapping_with_enum_mappings() {
    let parser = DslParserImpl::new();

    // 测试带有枚举映射的字段映射规则解析
    let dsl = r#"rule: field_mapping(source_package: "lower_layer_packet"; target_package: "upper_layer_packet"; mappings: [{source_field: "src_type", target_field: "vcid", mapping_logic: "hash_mod_64", default_value: "0", enum_mappings: [{source_enum: "data_type_a", target_enum: "vcid_0"}, {source_enum: "data_type_b", target_enum: "vcid_1"}, {source_enum: "*", target_enum: "default_vcid"}]}]; desc: "Map source type to VCID with enum mapping")"#;

    let result = parser.parse_semantic_rules(dsl);
    assert!(
        result.is_ok(),
        "Failed to parse field mapping rule with enum mappings: {:?}",
        result.err()
    );

    let rules = result.unwrap();
    assert_eq!(rules.len(), 1);

    if let SemanticRule::FieldMapping { mappings, .. } = &rules[0] {
        assert_eq!(mappings.len(), 1);
        let mapping = &mappings[0];
        assert!(mapping.enum_mappings.is_some());

        let enum_mappings = mapping.enum_mappings.as_ref().unwrap();
        assert_eq!(enum_mappings.len(), 3);

        assert_eq!(enum_mappings[0].source_enum, "data_type_a");
        assert_eq!(enum_mappings[0].target_enum, "vcid_0");
        assert_eq!(enum_mappings[1].source_enum, "data_type_b");
        assert_eq!(enum_mappings[1].target_enum, "vcid_1");
        assert_eq!(enum_mappings[2].source_enum, "*");
        assert_eq!(enum_mappings[2].target_enum, "default_vcid");
    } else {
        panic!("Expected FieldMapping rule");
    }
}

#[test]
fn test_wildcard_enum_matching() {
    use apdl_poem::standard_units::connector::FieldMapper;

    let mapper = FieldMapper::new();

    // 测试通配符枚举映射
    let enum_mappings = vec![
        EnumMappingEntry {
            source_enum: "type_*".to_string(), // 通配符模式
            target_enum: "general_type".to_string(),
        },
        EnumMappingEntry {
            source_enum: "special_case".to_string(),
            target_enum: "specific_type".to_string(),
        },
    ];

    // 通配符匹配应该成功
    let result = mapper.map_enum("type_123", Some(&enum_mappings));
    assert_eq!(result, Some("general_type".to_string()));

    let result = mapper.map_enum("type_data", Some(&enum_mappings));
    assert_eq!(result, Some("general_type".to_string()));

    let result = mapper.map_enum("type_xyz", Some(&enum_mappings));
    assert_eq!(result, Some("general_type".to_string()));

    // 精确匹配应该优先于通配符
    let result = mapper.map_enum("special_case", Some(&enum_mappings));
    assert_eq!(result, Some("specific_type".to_string()));
}

#[test]
fn test_question_mark_wildcard() {
    use apdl_poem::standard_units::connector::FieldMapper;

    let mapper = FieldMapper::new();

    // 测试问号通配符（匹配单个字符）
    let enum_mappings = vec![EnumMappingEntry {
        source_enum: "type_?".to_string(), // 问号匹配单个字符
        target_enum: "single_char_type".to_string(),
    }];

    // 问号匹配单个字符
    let result = mapper.map_enum("type_a", Some(&enum_mappings));
    assert_eq!(result, Some("single_char_type".to_string()));

    let result = mapper.map_enum("type_1", Some(&enum_mappings));
    assert_eq!(result, Some("single_char_type".to_string()));

    // 不应该匹配多个字符
    let result = mapper.map_enum("type_ab", Some(&enum_mappings));
    assert_eq!(result, None);

    // 不应该匹配单字符不足的情况
    let result = mapper.map_enum("type", Some(&enum_mappings));
    assert_eq!(result, None);
}

#[test]
fn test_complex_wildcard_patterns() {
    use apdl_poem::standard_units::connector::FieldMapper;

    let mapper = FieldMapper::new();

    // 测试复杂的通配符模式
    let enum_mappings = vec![
        EnumMappingEntry {
            source_enum: "cmd_*_req".to_string(), // 命令请求模式
            target_enum: "command_request".to_string(),
        },
        EnumMappingEntry {
            source_enum: "resp_*_ack".to_string(), // 响应确认模式
            target_enum: "response_ack".to_string(),
        },
    ];

    // 测试命令请求模式
    let result = mapper.map_enum("cmd_reset_req", Some(&enum_mappings));
    assert_eq!(result, Some("command_request".to_string()));

    let result = mapper.map_enum("cmd_init_req", Some(&enum_mappings));
    assert_eq!(result, Some("command_request".to_string()));

    // 测试响应确认模式
    let result = mapper.map_enum("resp_status_ack", Some(&enum_mappings));
    assert_eq!(result, Some("response_ack".to_string()));

    let result = mapper.map_enum("resp_data_ack", Some(&enum_mappings));
    assert_eq!(result, Some("response_ack".to_string()));

    // 不匹配的模式
    let result = mapper.map_enum("cmd_other", Some(&enum_mappings));
    assert_eq!(result, None);

    let result = mapper.map_enum("other_resp_ack", Some(&enum_mappings));
    assert_eq!(result, None);
}
