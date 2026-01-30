use apdl_poem::DslParserImpl;
use apdl_poem::FrameAssembler;

#[test]
fn test_fixed_xor_calculation() {
    println!("=== 测试修复后的XOR校验和计算 ===");

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
    field: fecf; type: Uint16; length: 2byte; scope: layer(data_link); cover: entire_field; alg: XorSum; desc: "帧错误控制字段";

    // 定义各种语义规则
    rule: checksum_range(start: sync_flag to data_field); // XOR校验范围
    rule: routing_dispatch(field: apid; algorithm: hash_apid_to_route; desc: "根据APID进行分路");
    rule: sequence_control(field: seq_count; trigger: on_transmission; algorithm: increment_seq; desc: "序列控制");
    rule: validation(field: fecf; algorithm: xor_verification; range: from(sync_flag) to(data_field); desc: "XOR验证");
    rule: synchronization(field: sync_flag; algorithm: sync_pattern_match; desc: "同步识别");
    rule: length_validation(field: data_len; condition: equals_data_field_plus_header_minus_one; desc: "长度验证");
    rule: dependency(field: seq_count depends_on apid); // 序列号依赖于APID
    rule: order(first: sync_flag before data_field); // 顺序规则
    rule: pointer(field: data_len points_to data_field); // 指针语义
    rule: algorithm(field: fecf uses xor); // 算法规则
    rule: length_rule(field: data_len equals "(total_length - 10)");
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
    assembler.set_field_value("fecf", &[0xFF, 0xFF]).unwrap(); // 初始值，将被XOR算法覆盖

    // 组装帧 - 这里会触发所有的规则处理
    let frame = assembler.assemble_frame().expect("帧组装失败");
    println!("组装的帧: {:02X?}", frame);

    // 验证帧长度
    assert_eq!(frame.len(), 18, "帧长度应该是18字节");

    // 验证帧内容
    // data_len字段应为8（因为total_length - 10 = 18 - 10 = 8）
    // 修复后，XOR校验和基于更新后的data_len值计算，结果为66（0x42）
    let expected_frame = vec![
        0xEB, 0x90, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x08, 0xCA, 0xFE, 0xBA,
        0xBE, 0x00, 0x42,
    ];
    assert_eq!(frame, expected_frame, "帧内容应该匹配预期值");

    // 验证校验和是否正确计算（基于最终帧数据）
    println!("--- 验证校验和计算正确性 ---");

    // 计算基于最终帧数据的理论XOR值
    let final_frame_data = &frame[0..16]; // sync_flag到data_field结束
    let theoretical_xor = calculate_manual_xor(final_frame_data);
    println!(
        "基于最终帧数据的理论XOR值: 0x{:02X} ({})",
        theoretical_xor, theoretical_xor
    );
    println!("实际帧中的XOR值: 0x{:02X} ({})", frame[17], frame[17]); // fecf的低字节

    if theoretical_xor == frame[17] {
        println!("✓ 修复成功：校验和计算基于最新的字段值");
        println!("  - 长度规则先执行，更新了data_len字段为08");
        println!("  - 校验和规则后执行，基于更新后的data_len值计算XOR");
        println!("  - 最终XOR校验和与数据匹配");
    } else {
        println!("✗ 修复失败：校验和计算仍存在问题");
        println!(
            "  - 理论值: 0x{:02X}, 实际值: 0x{:02X}",
            theoretical_xor, frame[17]
        );

        // 显示详细计算过程
        println!("\n详细XOR计算过程:");
        let mut xor_acc: u8 = 0;
        for (i, &byte) in final_frame_data.iter().enumerate() {
            let old_xor = xor_acc;
            xor_acc ^= byte;
            println!(
                "  位置 {}: 0x{:02X} ^ 0x{:02X} = 0x{:02X}",
                i, old_xor, byte, xor_acc
            );
        }
        println!("最终XOR: 0x{:02X}", xor_acc);
    }
}

// 辅助函数：手动计算XOR校验和
fn calculate_manual_xor(data: &[u8]) -> u8 {
    let mut xor_result: u8 = 0;
    for &byte in data {
        xor_result ^= byte;
    }
    xor_result
}
