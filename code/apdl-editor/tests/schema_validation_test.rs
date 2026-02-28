//! APDL Protocol Schema 验证测试
//!
//! 使用jsonschema库进行完整的Schema验证

use jsonschema::{Draft, JSONSchema};
use std::fs;
use std::path::Path;

/// 加载并编译Schema
fn load_schema() -> JSONSchema {
    let schema_path = Path::new("schema/apdl-protocol-schema-v1.json");
    let content = fs::read_to_string(schema_path).expect("无法读取Schema文件");
    let schema_json: serde_json::Value = serde_json::from_str(&content)
        .expect("Schema文件不是有效的JSON");
    
    JSONSchema::options()
        .with_draft(Draft::Draft7)
        .compile(&schema_json)
        .expect("Schema编译失败")
}

/// 测试Schema文件是否存在且为有效JSON
#[test]
fn test_schema_file_exists_and_valid_json() {
    let schema_path = Path::new("schema/apdl-protocol-schema-v1.json");
    assert!(schema_path.exists(), "Schema文件不存在");
    
    let content = fs::read_to_string(schema_path).expect("无法读取Schema文件");
    let json: serde_json::Value = serde_json::from_str(&content)
        .expect("Schema文件不是有效的JSON");
    
    // 验证必需的顶级字段
    assert!(json.get("$schema").is_some(), "缺少$schema字段");
    assert!(json.get("$id").is_some(), "缺少$id字段");
    assert!(json.get("title").is_some(), "缺少title字段");
    assert!(json.get("type").is_some(), "缺少type字段");
    assert!(json.get("definitions").is_some(), "缺少definitions字段");
}

/// 使用jsonschema验证CCSDS TM Frame示例
#[test]
fn test_ccsds_tm_frame_schema_validation() {
    let schema = load_schema();
    
    let example_path = Path::new("schema/examples/ccsds_tm_frame.json");
    let content = fs::read_to_string(example_path).expect("无法读取示例文件");
    let example: serde_json::Value = serde_json::from_str(&content)
        .expect("示例文件不是有效的JSON");
    
    let result = schema.validate(&example);
    if let Err(errors) = result {
        let error_messages: Vec<String> = errors
            .map(|e| format!("{}: {}", e.instance_path, e))
            .collect();
        panic!("Schema验证失败:\n{}", error_messages.join("\n"));
    }
}

/// 使用jsonschema验证CCSDS Full Stack示例
#[test]
fn test_ccsds_full_stack_schema_validation() {
    let schema = load_schema();
    
    let example_path = Path::new("schema/examples/ccsds_full_stack.json");
    let content = fs::read_to_string(example_path).expect("无法读取示例文件");
    let example: serde_json::Value = serde_json::from_str(&content)
        .expect("示例文件不是有效的JSON");
    
    let result = schema.validate(&example);
    if let Err(errors) = result {
        let error_messages: Vec<String> = errors
            .map(|e| format!("{}: {}", e.instance_path, e))
            .collect();
        panic!("Schema验证失败:\n{}", error_messages.join("\n"));
    }
}

/// 测试Schema包含所有必需的定义
#[test]
fn test_schema_has_required_definitions() {
    let schema_path = Path::new("schema/apdl-protocol-schema-v1.json");
    let content = fs::read_to_string(schema_path).expect("无法读取Schema文件");
    let json: serde_json::Value = serde_json::from_str(&content).unwrap();
    
    let definitions = json.get("definitions").unwrap().as_object().unwrap();
    
    let required_defs = vec![
        "ProtocolMeta",
        "SyntaxUnit",
        "FieldType",
        "Constraint",
        "FrameStructure",
        "FrameSection",
        "SemanticRule",
        "PackUnpackSpec",
        "FieldPackSpec",
        "FramePacketRelationship",
        "PlacementConfig",
        "HeaderPointerConfig",
        "MultiplexingConfig",
        "FieldMapping",
        "FieldMappingEntry",
        "EnumMappingEntry",
        "MaskMappingEntry",
        "ProtocolHierarchy",
        "LayerDefinition",
        "ParallelGroup",
    ];
    
    for def in required_defs {
        assert!(
            definitions.contains_key(def),
            "缺少必需的定义: {}",
            def
        );
    }
}

