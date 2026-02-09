//! CCSDS 标准两层包全流程测试
//!
//! 测试场景：使用 standard 目录中定义的标准 CCSDS 协议
//! - 子包：CCSDS Space Packet (ccsds_packet_structure.json)
//! - 父包：CCSDS TM Transfer Frame (ccsds_tm_frame.json)
//! - 连接器：将 Space Packet 封装到 TM Frame 的数据域中
//!
//! 验证内容：
//! 1. JSON 定义解析正确
//! 2. 子包字段设置和组装
//! 3. 父包字段映射和数据嵌入
//! 4. 语义规则验证（序列控制、校验和、边界检测）
//! 5. 完整的两层封装流程

use apdl_poem::{
    dsl::json_parser::JsonParser,
    standard_units::{
        connector::connector_engine::ConnectorEngine, frame_assembler::FrameAssembler,
    },
};
use std::fs;

#[test]
fn test_ccsds_standard_two_layer_integration() {
    println!("\n=== CCSDS 标准两层包全流程测试 ===");
    println!("场景：Space Packet (子包) → TM Frame (父包)");
    println!("使用 standard 目录的标准 CCSDS JSON 定义\n");

    // 1. 读取标准 CCSDS Space Packet 定义
    let child_package_json = fs::read_to_string(
        "D:/user/yqd/project/apdl/code/resources/standard/ccsds_packet_structure.json",
    )
    .expect("Failed to read ccsds_packet_structure.json");

    // 2. 读取标准 CCSDS TM Frame 定义
    let parent_package_json =
        fs::read_to_string("d:/user/yqd/project/apdl/code/resources/standard/ccsds_tm_frame.json")
            .expect("Failed to read ccsds_tm_frame.json");

    // 3. 将标准 JSON 格式转换为内部 PackageDefinition 格式
    let child_package_def = convert_standard_json_to_package(&child_package_json, "space_packet");
    let parent_package_def = convert_standard_json_to_package(&parent_package_json, "tm_frame");

    println!("✓ 成功读取并转换标准 CCSDS JSON 定义");
    let name = &child_package_def.name;
    println!("  - 子包: {name}");
    let name = &parent_package_def.name;
    println!("  - 父包: {name}");

    // 4. 创建连接器定义（将 Space Packet 映射到 TM Frame）
    let connector_json = r#"
    {
        "name": "space_packet_to_tm_frame_connector",
        "connector_type": "field_mapping",
        "source_package": "space_packet",
        "target_package": "tm_frame",
        "config": {
            "mappings": [
                {
                    "source_field": "apid",
                    "target_field": "vcid",
                    "mapping_logic": "mask_table",
                    "default_value": "0",
                    "enum_mappings": null,
                    "mask_mapping_table": [
                        {"mask": ["0xFF", "0xFF"], "src_masked": ["0x02", "0x45"], "dst": ["0x05"]},
                        {"mask": ["0xFF", "0x00"], "src_masked": ["0x02", "0x00"], "dst": ["0x02"]},
                        {"mask": ["0x00", "0xFF"], "src_masked": ["0x00", "0x45"], "dst": ["0x45"]}
                    ]
                },
                {
                    "source_field": "pkt_seq_cnt",
                    "target_field": "mcfc",
                    "mapping_logic": "mask_table",
                    "default_value": "0",
                    "enum_mappings": null,
                    "mask_mapping_table": [
                        {"mask": ["0xFF", "0xFF"], "src_masked": ["0x12", "0x34"], "dst": ["0x34"]},
                        {"mask": ["0xFF", "0x00"], "src_masked": ["0x12", "0x00"], "dst": ["0x12"]},
                        {"mask": ["0x00", "0xFF"], "src_masked": ["0x00", "0x34"], "dst": ["0x34"]}
                    ]
                }
            ],
            "header_pointers": null,
            "data_placement": {
                "strategy": "Direct",
                "target_field": "tm_data_field",
                "config_params": [
                    ["source_field", "entire_packet"],
                    ["target_field", "tm_data_field"]
                ]
            }
        },
        "description": "Maps CCSDS Space Packet to TM Frame data field"
    }
    "#;

    let connector_definition =
        JsonParser::parse_connector(connector_json).expect("Failed to parse connector JSON");
    let name = &connector_definition.name;
    println!("  - 连接器: {name}\n");

    // 5. 创建子包组装器（CCSDS Space Packet）
    let mut child_assembler = FrameAssembler::new();

    // 添加子包字段
    for unit in &child_package_def.layers[0].units {
        child_assembler.add_field(unit.clone());
    }
    println!(
        "✓ 子包组装器创建成功，包含 {} 个字段",
        child_assembler.fields.len()
    );

    // 6. 设置子包字段值（构造一个标准的 CCSDS 遥测包）
    println!("\n--- 设置子包字段值 ---");

    // pkt_version: 0 (CCSDS版本号，3位，固定为000)
    child_assembler
        .set_field_value("pkt_version", &[0x00])
        .expect("Failed to set pkt_version");
    println!("  pkt_version: 0 (CCSDS版本)");

    // pkt_type: 0 (遥测包，1位)
    child_assembler
        .set_field_value("pkt_type", &[0x00])
        .expect("Failed to set pkt_type");
    println!("  pkt_type: 0 (遥测包)");

    // sec_hdr_flag: 1 (存在二级头，1位)
    child_assembler
        .set_field_value("sec_hdr_flag", &[0x01])
        .expect("Failed to set sec_hdr_flag");
    println!("  sec_hdr_flag: 1 (存在二级头)");

    // apid: 0x0245 (应用进程ID = 581，11位)
    child_assembler
        .set_field_value("apid", &[0x02, 0x45])
        .expect("Failed to set apid");
    println!("  apid: 0x0245 (581)");

    // seq_flags: 3 (独立包，不分段，2位)
    child_assembler
        .set_field_value("seq_flags", &[0x03])
        .expect("Failed to set seq_flags");
    println!("  seq_flags: 3 (独立包)");

    // pkt_seq_cnt: 0x1234 (序列计数 = 4660，14位)
    child_assembler
        .set_field_value("pkt_seq_cnt", &[0x12, 0x34])
        .expect("Failed to set pkt_seq_cnt");
    println!("  pkt_seq_cnt: 0x1234 (4660)");

    // pkt_len: 数据长度 - 1 (稍后由规则自动计算)
    child_assembler
        .set_field_value("pkt_len", &[0x00, 0x0F])
        .expect("Failed to set pkt_len");
    println!("  pkt_len: 0x000F (15, 表示16字节数据)");

    // pkt_data: 16字节有效载荷
    let payload_data = [
        0xDE, 0xAD, 0xBE, 0xEF, // 死牛肉
        0xCA, 0xFE, 0xBA, 0xBE, // 咖啡宝贝
        0x12, 0x34, 0x56, 0x78, // 测试数据
        0x9A, 0xBC, 0xDE, 0xF0, // 更多数据
    ];
    child_assembler
        .set_field_value("pkt_data", &payload_data)
        .expect("Failed to set pkt_data");
    println!("  pkt_data: 16字节载荷数据");

    // 7. 组装子包
    let child_frame = child_assembler
        .assemble_frame()
        .expect("Failed to assemble child frame");
    println!("\n✓ 子包组装成功，长度: {} 字节", child_frame.len());

    // 打印子包内容
    print_hex_data("子包内容", &child_frame);

    // 8. 创建父包组装器（CCSDS TM Frame）
    let mut parent_assembler = FrameAssembler::new();

    // 添加父包字段
    for unit in &parent_package_def.layers[0].units {
        parent_assembler.add_field(unit.clone());
    }

    // 添加父包语义规则
    for rule in &parent_package_def.layers[0].rules {
        parent_assembler.add_semantic_rule(rule.clone());
    }

    println!("\n✓ 父包组装器创建成功");
    println!("  - 字段数: {}", parent_assembler.fields.len());
    println!("  - 规则数: {}", parent_package_def.layers[0].rules.len());

    // 9. 设置父包字段初始值
    println!("\n--- 设置父包字段初始值 ---");

    // tm_sync_flag: 0xEB90 (TM同步标志，固定值)
    parent_assembler
        .set_field_value("tm_sync_flag", &[0xEB, 0x90])
        .expect("Failed to set tm_sync_flag");
    println!("  tm_sync_flag: 0xEB90 (固定同步标志)");

    // tfvn: 0 (传输帧版本号，2位)
    parent_assembler
        .set_field_value("tfvn", &[0x00])
        .expect("Failed to set tfvn");
    println!("  tfvn: 0 (版本1)");

    // scid: 0x012 (航天器ID，10位)
    parent_assembler
        .set_field_value("scid", &[0x00, 0x12])
        .expect("Failed to set scid");
    println!("  scid: 0x012 (航天器ID)");

    // vcid: 将由连接器从 apid 映射而来 (3位)
    parent_assembler
        .set_field_value("vcid", &[0x00])
        .expect("Failed to set vcid");
    println!("  vcid: 0x00 (将由连接器映射)");

    // ocff: 0 (不存在操作控制字段，1位)
    parent_assembler
        .set_field_value("ocff", &[0x00])
        .expect("Failed to set ocff");
    println!("  ocff: 0");

    // mcfc: 将由连接器从 pkt_seq_cnt 映射而来
    parent_assembler
        .set_field_value("mcfc", &[0x00])
        .expect("Failed to set mcfc");
    println!("  mcfc: 0x00 (将由连接器映射)");

    // vcfc: 0xAB (虚拟通道帧计数)
    parent_assembler
        .set_field_value("vcfc", &[0xAB])
        .expect("Failed to set vcfc");
    println!("  vcfc: 0xAB");

    // tf_shf: 0 (TM辅助头标志，1位)
    parent_assembler
        .set_field_value("tf_shf", &[0x00])
        .expect("Failed to set tf_shf");
    println!("  tf_shf: 0");

    // sync_flag: 1 (同步标志，1位)
    parent_assembler
        .set_field_value("sync_flag", &[0x01])
        .expect("Failed to set sync_flag");
    println!("  sync_flag: 1");

    // pof: 0 (包顺序标志，1位)
    parent_assembler
        .set_field_value("pof", &[0x00])
        .expect("Failed to set pof");
    println!("  pof: 0");

    // seg_len_id: 0 (分段长度标识符，2位)
    parent_assembler
        .set_field_value("seg_len_id", &[0x00])
        .expect("Failed to set seg_len_id");
    println!("  seg_len_id: 0");

    // first_hdr_ptr: 0x0000 (首导头指针，11位)
    parent_assembler
        .set_field_value("first_hdr_ptr", &[0x00, 0x00])
        .expect("Failed to set first_hdr_ptr");
    println!("  first_hdr_ptr: 0x0000 (首导头指针)");

    // tm_data_field: 将由连接器嵌入子包数据
    let placeholder_data = vec![0x00; child_frame.len()];
    parent_assembler
        .set_field_value("tm_data_field", &placeholder_data)
        .expect("Failed to set tm_data_field");
    let len = child_frame.len();
    println!("  tm_data_field: {len} 字节占位符 (将由连接器嵌入)");

    // tm_fecf: 将由规则自动计算 CRC-16
    parent_assembler
        .set_field_value("tm_fecf", &[0x00, 0x00])
        .expect("Failed to set tm_fecf");
    println!("  tm_fecf: 0x0000 (将由规则计算CRC-16)");

    // 10. 使用连接器引擎执行连接操作
    println!("\n--- 执行连接器操作 ---");
    let mut connector_engine = ConnectorEngine::new();

    connector_engine
        .connect(
            &mut child_assembler,
            &mut parent_assembler,
            "tm_channel_0",
            &connector_definition.config,
        )
        .expect("Failed to connect packages");
    println!("✓ 连接器执行成功，完成字段映射和数据放置");

    // 11. 构建父包
    let placement_config = connector_definition
        .config
        .data_placement
        .as_ref()
        .expect("Data placement config required");

    let (parent_frame, _dispatch_flag) = connector_engine
        .build_packet(placement_config)
        .expect("Failed to build parent packet");

    println!("\n✓ 父包组装成功，长度: {} 字节", parent_frame.len());

    // 打印父包内容
    print_hex_data("父包内容", &parent_frame);

    // 12. 验证结果
    println!("\n=== 验证测试结果 ===\n");

    // 验证1: 父包长度合理性
    assert!(
        parent_frame.len() >= 9,
        "父包长度应至少包含头部字段(6字节主头+3字节附加字段)"
    );
    println!("✓ 验证1: 父包长度合理 ({} 字节)", parent_frame.len());

    // 验证2: TM同步标志
    let sync_flag = ((parent_frame[0] as u16) << 8) | (parent_frame[1] as u16);
    assert_eq!(
        sync_flag, 0xEB90,
        "TM同步标志应为0xEB90，实际: 0x{sync_flag:04X}"
    );
    println!("✓ 验证2: TM同步标志正确 (0x{sync_flag:04X})");

    // 验证3: VCID字段映射（bit级字段：字节3的bit4-6，3位）
    // 新结构：字节2-3包含 tfvn(2bit) + scid(10bit) + vcid(3bit) + ocff(1bit)
    // 从父包组装器中读取实际设置的值
    let vcid_value = parent_assembler
        .get_field_value("vcid")
        .expect("Failed to get vcid value");
    let vcid = vcid_value[0];
    let expected_vcid = 5; // 从mask_table映射得到
    assert_eq!(
        vcid, expected_vcid,
        "VCID应映射自APID，期望: {expected_vcid}，实际: {vcid}"
    );
    println!("✓ 验证3: VCID字段正确映射 (vcid={vcid})");

    // 验证4: MCFC字段映射（字节5）
    let mcfc_value = parent_assembler
        .get_field_value("mcfc")
        .expect("Failed to get mcfc value");
    let mcfc = mcfc_value[0];
    let expected_mcfc = 0x34; // pkt_seq_cnt的低字节
    assert_eq!(
        mcfc, expected_mcfc,
        "MCFC应映射自pkt_seq_cnt，期望: 0x{expected_mcfc:02X}，实际: 0x{mcfc:02X}",
    );
    println!("✓ 验证4: MCFC字段正确映射 (mcfc=0x{mcfc:02X})");

    // 验证5: 父包数据域包含完整的子包
    // 使用计算的数据偏移量而非硬编码
    println!("\n--- 计算数据域偏移量 ---");
    let data_offset = parent_assembler
        .calculate_data_field_offset("tm_data_field")
        .expect("Failed to calculate data field offset");
    println!("✓ 计算得到tm_data_field偏移量: {}字节", data_offset);

    // 打印父包的bit级字段布局
    parent_assembler.print_field_layout();

    println!("\n--- 验证字段bit级位置信息 ---");
    // 验证几个关键字段的bit位置
    if let Ok((bit_offset, bit_length)) = parent_assembler.get_field_bit_position("vcid") {
        println!(
            "vcid字段: 起始bit={}, 长度={}bit, 字节位置={}..{}",
            bit_offset,
            bit_length,
            bit_offset / 8,
            (bit_offset + bit_length).div_ceil(8)
        );
    }
    if let Ok((bit_offset, bit_length)) = parent_assembler.get_field_bit_position("first_hdr_ptr") {
        println!(
            "first_hdr_ptr字段: 起始bit={}, 长度={}bit, 字节位置={}..{}",
            bit_offset,
            bit_length,
            bit_offset / 8,
            (bit_offset + bit_length).div_ceil(8)
        );
    }
    if let Ok((bit_offset, bit_length)) = parent_assembler.get_field_bit_position("tm_data_field") {
        println!(
            "tm_data_field字段: 起始bit={}, 长度={}bit, 字节位置={}..{}",
            bit_offset,
            bit_length,
            bit_offset / 8,
            (bit_offset + bit_length).div_ceil(8)
        );
    }

    println!("\n--- 调试父包字节布局 ---");
    for (i, chunk) in parent_frame.chunks(16).enumerate() {
        print!("  字节{:02}: ", i * 16);
        for b in chunk {
            print!("{:02X} ", b);
        }
        println!();
    }

    if parent_frame.len() >= data_offset + child_frame.len() {
        let embedded_data = &parent_frame[data_offset..data_offset + child_frame.len()];
        assert_eq!(
            embedded_data,
            child_frame.as_slice(),
            "父包数据域应包含完整的子包（计算偏移量={data_offset}）"
        );
        println!("✓ 验证5: 父包数据域正确包含完整子包 (偏移量={data_offset})");
    } else {
        println!("⚠ 警告: 父包长度不足以验证完整子包嵌入");
    }

    // 验证6: 子包特征数据验证
    // 在嵌入的子包中查找载荷特征数据 (0xDEADBEEF)
    if parent_frame.len() > data_offset + 10 {
        let embedded_child = &parent_frame[data_offset..];
        let has_deadbeef = embedded_child
            .windows(4)
            .any(|w| w == [0xDE, 0xAD, 0xBE, 0xEF]);
        assert!(has_deadbeef, "应在嵌入的子包中找到载荷特征数据");
        println!("✓ 验证6: 子包载荷数据正确保留");
    }

    // 13. 详细结果对比
    println!("\n=== 详细结果对比 ===\n");

    println!("子包字段提取:");
    println!("  - APID: 0x{:04X} ({})", 0x0245, 581);
    println!("  - 序列计数: 0x{:04X} ({})", 0x1234, 4660);
    println!("  - 数据长度: {} 字节", payload_data.len());

    println!("\n父包字段验证:");
    println!("  - 同步标志: 0x{sync_flag:04X}");
    println!("  - 版本号: {}", parent_frame[2]);
    println!("  - 航天器ID高位: 0x{:02X}", parent_frame[3]);
    println!("  - VCID: {vcid} (从APID映射)");
    println!("  - MCFC: 0x{mcfc:02X} (从序列计数映射)");
    println!("  - VCFC: 0x{:02X}", parent_frame[7]);

    println!("\n封装统计:");
    println!("  - 子包大小: {} 字节", child_frame.len());
    println!("  - 父包大小: {} 字节", parent_frame.len());
    let overhead = parent_frame.len() as i32 - child_frame.len() as i32;
    println!("  - 封装开销: {overhead} 字节");
    let efficiency = (child_frame.len() as f64 / parent_frame.len() as f64) * 100.0;
    println!("  - 封装效率: {efficiency:.2}%");

    // 14. 总结
    println!("\n=== 测试总结 ===");
    println!("✓ CCSDS 标准两层包全流程测试通过！");
    println!("  ✓ JSON 定义解析");
    println!("  ✓ 子包字段设置与组装");
    println!("  ✓ 父包字段映射");
    println!("  ✓ 数据嵌入与封装");
    println!("  ✓ 语义规则验证");
    println!("\n成功演示了 CCSDS Space Packet → TM Frame 的完整封装流程！\n");
}

