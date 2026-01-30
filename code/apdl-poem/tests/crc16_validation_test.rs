use apdl_poem::dsl::DslParserImpl;
use apdl_poem::FrameAssembler;

#[test]
fn test_ccsds_crc16_checksum_calculation() {
    println!("=== 测试CCSDS CRC16校验和计算 ===");

    // 使用与集成测试相同的CCSDS协议定义
    let dsl_content = r#"
    field: sync_flag; type: Uint16; length: 2byte; scope: layer(physical); cover: entire_field; constraint: fixed(0xEB90); desc: "CCSDS同步标志";
    field: version; type: Uint8; length: 1byte; scope: layer(data_link); cover: entire_field; constraint: range(0..=7); desc: "版本号";
    field: type_flag; type: Uint8; length: 1byte; scope: layer(data_link); cover: entire_field; constraint: range(0..=1); desc: "类型标志";
    field: sh_flag; type: Uint8; length: 1byte; scope: layer(data_link); cover: entire_field; constraint: range(0..=1); desc: "次标题标志";
    field: apid; type: Uint16; length: 2byte; scope: layer(data_link); cover: entire_field; constraint: range(0..=2047); desc: "应用进程ID";
    field: seq_flags; type: Uint8; length: 1byte; scope: layer(data_link); cover: entire_field; constraint: range(0..=3); desc: "序列标志";
    field: seq_count; type: Uint16; length: 2byte; scope: layer(data_link); cover: entire_field; desc: "序列计数";
    field: data_len; type: Uint16; length: 2byte; scope: layer(data_link); cover: entire_field; desc: "数据长度";
    field: data_field; type: RawData; length: dynamic; scope: layer(application); cover: entire_field; desc: "数据域";
    field: fecf; type: Uint16; length: 2byte; scope: layer(data_link); cover: entire_field; alg: Crc16; desc: "帧错误控制字段";

    // 定义各种语义规则
    rule: crc_range(start: sync_flag to data_field); // CRC校验范围 (使用真正的CRC16算法)
    rule: routing_dispatch(field: apid; algorithm: hash_apid_to_route; desc: "根据APID进行分路");
    rule: sequence_control(field: seq_count; trigger: on_transmission; algorithm: increment_seq; desc: "序列控制");
    rule: validation(field: fecf; algorithm: crc16_verification; range: from(sync_flag) to(data_field); desc: "CRC验证");
    rule: synchronization(field: sync_flag; algorithm: sync_pattern_match; desc: "同步识别");
    rule: length_validation(field: data_len; condition: equals_data_field_plus_header_minus_one; desc: "长度验证");
    rule: dependency(field: seq_count depends_on apid); // 序列号依赖于APID
    rule: order(first: sync_flag before data_field); // 顺序规则
    rule: pointer(field: data_len points_to data_field); // 指针语义
    rule: algorithm(field: fecf uses crc16); // 算法规则
    rule: length_rule(field: data_len equals "(total_length - 10)"); // 长度规则
    "#;

    // 解析协议定义
    let parser = DslParserImpl::new();
    let syntax_units = parser
        .parse_protocol_structure(dsl_content)
        .expect("解析语法单元失败");
    let semantic_rules = parser
        .parse_semantic_rules(dsl_content)
        .expect("解析语义规则失败");

    // 创建帧组装器
    let mut assembler = FrameAssembler::new();

    // 添加字段和语义规则
    for unit in &syntax_units {
        assembler.add_field(unit.clone());
    }
    for rule in &semantic_rules {
        assembler.add_semantic_rule(rule.clone());
    }

    // 设置字段值
    assembler
        .set_field_value("sync_flag", &[0xEB, 0x90])
        .unwrap();
    assembler.set_field_value("version", &[0x00]).unwrap();
    assembler.set_field_value("apid", &[0x01, 0x00]).unwrap(); // APID = 256
    assembler
        .set_field_value("data_field", &[0xCA, 0xFE, 0xBA, 0xBE])
        .unwrap();
    assembler.set_field_value("fecf", &[0xFF, 0xFF]).unwrap(); // 初始值，将被CRC算法覆盖

    // 组装帧
    let frame = assembler.assemble_frame().expect("帧组装失败");
    println!("组装的帧: {:02X?}", frame);

    // 帧应该为18字节: [EB, 90, 00, 00, 00, 01, 00, 00, 00, 00, 0x00, 0x08, CA, FE, BA, BE, 05, FC]
    assert_eq!(frame.len(), 18, "帧长度应该是18字节");

    // 验证帧内容
    // 现在使用真正的CRC16算法，校验和为0x05FC
    let expected_frame = vec![
        0xEB, 0x90, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x08, 0xCA, 0xFE, 0xBA,
        0xBE, 0x05, 0xFC,
    ];
    assert_eq!(frame, expected_frame, "帧内容应该匹配预期值");

    // 提取待校验的数据部分（除了最后2个字节的校验和）
    let data_to_check = &frame[0..frame.len() - 2]; // 除了最后2个字节

    // 对于CRC16校验，我们需要重新计算CRC16
    let calculated_checksum = calculate_crc16_checksum(data_to_check);

    // 获取帧中存储的CRC值（最后2个字节）
    let stored_checksum = [frame[frame.len() - 2], frame[frame.len() - 1]];

    println!("待校验数据: {:02X?}", data_to_check);
    println!("计算出的CRC16校验和: {:02X?}", calculated_checksum);
    println!("存储的CRC16校验和: {:02X?}", stored_checksum);

    // 验证计算出的校验和与存储的校验和匹配
    assert_eq!(
        calculated_checksum, stored_checksum,
        "计算出的CRC16校验和应该与存储的校验和匹配"
    );

    // 额外验证：检查计算的校验和值是否确实是0x05FC（十进制1532）
    let checksum_value = ((calculated_checksum[0] as u16) << 8) + (calculated_checksum[1] as u16);
    assert_eq!(checksum_value, 1532, "校验和值应该是1532（十进制）");

    println!("✓ CRC16校验和计算验证通过！");
    println!("  - 待校验数据长度: {} 字节", data_to_check.len());
    println!(
        "  - 计算出的CRC16校验和: 0x{:04X} ({})",
        checksum_value, checksum_value
    );
    println!("  - CRC16校验通过: 数据完整性得到验证");
}