/// 测试示例文件CCSDS TM Frame是否有效
#[test]
fn test_ccsds_tm_frame_example_valid() {
    let schema_path = Path::new("schema/apdl-protocol-schema-v1.json");
    let example_path = Path::new("schema/examples/ccsds_tm_frame.json");
    
    assert!(example_path.exists(), "示例文件不存在");
    
    let schema_content = fs::read_to_string(schema_path).unwrap();
    let example_content = fs::read_to_string(example_path).unwrap();
    
    let schema: serde_json::Value = serde_json::from_str(&schema_content).unwrap();
    let example: serde_json::Value = serde_json::from_str(&example_content).unwrap();
    
    // 验证示例包含必需的字段
    assert!(example.get("protocol_meta").is_some(), "缺少protocol_meta");
    assert!(example.get("syntax_units").is_some(), "缺少syntax_units");
    
    // 验证protocol_meta结构
    let meta = example.get("protocol_meta").unwrap();
    assert!(meta.get("protocol_id").is_some(), "缺少protocol_id");
    assert!(meta.get("protocol_name").is_some(), "缺少protocol_name");
    assert!(meta.get("version").is_some(), "缺少version");
    
    // 验证syntax_units是数组
    let units = example.get("syntax_units").unwrap().as_array().unwrap();
    assert!(!units.is_empty(), "syntax_units不能为空");
    
    // 验证第一个字段结构
    let first_field = &units[0];
    assert!(first_field.get("field_id").is_some(), "字段缺少field_id");
    assert!(first_field.get("type").is_some(), "字段缺少type");
}

/// 测试示例文件CCSDS Full Stack是否有效
#[test]
fn test_ccsds_full_stack_example_valid() {
    let example_path = Path::new("schema/examples/ccsds_full_stack.json");
    
    assert!(example_path.exists(), "Full Stack示例文件不存在");
    
    let content = fs::read_to_string(example_path).unwrap();
    let example: serde_json::Value = serde_json::from_str(&content).unwrap();
    
    // 验证包含帧包关系
    assert!(
        example.get("frame_packet_relationships").is_some(),
        "缺少frame_packet_relationships"
    );
    
    // 验证包含字段映射
    assert!(example.get("field_mappings").is_some(), "缺少field_mappings");
    
    // 验证包含协议层级
    assert!(
        example.get("protocol_hierarchy").is_some(),
        "缺少protocol_hierarchy"
    );
    
    // 验证帧包关系结构
    let relationships = example
        .get("frame_packet_relationships")
        .unwrap()
        .as_array()
        .unwrap();
    assert!(!relationships.is_empty(), "frame_packet_relationships不能为空");
    
    let first_rel = &relationships[0];
    assert!(
        first_rel.get("relationship_id").is_some(),
        "关系缺少relationship_id"
    );
    assert!(
        first_rel.get("relationship_type").is_some(),
        "关系缺少relationship_type"
    );
    assert!(
        first_rel.get("parent_frame").is_some(),
        "关系缺少parent_frame"
    );
    assert!(
        first_rel.get("child_packet").is_some(),
        "关系缺少child_packet"
    );
    
    // 验证字段映射结构
    let mappings = example.get("field_mappings").unwrap().as_array().unwrap();
    assert!(!mappings.is_empty(), "field_mappings不能为空");
    
    let first_mapping = &mappings[0];
    assert!(
        first_mapping.get("mapping_id").is_some(),
        "映射缺少mapping_id"
    );
    assert!(
        first_mapping.get("source_protocol").is_some(),
        "映射缺少source_protocol"
    );
    assert!(
        first_mapping.get("target_protocol").is_some(),
        "映射缺少target_protocol"
    );
    assert!(
        first_mapping.get("mappings").is_some(),
        "映射缺少mappings数组"
    );
}

/// 测试打包规范混用配置
#[test]
fn test_pack_unpack_spec_mixed_config() {
    let example_path = Path::new("schema/examples/ccsds_full_stack.json");
    let content = fs::read_to_string(example_path).unwrap();
    let example: serde_json::Value = serde_json::from_str(&content).unwrap();
    
    // 获取第一个字段的pack_unpack_spec（如果存在）
    let units = example.get("syntax_units").unwrap().as_array().unwrap();
    
    // 检查是否有字段级别的pack_unpack_spec
    let field_with_spec = units.iter().find(|u| u.get("pack_unpack_spec").is_some());
    
    if let Some(field) = field_with_spec {
        let spec = field.get("pack_unpack_spec").unwrap();
        // 验证字段级别的配置
        if let Some(byte_order) = spec.get("byte_order") {
            let order = byte_order.as_str().unwrap();
            assert!(
                order == "big_endian" || order == "little_endian" || order == "mixed",
                "无效的byte_order: {}",
                order
            );
        }
    }
    
    // 验证示例中的field_level_specs配置
    let pack_spec = example.get("pack_unpack_spec");
    if let Some(spec) = pack_spec {
        if let Some(field_specs) = spec.get("field_level_specs") {
            let specs = field_specs.as_array().unwrap();
            assert!(!specs.is_empty(), "field_level_specs不应为空");
            
            for spec in specs {
                assert!(
                    spec.get("field_id").is_some(),
                    "field_level_spec缺少field_id"
                );
            }
        }
    }
}