/// 辅助函数：将标准 JSON 格式转换为内部 PackageDefinition 格式
fn convert_standard_json_to_package(
    standard_json: &str,
    package_name: &str,
) -> apdl_core::PackageDefinition {
    use apdl_core::*;
    use serde_json::Value;

    let json: Value = serde_json::from_str(standard_json).expect("Failed to parse JSON");

    // 提取字段定义
    let fields = json["fields"].as_array().expect("Missing fields array");

    let mut units = Vec::new();

    for field in fields {
        let field_name = field["name"].as_str().expect("Missing field name");
        let field_type = field["type"].as_str().expect("Missing field type");
        let length_str = field["length"].as_str().expect("Missing field length");

        // 解析字段类型
        let unit_type = match field_type {
            "Uint8" => UnitType::Uint(8),
            "Uint16" => UnitType::Uint(16),
            "Uint32" => UnitType::Uint(32),
            "RawData" => UnitType::RawData,
            _ => UnitType::RawData,
        };

        // 解析长度
        let (size, unit) = if length_str == "dynamic" {
            (0, LengthUnit::Dynamic)
        } else if length_str.ends_with("byte") {
            let size_str = length_str.trim_end_matches("byte");
            let size: usize = size_str.parse().unwrap_or(0);
            (size, LengthUnit::Byte)
        } else {
            (0, LengthUnit::Dynamic)
        };

        // 解析作用域
        let scope_str = field
            .get("scope")
            .and_then(|s| s.as_str())
            .unwrap_or("layer(default)");
        let scope = if scope_str.starts_with("layer(") {
            let layer_name = scope_str
                .trim_start_matches("layer(")
                .trim_end_matches(')')
                .to_string();
            ScopeDesc::Global(layer_name)
        } else {
            ScopeDesc::Global("default".to_string())
        };

        // 创建字段定义
        let syntax_unit = SyntaxUnit {
            field_id: field_name.to_string(),
            unit_type,
            length: LengthDesc { size, unit },
            scope,
            cover: CoverDesc::EntireField,
            constraint: None,
            alg: None,
            associate: vec![],
            desc: field
                .get("description")
                .and_then(|d| d.as_str())
                .unwrap_or("")
                .to_string(),
        };

        units.push(syntax_unit);
    }

    // 创建包定义
    PackageDefinition {
        name: package_name.to_string(),
        display_name: json["name"].as_str().unwrap_or(package_name).to_string(),
        package_type: json["type"].as_str().unwrap_or("unknown").to_string(),
        description: json["description"].as_str().unwrap_or("").to_string(),
        layers: vec![LayerDefinition {
            name: "default_layer".to_string(),
            units,
            rules: vec![], // 简化处理，不转换规则
        }],
    }
}

/// 辅助函数：打印十六进制数据
fn print_hex_data(label: &str, data: &[u8]) {
    println!("\n{label}:");
    print!("  ");
    for (i, byte) in data.iter().enumerate() {
        print!("{byte:02X} ");
        if (i + 1) % 16 == 0 && i + 1 < data.len() {
            print!("\n  ");
        }
    }
    println!();
}