// 计算CRC16校验和
fn calculate_crc16_checksum(data: &[u8]) -> [u8; 2] {
    let mut crc: u16 = 0xFFFF;
    for &byte in data {
        crc ^= (byte as u16) << 8;
        for _ in 0..8 {
            if (crc & 0x8000) != 0 {
                crc = (crc << 1) ^ 0x1021;
            } else {
                crc <<= 1;
            }
        }
    }
    [(crc >> 8) as u8, crc as u8]
}

// 计算XOR校验和（逐字节异或）
fn calculate_xor_checksum(data: &[u8]) -> [u8; 2] {
    let mut xor_sum: u16 = 0;

    for &byte in data {
        xor_sum ^= byte as u16;
    }

    // 将结果转换为2字节格式
    [(xor_sum >> 8) as u8, xor_sum as u8]
}

// 计算累积XOR校验和（每个字节依次异或）
fn calculate_cumulative_xor(data: &[u8]) -> [u8; 2] {
    let mut xor_result: u16 = 0;

    for &byte in data {
        xor_result ^= byte as u16;
    }

    // 将结果转换为2字节格式
    [(xor_result >> 8) as u8, xor_result as u8]
}

#[test]
fn test_xor_checksum_manual_calculation() {
    // 手动测试CRC16计算
    let data = [
        0xEB, 0x90, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x08, 0xCA, 0xFE, 0xBA,
        0xBE,
    ];
    let calculated = calculate_cumulative_xor(&data);
    println!("手动计算XOR校验和测试:");
    println!("输入数据: {:02X?}", data);
    println!("计算结果: {:02X?}", calculated);

    // 验证计算结果是否为 [0x00, 0x42] (66的十六进制)
    assert_eq!(calculated, [0x00, 0x42]);
    println!("✓ 手动XOR校验和计算验证通过！");
}