/// 测试ProtocolMeta的字段格式
#[test]
fn test_protocol_meta_format() {
    let example_path = Path::new("schema/examples/ccsds_tm_frame.json");
    let content = fs::read_to_string(example_path).unwrap();
    let example: serde_json::Value = serde_json::from_str(&content).unwrap();
    
    let meta = example.get("protocol_meta").unwrap();
    
    // 验证protocol_id格式（大写字母开头）
    let protocol_id = meta.get("protocol_id").unwrap().as_str().unwrap();
    assert!(
        protocol_id.chars().next().unwrap().is_ascii_uppercase(),
        "protocol_id必须以大写字母开头"
    );
    
    // 验证version格式（x.x.x）
    let version = meta.get("version").unwrap().as_str().unwrap();
    let version_parts: Vec<&str> = version.split('.').collect();
    assert_eq!(version_parts.len(), 3, "version格式必须为x.x.x");
    for part in version_parts {
        assert!(part.parse::<u32>().is_ok(), "version各部分必须是数字");
    }
    
    // 验证layer是有效值
    let layer = meta.get("layer").unwrap().as_str().unwrap();
    let valid_layers = ["physical", "data_link", "network", "transport", "application"];
    assert!(
        valid_layers.contains(&layer),
        "layer必须是有效值之一"
    );
}

/// 测试字段ID格式
#[test]
fn test_field_id_format() {
    let example_path = Path::new("schema/examples/ccsds_tm_frame.json");
    let content = fs::read_to_string(example_path).unwrap();
    let example: serde_json::Value = serde_json::from_str(&content).unwrap();
    
    let units = example.get("syntax_units").unwrap().as_array().unwrap();
    
    for unit in units {
        let field_id = unit.get("field_id").unwrap().as_str().unwrap();
        
        // 验证小写字母开头
        assert!(
            field_id.chars().next().unwrap().is_ascii_lowercase(),
            "field_id必须以小写字母开头: {}",
            field_id
        );
        
        // 验证只包含小写字母、数字和下划线
        assert!(
            field_id.chars().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_'),
            "field_id包含无效字符: {}",
            field_id
        );
    }
}

/// 测试语义规则类型有效性
#[test]
fn test_semantic_rule_types() {
    let example_path = Path::new("schema/examples/ccsds_tm_frame.json");
    let content = fs::read_to_string(example_path).unwrap();
    let example: serde_json::Value = serde_json::from_str(&content).unwrap();
    
    let valid_rule_types = [
        "checksum_validation",
        "sequence_control",
        "header_pointer_resolution",
        "constraint_validation",
        "field_mapping",
        "length_calculation",
        "address_resolution",
        "security",
        "redundancy",
    ];
    
    if let Some(rules) = example.get("semantic_rules") {
        for rule in rules.as_array().unwrap() {
            let rule_type = rule.get("rule_type").unwrap().as_str().unwrap();
            assert!(
                valid_rule_types.contains(&rule_type),
                "无效的rule_type: {}",
                rule_type
            );
        }
    }
}

/// 测试映射逻辑类型有效性
#[test]
fn test_mapping_logic_types() {
    let example_path = Path::new("schema/examples/ccsds_full_stack.json");
    let content = fs::read_to_string(example_path).unwrap();
    let example: serde_json::Value = serde_json::from_str(&content).unwrap();
    
    let valid_logic_types = [
        "identity",
        "hash_mod_64",
        "hash_mod_2048",
        "shift_right_8",
        "mask_table",
        "enum_mapping",
        "custom",
    ];
    
    if let Some(mappings) = example.get("field_mappings") {
        for mapping in mappings.as_array().unwrap() {
            if let Some(entries) = mapping.get("mappings") {
                for entry in entries.as_array().unwrap() {
                    let logic = entry.get("mapping_logic").unwrap().as_str().unwrap();
                    assert!(
                        valid_logic_types.contains(&logic),
                        "无效的mapping_logic: {}",
                        logic
                    );
                }
            }
        }
    }
}
