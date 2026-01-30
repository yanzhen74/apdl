//! APDL连接器功能演示
//!
//! 展示如何使用连接器进行字段映射

use apdl_core::{
    Constraint, CoverDesc, FieldMappingEntry, LengthDesc, LengthUnit, SemanticRule, SyntaxUnit,
    UnitType,
};
use apdl_poem::dsl::parser::DslParserImpl;
use apdl_poem::standard_units::connector::{ConnectorEngine, FieldMapper};

fn main() {
    println!("=== APDL 连接器功能演示 ===\n");

    // 1. 演示DSL解析字段映射规则
    println!("1. DSL解析字段映射规则:");
    let parser = DslParserImpl::new();
    let dsl = r#"rule: field_mapping(source_package: "lower_layer_packet"; target_package: "upper_layer_packet"; mappings: [{source_field: "src_id", target_field: "vcid", mapping_logic: "hash_mod_64", default_value: "0"}]; desc: "Map source ID to VCID")"#;

    match parser.parse_semantic_rules(dsl) {
        Ok(rules) => {
            println!("   成功解析 {} 条规则", rules.len());
            for rule in &rules {
                if let SemanticRule::FieldMapping {
                    source_package,
                    target_package,
                    mappings,
                    description,
                } = rule
                {
                    println!(
                        "   源包: {}, 目标包: {}, 描述: {}",
                        source_package, target_package, description
                    );
                    println!("   包含 {} 个映射条目:", mappings.len());
                    for mapping in mappings {
                        println!(
                            "     源字段: {} -> 目标字段: {}, 映射逻辑: {}",
                            mapping.source_field, mapping.target_field, mapping.mapping_logic
                        );
                    }
                }
            }
        }
        Err(e) => {
            eprintln!("解析失败: {}", e);
        }
    }

    // 2. 演示字段映射器功能
    println!("\n2. 字段映射器功能演示:");
    let mapper = FieldMapper::new();
    println!("   注册了 {} 个映射函数", mapper.mapping_functions.len());

    let test_input = vec![0x12, 0x34, 0x56];
    let result = mapper.map_field(&test_input, "hash_mod_64").unwrap();
    println!(
        "   输入: {:02X?}, 经过 hash_mod_64 映射后: {:02X?}",
        test_input, result
    );

    let result2 = mapper.map_field(&test_input, "identity").unwrap();
    println!(
        "   输入: {:02X?}, 经过 identity 映射后: {:02X?}",
        test_input, result2
    );

    // 3. 演示连接器引擎功能
    println!("\n3. 连接器引擎功能演示:");
    let mut engine = ConnectorEngine::new();

    // 创建示例源包和目标包
    let source_package = vec![SyntaxUnit {
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
    }];

    let mut target_package = vec![SyntaxUnit {
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
    }];

    // 创建映射规则
    let mapping_rule = SemanticRule::FieldMapping {
        source_package: "telemetry_packet".to_string(),
        target_package: "encapsulating_packet".to_string(),
        mappings: vec![FieldMappingEntry {
            source_field: "src_id".to_string(),
            target_field: "vcid".to_string(),
            mapping_logic: "hash_mod_64".to_string(),
            default_value: "0".to_string(),
        }],
        description: "Map telemetry source to VCID".to_string(),
    };

    engine.add_mapping_rule(mapping_rule);
    println!("   添加了字段映射规则");

    // 应用映射规则
    match engine.apply_mapping_rules(&source_package, &mut target_package) {
        Ok(_) => {
            println!("   成功应用映射规则");
            println!("   目标包字段:");
            for field in &target_package {
                println!("     字段: {}, 类型: {:?}", field.field_id, field.unit_type);
            }
        }
        Err(e) => {
            eprintln!("   应用规则失败: {}", e);
        }
    }

    println!("\n=== 演示完成 ===");
}
