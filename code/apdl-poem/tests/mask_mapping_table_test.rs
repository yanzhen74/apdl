//! 掩码映射表功能测试
//!
//! 验证基于掩码表的字段映射功能

use apdl_core::SemanticRule;
use apdl_poem::dsl::parser::DslParserImpl;

#[test]
fn test_parse_mask_mapping_table() {
    let parser = DslParserImpl::new();

    // 测试带有掩码映射表的字段映射规则解析
    let dsl = r#"rule: field_mapping(source_package: "mpdu_packet"; target_package: "tm_frame"; mappings: [{source_field: "apid", target_field: "vcid", mapping_logic: "mask_table", default_value: "0x00", mask_mapping_table: [{mask: [0xFF, 0xF0], src_masked: [0x04, 0x80], dst: [0x35]}, {mask: [0xFF, 0xF0], src_masked: [0x04, 0x90], dst: [0x36]}, {mask: [0xFF, 0xFF], src_masked: [0x05, 0x00], dst: [0x37]}]}]; desc: "APID到VCID的掩码映射")"#;

    let result = parser.parse_semantic_rules(dsl);
    assert!(
        result.is_ok(),
        "Failed to parse mask mapping table rule: {:?}",
        result.err()
    );

    let rules = result.unwrap();
    assert_eq!(rules.len(), 1);

    if let SemanticRule::FieldMapping { mappings, .. } = &rules[0] {
        assert_eq!(mappings.len(), 1);
        let mapping = &mappings[0];

        assert_eq!(mapping.source_field, "apid");
        assert_eq!(mapping.target_field, "vcid");
        assert_eq!(mapping.mapping_logic, "mask_table");
        assert_eq!(mapping.default_value, "0x00");

        // 验证掩码映射表
        assert!(mapping.mask_mapping_table.is_some());
        let table = mapping.mask_mapping_table.as_ref().unwrap();
        assert_eq!(table.len(), 3);

        // 验证第一个映射条目
        assert_eq!(table[0].mask, vec![0xFF, 0xF0]);
        assert_eq!(table[0].src_masked, vec![0x04, 0x80]);
        assert_eq!(table[0].dst, vec![0x35]);

        // 验证第二个映射条目
        assert_eq!(table[1].mask, vec![0xFF, 0xF0]);
        assert_eq!(table[1].src_masked, vec![0x04, 0x90]);
        assert_eq!(table[1].dst, vec![0x36]);

        // 验证第三个映射条目
        assert_eq!(table[2].mask, vec![0xFF, 0xFF]);
        assert_eq!(table[2].src_masked, vec![0x05, 0x00]);
        assert_eq!(table[2].dst, vec![0x37]);
    } else {
        panic!("Expected FieldMapping rule");
    }
}

#[test]
fn test_parse_mask_mapping_table_decimal() {
    let parser = DslParserImpl::new();

    // 测试使用十进制数值的掩码映射表
    let dsl = r#"rule: field_mapping(source_package: "source_pkg"; target_package: "target_pkg"; mappings: [{source_field: "src_field", target_field: "dst_field", mapping_logic: "mask_table", default_value: "0", mask_mapping_table: [{mask: [255, 240], src_masked: [4, 128], dst: [53]}]}]; desc: "Decimal mask mapping")"#;

    let result = parser.parse_semantic_rules(dsl);
    assert!(
        result.is_ok(),
        "Failed to parse decimal mask mapping: {:?}",
        result.err()
    );

    let rules = result.unwrap();
    if let SemanticRule::FieldMapping { mappings, .. } = &rules[0] {
        let table = mappings[0].mask_mapping_table.as_ref().unwrap();
        assert_eq!(table[0].mask, vec![255, 240]);
        assert_eq!(table[0].src_masked, vec![4, 128]);
        assert_eq!(table[0].dst, vec![53]);
    }
}
