use apdl_poem::{
    dsl::json_parser::JsonParser,
    standard_units::{
        connector::connector_engine::ConnectorEngine, frame_assembler::FrameAssembler,
    },
};

#[test]
fn test_full_stack_integration() {
    println!("Testing full stack integration with parent package, child package, and connector...");

    // 1. 定义子包JSON (Telemetry Packet)
    let child_package_json = r#"
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
                        },
                        {
                            "field_id": "apid",
                            "unit_type": {
                                "Uint": 16
                            },
                            "length": {
                                "size": 2,
                                "unit": "Byte"
                            },
                            "scope": {
                                "Global": "telemetry"
                            },
                            "cover": "EntireField",
                            "constraint": {
                                "Range": [0, 65535]
                            },
                            "alg": null,
                            "associate": [],
                            "desc": "Application Process Identifier"
                        },
                        {
                            "field_id": "length",
                            "unit_type": {
                                "Uint": 16
                            },
                            "length": {
                                "size": 2,
                                "unit": "Byte"
                            },
                            "scope": {
                                "Global": "telemetry"
                            },
                            "cover": "EntireField",
                            "constraint": {
                                "Range": [0, 65535]
                            },
                            "alg": null,
                            "associate": [],
                            "desc": "Packet length"
                        },
                        {
                            "field_id": "data",
                            "unit_type": "RawData",
                            "length": {
                                "size": 0,
                                "unit": "Dynamic"
                            },
                            "scope": {
                                "Global": "telemetry"
                            },
                            "cover": "EntireField",
                            "constraint": null,
                            "alg": null,
                            "associate": [],
                            "desc": "Variable data field"
                        }
                    ],
                    "rules": []
                }
            ]
        }
    "#;

    // 2. 解析子包JSON
    let child_package_result = JsonParser::parse_package(child_package_json);
    assert!(
        child_package_result.is_ok(),
        "Failed to parse child package JSON: {:?}",
        child_package_result.err()
    );
    let child_package = child_package_result.unwrap();
    println!(
        "Successfully parsed child package: {name}",
        name = child_package.name
    );

    // 3. 从解析的包中提取子包字段
    let child_fields = child_package.layers[0].units.clone();
    let len = child_fields.len();
    println!("Extracted {len} child package fields from parsed package");

    // 4. 定义父包JSON (Encapsulating Packet)
    let parent_package_json = r#"
        {
            "name": "encapsulating_packet",
            "display_name": "Encapsulating Packet",
            "package_type": "encapsulation",
            "description": "Encapsulating packet with VCID, length, data and FECF",
            "layers": [
                {
                    "name": "encap_layer",
                    "units": [
                        {
                            "field_id": "vcid",
                            "unit_type": {
                                "Uint": 16
                            },
                            "length": {
                                "size": 2,
                                "unit": "Byte"
                            },
                            "scope": {
                                "Global": "encap"
                            },
                            "cover": "EntireField",
                            "constraint": {
                                "Range": [0, 65535]
                            },
                            "alg": null,
                            "associate": [],
                            "desc": "Virtual Channel ID"
                        },
                        {
                            "field_id": "encap_length",
                            "unit_type": {
                                "Uint": 16
                            },
                            "length": {
                                "size": 2,
                                "unit": "Byte"
                            },
                            "scope": {
                                "Global": "encap"
                            },
                            "cover": "EntireField",
                            "constraint": null,
                            "alg": null,
                            "associate": [],
                            "desc": "Encapsulation length"
                        },
                        {
                            "field_id": "data",
                            "unit_type": "RawData",
                            "length": {
                                "size": 20,
                                "unit": "Byte"
                            },
                            "scope": {
                                "Global": "encap"
                            },
                            "cover": "EntireField",
                            "constraint": null,
                            "alg": null,
                            "associate": [],
                            "desc": "Encapsulated data"
                        },
                        {
                            "field_id": "fecf",
                            "unit_type": {
                                "Uint": 16
                            },
                            "length": {
                                "size": 2,
                                "unit": "Byte"
                            },
                            "scope": {
                                "Global": "encap"
                            },
                            "cover": "EntireField",
                            "constraint": null,
                            "alg": {
                                "XorSum": null
                            },
                            "associate": [],
                            "desc": "Frame Error Control Field"
                        }
                    ],
                    "rules": [
                        {
                            "LengthRule": {
                                "field_name": "encap_length",
                                "expression": "total_length - 2"
                            }
                        },
                        {
                            "ChecksumRange": {
                                "algorithm": "XOR",
                                "start_field": "vcid",
                                "end_field": "data"
                            }
                        }
                    ]
                }
            ]
        }
    "#;

    // 5. 解析父包JSON
    let parent_package_result = JsonParser::parse_package(parent_package_json);
    assert!(
        parent_package_result.is_ok(),
        "Failed to parse parent package JSON: {:?}",
        parent_package_result.err()
    );
    let parent_package = parent_package_result.unwrap();
    let name = &parent_package.name;
    println!("Successfully parsed parent package: {name}");

    // 6. 从解析的包中提取父包字段
    let parent_fields = parent_package.layers[0].units.clone();
    let len = parent_fields.len();
    println!("Extracted {len} parent package fields from parsed package");

    // 2. 定义连接器JSON (将Telemetry Packet嵌入到Encapsulating Packet)
    let connector_json = r#"
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
                    },
                    {
                        "source_field": "length",
                        "target_field": "encap_length",
                        "mapping_logic": "identity",
                        "default_value": "0",
                        "enum_mappings": null
                    }
                ],
                "header_pointers": null,
                "data_placement": {
                    "strategy": "Direct",
                    "target_field": "data",
                    "config_params": [
                        ["source_field", "data"],
                        ["target_field", "data"]
                    ]
                }
            },
            "description": "Maps telemetry packet fields to encap packet fields and embeds telemetry data"
        }
    "#;

    // 3. 解析连接器JSON
    let connector_result = JsonParser::parse_connector(connector_json);
    assert!(
        connector_result.is_ok(),
        "Failed to parse connector JSON: {:?}",
        connector_result.err()
    );
    let connector_definition = connector_result.unwrap();
    let name = &connector_definition.name;
    println!("Successfully parsed connector definition: {name}");

    // 4. 创建FrameAssembler实例并添加字段定义
    let mut child_assembler = FrameAssembler::new();
    let mut parent_assembler = FrameAssembler::new();

    // 将字段添加到组装器
    for unit in &child_fields {
        child_assembler.add_field(unit.clone());
    }
    let len = child_assembler.fields.len();
    println!("Added {len} child package fields to assembler");

    for unit in &parent_fields {
        parent_assembler.add_field(unit.clone());
    }
    let len = parent_assembler.fields.len();
    println!("Added {len} parent package fields to assembler");

    // 添加父包的语义规则到assembler
    for rule in &parent_package.layers[0].rules {
        parent_assembler.add_semantic_rule(rule.clone());
    }
    let len = parent_package.layers[0].rules.len();
    println!("Added {len} semantic rules to parent assembler");

    // 5. 设置子包字段值
    // 注意：如果字段在定义中具有FixedValue约束，则无需显式调用set_field_value
    child_assembler.set_field_value("version", &[0x01]).unwrap(); // Version 1
    child_assembler
        .set_field_value("apid", &[0x01, 0x3B])
        .unwrap(); // APID 315
    child_assembler
        .set_field_value("length", &[0x00, 0x0A])
        .unwrap(); // Length 10 bytes
    child_assembler
        .set_field_value("data", &[0xDE, 0xAD, 0xBE, 0xEF, 0xCA, 0xFE])
        .unwrap(); // Sample data
    println!("Set child packet field values");

    // 6. 组装子包帧
    let child_frame = child_assembler.assemble_frame().unwrap();
    let len = child_frame.len();
    println!("Child frame assembled, length: {len} bytes");

    // 7. 设置父包字段值
    // 注意：如果字段在定义中具有FixedValue约束，则无需显式调用set_field_value
    parent_assembler
        .set_field_value("vcid", &[0x00, 0x00])
        .unwrap(); // Will be updated by mapping
    parent_assembler
        .set_field_value("encap_length", &[0x00, 0x00])
        .unwrap(); // Will be calculated
    parent_assembler
        .set_field_value("data", &[0x00; 20])
        .unwrap(); // Will be updated with child data (20 bytes)
    parent_assembler
        .set_field_value("fecf", &[0x00, 0x00])
        .unwrap(); // Will be calculated
    println!("Set initial parent packet field values");

    // 8. 使用连接器引擎执行完整的连接操作
    let mut connector_engine = ConnectorEngine::new();
    println!("Created connector engine");

    // 9. 使用连接器引擎应用字段映射和数据放置
    connector_engine
        .connect(
            &mut child_assembler,
            &mut parent_assembler,
            "test_channel",
            &connector_definition.config,
        )
        .expect("Failed to connect packages");
    println!("Applied field mapping and data placement via connector engine");

    // 10. 构建包含子包数据的父包
    let placement_config = connector_definition
        .config
        .data_placement
        .as_ref()
        .expect("Data placement config should be present");

    let (parent_frame_data, _dispatch_flag) = connector_engine
        .build_packet(placement_config)
        .expect("Failed to build parent packet");

    let len = parent_frame_data.len();
    println!("Parent frame assembled, length: {len} bytes");

    // 11. 验证结果
    assert!(
        !parent_frame_data.is_empty(),
        "Parent frame should not be empty"
    );

    // 父包应该至少包含：vcid(2) + encap_length(2) + 数据区(20) + fecf(2) = 26字节
    let len = parent_frame_data.len();
    assert!(
        parent_frame_data.len() >= 26,
        "Parent frame ({len} bytes) should be at least 26 bytes (headers + data + footer)"
    );

    // 12. 验证父包字段映射是否正确
    println!("\n=== 验证父包字段映射 ===");

    // 验证vcid字段（应该映射自apid = [1, 59]）
    let vcid_value = ((parent_frame_data[0] as u16) << 8) | (parent_frame_data[1] as u16);
    let expected_vcid = 0x013B; // [1, 59] = 0x013B
    assert_eq!(
        vcid_value, expected_vcid,
        "VCID should be mapped from APID [1, 59] = 0x{expected_vcid:04X}, got 0x{vcid_value:04X}"
    );
    println!("✓ VCID correctly mapped from APID: 0x{vcid_value:04X}");

    // 验证encap_length字段（由语义规则自动计算：total_length - 2）
    let encap_length_value = ((parent_frame_data[2] as u16) << 8) | (parent_frame_data[3] as u16);
    let expected_encap_length = 24; // 26 (total_length) - 2 = 24
    assert_eq!(
        encap_length_value, expected_encap_length,
        "Encap length should be calculated by semantic rule (total_length - 2) = {expected_encap_length}, got {encap_length_value}"
    );
    let total_len = parent_frame_data.len();
    println!("✓ Encap length correctly calculated by semantic rule: {encap_length_value} (total_length {total_len} - 2)");

    // 13. 验证父包是否包含子包内容
    println!("\n=== 验证父包包含子包内容 ===");

    // 打印完整的父包内容
    let len = parent_frame_data.len();
    println!("父包完整内容 ({len} bytes):");
    print!("  Hex: ");
    for (i, byte) in parent_frame_data.iter().enumerate() {
        print!("{byte:02X} ");
        if (i + 1) % 16 == 0 {
            print!("\n       ");
        }
    }
    println!();

    // 打印子包内容
    let len = child_frame.len();
    println!("\n子包完整内容 ({len} bytes):");
    print!("  Hex: ");
    for (i, byte) in child_frame.iter().enumerate() {
        print!("{byte:02X} ");
        if (i + 1) % 16 == 0 {
            print!("\n       ");
        }
    }
    println!();

    // 数据字段从偏移4开始（vcid(2) + encap_length(2)）
    let data_offset = 4;
    let _data_field_size = 20;

    // 检查子包数据是否嵌入到父包的data字段中
    let embedded_data = &parent_frame_data[data_offset..data_offset + child_frame.len()];

    let len = child_frame.len();
    println!("\n父包data字段中嵌入的内容 ({len} bytes):");
    print!("  Hex: ");
    for (i, byte) in embedded_data.iter().enumerate() {
        print!("{byte:02X} ");
        if (i + 1) % 16 == 0 {
            print!("\n       ");
        }
    }
    println!();

    assert_eq!(
        embedded_data,
        child_frame.as_slice(),
        "Parent packet's data field should contain the complete child packet"
    );
    println!("✓ 父包的data字段正确包含了完整的子包数据");

    // 14. 简化的验证总结
    println!("\n=== 测试总结 ===");
    println!("✓ Full stack integration test passed!");
    let child_len = child_frame.len();
    let parent_len = parent_frame_data.len();
    println!("  - Child frame: {child_len} bytes");
    println!("  - Parent frame: {parent_len} bytes");
    println!("Parent packet correctly contains child packet with proper field mappings and data embedding.");
}
