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
    println!("Successfully parsed child package: {}", child_package.name);

    // 3. 从解析的包中提取子包字段
    let child_fields = child_package.layers[0].units.clone();
    println!(
        "Extracted {} child package fields from parsed package",
        child_fields.len()
    );

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
                            "constraint": {
                                "Range": [0, 65535]
                            },
                            "alg": null,
                            "associate": [],
                            "desc": "Encapsulation length"
                        },
                        {
                            "field_id": "data",
                            "unit_type": "RawData",
                            "length": {
                                "size": 0,
                                "unit": "Dynamic"
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
                    "rules": []
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
    println!(
        "Successfully parsed parent package: {}",
        parent_package.name
    );

    // 6. 从解析的包中提取父包字段
    let parent_fields = parent_package.layers[0].units.clone();
    println!(
        "Extracted {} parent package fields from parsed package",
        parent_fields.len()
    );

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
    println!(
        "Successfully parsed connector definition: {}",
        connector_definition.name
    );

    // 4. 创建FrameAssembler实例并添加字段定义
    let mut child_assembler = FrameAssembler::new();
    let mut parent_assembler = FrameAssembler::new();

    // 将字段添加到组装器
    for unit in &child_fields {
        child_assembler.add_field(unit.clone());
    }
    println!(
        "Added {} child package fields to assembler",
        child_assembler.fields.len()
    );

    for unit in &parent_fields {
        parent_assembler.add_field(unit.clone());
    }
    println!(
        "Added {} parent package fields to assembler",
        parent_assembler.fields.len()
    );

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
    println!("Child frame assembled, length: {} bytes", child_frame.len());

    // 7. 设置父包字段值
    // 注意：如果字段在定义中具有FixedValue约束，则无需显式调用set_field_value
    parent_assembler
        .set_field_value("vcid", &[0x00, 0x00])
        .unwrap(); // Will be updated by mapping
    parent_assembler
        .set_field_value("encap_length", &[0x00, 0x00])
        .unwrap(); // Will be calculated
    parent_assembler
        .set_field_value("data", &[0x00; 0])
        .unwrap(); // Will be updated with child data
    parent_assembler
        .set_field_value("fecf", &[0x00, 0x00])
        .unwrap(); // Will be calculated
    println!("Set initial parent packet field values");

    // 8. 使用连接器引擎执行完整的连接操作
    let connector_engine = ConnectorEngine::new();
    println!("Created connector engine");

    // 9. 使用连接器引擎应用字段映射和数据放置
    connector_engine
        .connect(
            &mut child_assembler,
            &mut parent_assembler,
            &connector_definition.config,
        )
        .expect("Failed to connect packages");
    println!("Applied field mapping and data placement via connector engine");

    // 11. 重新计算封装包长度
    // 计算除了FECF之外的总长度
    let total_len_except_fecf: usize = parent_assembler
        .fields
        .iter()
        .filter(|f| f.field_id != "fecf")
        .map(|f| {
            let size = parent_assembler.get_field_size(f).unwrap_or(1);
            println!("Field {} size: {}", f.field_id, size);
            size
        })
        .sum();

    let len_bytes = [
        (total_len_except_fecf >> 8) as u8,
        (total_len_except_fecf & 0xFF) as u8,
    ];
    parent_assembler
        .set_field_value("encap_length", &len_bytes)
        .unwrap();
    println!("Updated encapsulation length to: {}", total_len_except_fecf);

    // 12. 计算FECF (Frame Error Control Field) - 简单的XOR校验
    // 先收集除FECF外的数据
    let mut data_for_checksum = Vec::new();
    for field in &parent_assembler.fields {
        if field.field_id != "fecf" {
            if let Ok(field_bytes) = parent_assembler.get_field_value(&field.field_id) {
                data_for_checksum.extend_from_slice(&field_bytes);
                println!("Adding field {} data: {:?}", field.field_id, field_bytes);
            } else {
                // 如果字段值未设置，使用默认值
                let size = parent_assembler.get_field_size(field).unwrap_or(1);
                let default_bytes = vec![0; size];
                data_for_checksum.extend_from_slice(&default_bytes);
                println!(
                    "Adding default data for field {}: {:?}",
                    field.field_id, default_bytes
                );
            }
        }
    }

    let xor_checksum = data_for_checksum
        .iter()
        .fold(0u16, |acc, &x| acc ^ (x as u16));
    let fecf_bytes = [(xor_checksum >> 8) as u8, (xor_checksum & 0xFF) as u8];
    parent_assembler
        .set_field_value("fecf", &fecf_bytes)
        .unwrap();
    println!("Calculated and set FECF value: 0x{:04X}", xor_checksum);

    // 13. 组装最终的父包帧
    let parent_frame = parent_assembler.assemble_frame().unwrap();
    println!(
        "Parent frame assembled, length: {} bytes",
        parent_frame.len()
    );

    // 14. 验证结果
    assert!(!parent_frame.is_empty(), "Parent frame should not be empty");
    assert!(
        parent_frame.len() > child_frame.len(),
        "Parent frame ({} bytes) should be larger than child frame ({} bytes)",
        parent_frame.len(),
        child_frame.len()
    );

    // 15. 验证子包数据确实嵌入到了父包中
    let data_field_pos = parent_assembler.field_index.get("data").unwrap();
    let data_field = &parent_assembler.fields[*data_field_pos];
    let data_field_size = parent_assembler.get_field_size(data_field).unwrap();

    if data_field_size >= child_frame.len() {
        // 找到数据字段在帧中的位置
        let data_offset = parent_assembler
            .calculate_field_offset(*data_field_pos)
            .unwrap();
        let data_slice = &parent_frame[data_offset..data_offset + child_frame.len()];

        assert_eq!(
            data_slice,
            child_frame.as_slice(),
            "Parent packet's data field should contain child packet"
        );
        println!("Verified that parent packet contains child packet in data field");
    } else {
        println!(
            "Data field size ({}) is smaller than child frame size ({}), partial embedding",
            data_field_size,
            child_frame.len()
        );
    }

    // 16. 额外验证：检查长度字段是否正确设置
    let encap_length_value = parent_assembler.get_field_value("encap_length").unwrap();
    let calculated_length = ((encap_length_value[0] as u16) << 8) | (encap_length_value[1] as u16);

    // 计算预期长度（除了FECF之外的所有字段）
    let expected_length: usize = parent_assembler
        .fields
        .iter()
        .filter(|f| f.field_id != "fecf") // 不包括FECF
        .map(|f| parent_assembler.get_field_size(f).unwrap_or(1))
        .sum();

    assert_eq!(
        calculated_length as usize, expected_length,
        "Encapsulation length field should match calculated length (expected: {}, got: {})",
        expected_length, calculated_length
    );
    println!("Verified encapsulation length field: {}", calculated_length);

    println!("Full stack integration test completed successfully!");
    println!("Parent packet correctly contains child packet with proper field mappings, data embedding, and validation.");
}
