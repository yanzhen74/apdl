//! JSON解析器规则解析测试
//!
//! 验证 JsonParser::parse_standard_ccsds_json 正确解析标准 JSON 中的规则定义

use apdl_poem::dsl::json_parser::JsonParser;

#[test]
fn test_parse_standard_json_with_rules() {
    // 测试包含多种规则类型的 JSON
    let json_with_rules = r#"
    {
        "name": "test_package",
        "type": "packet",
        "description": "测试包定义",
        "fields": [
            {
                "name": "sync_flag",
                "type": "Uint16",
                "length": "2byte",
                "description": "同步标志"
            },
            {
                "name": "seq_count",
                "type": "Uint16",
                "length": "2byte",
                "description": "序列计数"
            },
            {
                "name": "data_len",
                "type": "Uint16",
                "length": "2byte",
                "description": "数据长度"
            },
            {
                "name": "data_field",
                "type": "RawData",
                "length": "dynamic",
                "description": "数据域"
            }
        ],
        "rules": [
            {
                "type": "fixed_value_validation",
                "description": "验证同步标志为固定值0xEB90",
                "field": "sync_flag",
                "expected_value": "0xEB90",
                "severity": "error"
            },
            {
                "type": "sequence_control",
                "description": "序列计数器应递增并循环",
                "field": "seq_count",
                "logic": "monotonic_increase",
                "modulo": 65536
            },
            {
                "type": "dependency",
                "description": "data_len依赖于data_field的实际长度",
                "source_field": "data_len",
                "target_field": "data_field",
                "logic": "data_len = length(data_field)"
            },
            {
                "type": "boundary_detection",
                "description": "根据data_len确定包边界",
                "length_field": "data_len",
                "data_field": "data_field",
                "calculation": "data_len"
            }
        ]
    }
    "#;

    println!("\n=== 测试标准 JSON 规则解析 ===\n");

    // 解析 JSON
    let result = JsonParser::parse_standard_ccsds_json(json_with_rules, "test_package");
    assert!(
        result.is_ok(),
        "解析失败: {:?}",
        result.err()
    );

    let package = result.unwrap();
    
    println!("✓ 成功解析包定义: {}", package.name);
    println!("  - 字段数: {}", package.layers[0].units.len());
    println!("  - 规则数: {}", package.layers[0].rules.len());

    // 验证字段数量
    assert_eq!(package.layers[0].units.len(), 4, "应该解析出4个字段");
    
    // 验证规则数量
    assert_eq!(package.layers[0].rules.len(), 4, "应该解析出4个规则");

    // 验证各类规则
    let rules = &package.layers[0].rules;
    
    println!("\n规则详情:");
    for (i, rule) in rules.iter().enumerate() {
        println!("  [{}] {:?}", i + 1, rule);
    }

    // 验证固定值验证规则
    assert!(
        rules.iter().any(|r| matches!(r, apdl_core::SemanticRule::Validation { 
            field_name, 
            algorithm, 
            .. 
        } if field_name == "sync_flag" && algorithm == "fixed_value")),
        "应该包含固定值验证规则"
    );

    // 验证序列控制规则
    assert!(
        rules.iter().any(|r| matches!(r, apdl_core::SemanticRule::SequenceControl { 
            field_name, 
            algorithm,
            .. 
        } if field_name == "seq_count" && algorithm == "monotonic_increase")),
        "应该包含序列控制规则"
    );

    // 验证依赖规则
    assert!(
        rules.iter().any(|r| matches!(r, apdl_core::SemanticRule::Dependency { 
            dependent_field,
            dependency_field,
        } if dependent_field == "data_len" && dependency_field == "data_field")),
        "应该包含依赖规则"
    );

    // 验证长度规则（从boundary_detection转换）
    assert!(
        rules.iter().any(|r| matches!(r, apdl_core::SemanticRule::LengthRule { 
            field_name,
            .. 
        } if field_name == "data_len")),
        "应该包含长度规则"
    );

    println!("\n✓ 所有规则类型验证通过！\n");
}

#[test]
fn test_parse_ccsds_tm_frame_rules() {
    use std::fs;

    println!("\n=== 测试 CCSDS TM Frame 规则解析 ===\n");

    // 读取实际的 CCSDS TM Frame JSON 文件
    let json = fs::read_to_string("d:/user/yqd/project/apdl/code/resources/standard/ccsds_tm_frame.json")
        .expect("Failed to read ccsds_tm_frame.json");

    let result = JsonParser::parse_standard_ccsds_json(&json, "tm_frame");
    assert!(
        result.is_ok(),
        "解析 TM Frame 失败: {:?}",
        result.err()
    );

    let package = result.unwrap();
    
    println!("✓ 成功解析 CCSDS TM Frame");
    println!("  - 包名: {}", package.name);
    println!("  - 字段数: {}", package.layers[0].units.len());
    println!("  - 规则数: {}", package.layers[0].rules.len());

    assert!(
        !package.layers[0].rules.is_empty(),
        "TM Frame 应该包含规则定义"
    );

    println!("\n规则列表:");
    for (i, rule) in package.layers[0].rules.iter().enumerate() {
        match rule {
            apdl_core::SemanticRule::Validation { field_name, algorithm, description, .. } => {
                println!("  [{}] Validation - 字段: {}, 算法: {}, 描述: {}", 
                    i + 1, field_name, algorithm, description);
            }
            apdl_core::SemanticRule::SequenceControl { field_name, algorithm, description, .. } => {
                println!("  [{}] SequenceControl - 字段: {}, 算法: {}, 描述: {}", 
                    i + 1, field_name, algorithm, description);
            }
            apdl_core::SemanticRule::LengthRule { field_name, expression } => {
                println!("  [{}] LengthRule - 字段: {}, 表达式: {}", 
                    i + 1, field_name, expression);
            }
            _ => {
                println!("  [{}] {:?}", i + 1, rule);
            }
        }
    }

    println!("\n✓ CCSDS TM Frame 规则解析成功！\n");
}
