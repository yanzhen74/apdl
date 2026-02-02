//! 语义规则验证测试
//!
//! 验证所有语义规则在组帧、解帧过程中的有效性

use apdl_core::{ProtocolError, SemanticRule};
use apdl_poem::dsl::parser::DslParserImpl;
use apdl_poem::standard_units::frame_assembler::FrameAssembler;

#[test]
fn test_all_semantic_rules_validation() -> Result<(), ProtocolError> {
    println!("=== 开始验证所有语义规则 ===\n");

    // 创建DSL解析器实例
    let parser = DslParserImpl::new();

    // 定义包含所有语义规则类型的协议定义
    let protocol_definition = r#"
    // 字段定义
    field: sync_flag; type: Uint16; length: 2byte; scope: layer(physical); cover: entire_field; constraint: fixed(0xEB90); desc: "同步标志";
    field: version; type: Uint8; length: 1byte; scope: layer(data_link); cover: entire_field; constraint: range(0..=7); desc: "版本号";
    field: type_flag; type: Uint8; length: 1byte; scope: layer(data_link); cover: entire_field; constraint: range(0..=1); desc: "类型标志";
    field: apid; type: Uint16; length: 2byte; scope: layer(data_link); cover: entire_field; constraint: range(0..=2047); desc: "应用进程ID";
    field: seq_flags; type: Uint8; length: 1byte; scope: layer(data_link); cover: entire_field; constraint: range(0..=3); desc: "序列标志";
    field: seq_count; type: Uint16; length: 2byte; scope: layer(data_link); cover: entire_field; desc: "序列计数";
    field: data_len; type: Uint16; length: 2byte; scope: layer(data_link); cover: entire_field; desc: "数据长度";
    field: data_field; type: RawData; length: dynamic; scope: layer(application); cover: entire_field; desc: "数据域";
    field: fecf; type: Uint16; length: 2byte; scope: layer(data_link); cover: entire_field; desc: "帧错误控制字段";

    // 基础语义规则定义
    rule: checksum_range(start: sync_flag to data_field);  // 校验范围规则
    rule: dependency(field: seq_count depends_on apid); // 依赖规则
    rule: conditional(fieldC if apid.value == 0x0001); // 条件规则
    rule: order(first: sync_flag before data_field); // 顺序规则
    rule: pointer(field: data_len points_to data_field); // 指针规则
    rule: algorithm(field: fecf uses custom_checksum); // 算法规则
    rule: length_rule(field: data_len equals "(total_length - 10)"); // 长度规则

    // CCSDS特有规则
    rule: routing_dispatch(field: apid; algorithm: hash_apid_to_route; desc: "根据APID进行分路");
    rule: sequence_control(field: seq_count; trigger: on_transmission; algorithm: increment_seq; desc: "序列控制");
    rule: validation(field: fecf; algorithm: crc16_verification; range: from(sync_flag) to(data_field); desc: "验证规则");
    rule: synchronization(field: sync_flag; algorithm: sync_pattern_match; desc: "同步规则");
    rule: length_validation(field: data_len; condition: equals_remaining; desc: "长度验证");

    // CAN特有规则
    rule: multiplexing(field: data_field; condition: data_type_check; route: handler_a; desc: "多路复用");
    rule: priority_processing(field: sync_flag; algorithm: priority_arb; desc: "优先级处理");
    rule: state_machine(condition: idle_state; algorithm: transition_ready; desc: "状态机");
    rule: periodic_transmission(field: data_field; condition: interval_check; algorithm: send_periodic; desc: "周期传输");
    rule: message_filtering(condition: filter_cond; action: accept_msg; desc: "消息过滤");
    rule: error_detection(algorithm: detect_errors; desc: "错误检测");

    // 通用规则
    rule: flow_control(field: sync_flag; algorithm: flow_ctl_alg; desc: "流量控制");
    rule: time_synchronization(field: sync_flag; algorithm: time_sync_alg; desc: "时间同步");
    rule: address_resolution(field: data_field; algorithm: addr_res_alg; desc: "地址解析");
    rule: security(field: data_field; algorithm: encrypt_alg; desc: "安全规则");
    rule: redundancy(field: sync_flag; algorithm: redundancy_alg; desc: "冗余规则");

    // 连接器规则
    rule: field_mapping(source_package: "lower_layer"; target_package: "upper_layer"; mappings: [{source_field: "apid", target_field: "vcid", mapping_logic: "identity", default_value: "0", enum_mappings: []}]; desc: "字段映射规则")
    "#;

    println!("1. 解析协议定义...");
    // 解析语法单元（字段定义）
    let syntax_units = parser.parse_protocol_structure(protocol_definition)?;
    // 解析语义规则
    let semantic_rules = parser.parse_semantic_rules(protocol_definition)?;

    println!("   - 成功解析 {} 个语法单元", syntax_units.len());
    println!("   - 成功解析 {} 个语义规则", semantic_rules.len());

    println!("\n2. 创建帧组装器并添加字段和规则...");
    let mut assembler = FrameAssembler::new();

    // 添加字段定义
    for unit in syntax_units {
        assembler.add_field(unit);
    }

    // 添加语义规则
    for rule in semantic_rules {
        assembler.add_semantic_rule(rule);
    }

    // 设置一些字段值
    assembler.set_field_value("sync_flag", &[0xEB, 0x90])?;
    assembler.set_field_value("version", &[0x01])?;
    assembler.set_field_value("type_flag", &[0x00])?;
    assembler.set_field_value("apid", &[0x00, 0x01])?; // 设置为0x0001以满足条件规则
    assembler.set_field_value("seq_flags", &[0x03])?;
    assembler.set_field_value("seq_count", &[0x00, 0x01])?;
    assembler.set_field_value("data_len", &[0x00, 0x05])?;
    assembler.set_field_value("data_field", &[0xDE, 0xAD, 0xBE, 0xEF, 0x00])?;

    println!("\n3. 执行帧组装...");
    let frame = assembler.assemble_frame()?;
    println!("   - 成功组装帧，长度: {} 字节", frame.len());
    println!(
        "   - 帧内容: {:02X?}",
        &frame[..std::cmp::min(20, frame.len())]
    ); // 只打印前20字节

    println!("\n4. 执行帧解析...");
    let parsed_fields = assembler.parse_frame(&frame)?;
    println!("   - 成功解析 {} 个字段", parsed_fields.len());

    for (field_name, field_value) in &parsed_fields {
        println!("   - {field_name}: {field_value:02X?}");
    }

    println!("\n5. 验证规则处理结果...");
    // 验证序列号是否已增加（通过序列控制规则）
    if let Ok(seq_count_value) = assembler.get_field_value("seq_count") {
        println!("   - 序列号字段值: {seq_count_value:02X?}");
    }

    println!("\n=== 所有语义规则验证完成 ===");
    Ok(())
}

#[test]
fn test_individual_semantic_rules() -> Result<(), ProtocolError> {
    println!("\n=== 开始逐个验证语义规则 ===\n");

    let parser = DslParserImpl::new();

    // 测试ChecksumRange规则
    {
        println!("测试ChecksumRange规则...");
        let dsl = r#"
        field: sync_flag; type: Uint16; length: 2byte; scope: layer(physical); cover: entire_field; constraint: fixed(0xEB90); desc: "同步标志";
        field: data_field; type: RawData; length: 4byte; scope: layer(application); cover: entire_field; desc: "数据域";
        field: checksum; type: Uint16; length: 2byte; scope: layer(data_link); cover: entire_field; desc: "校验和";
        rule: checksum_range(start: sync_flag to data_field);
        "#;

        let _syntax_units = parser.parse_protocol_structure(dsl)?;
        let semantic_rules = parser.parse_semantic_rules(dsl)?;
        assert_eq!(semantic_rules.len(), 1);

        if let SemanticRule::ChecksumRange {
            algorithm: _,
            start_field,
            end_field,
        } = &semantic_rules[0]
        {
            assert_eq!(start_field, "sync_flag");
            assert_eq!(end_field, "data_field");
            println!("   ✓ ChecksumRange规则解析正确");
        } else {
            panic!("Expected ChecksumRange rule");
        }
    }

    // 测试Dependency规则
    {
        println!("测试Dependency规则...");
        let dsl = r#"
        field: apid; type: Uint16; length: 2byte; scope: layer(data_link); cover: entire_field; desc: "应用进程ID";
        field: seq_count; type: Uint16; length: 2byte; scope: layer(data_link); cover: entire_field; desc: "序列计数";
        rule: dependency(field: seq_count depends_on apid);
        "#;

        let _syntax_units = parser.parse_protocol_structure(dsl)?;
        let semantic_rules = parser.parse_semantic_rules(dsl)?;
        assert_eq!(semantic_rules.len(), 1);

        if let SemanticRule::Dependency {
            dependent_field,
            dependency_field,
        } = &semantic_rules[0]
        {
            assert_eq!(dependent_field, "seq_count");
            assert_eq!(dependency_field, "apid");
            println!("   ✓ Dependency规则解析正确");
        } else {
            panic!("Expected Dependency rule");
        }
    }

    // 测试Order规则
    {
        println!("测试Order规则...");
        let dsl = r#"
        field: sync_flag; type: Uint16; length: 2byte; scope: layer(physical); cover: entire_field; desc: "同步标志";
        field: data_field; type: RawData; length: 4byte; scope: layer(application); cover: entire_field; desc: "数据域";
        rule: order(first: sync_flag before data_field);
        "#;

        let _syntax_units = parser.parse_protocol_structure(dsl)?;
        let semantic_rules = parser.parse_semantic_rules(dsl)?;
        assert_eq!(semantic_rules.len(), 1);

        if let SemanticRule::Order {
            first_field,
            second_field,
        } = &semantic_rules[0]
        {
            assert_eq!(first_field, "sync_flag");
            assert_eq!(second_field, "data_field");
            println!("   ✓ Order规则解析正确");
        } else {
            panic!("Expected Order rule");
        }
    }

    // 测试Validation规则
    {
        println!("测试Validation规则...");
        let dsl = r#"
        field: sync_flag; type: Uint16; length: 2byte; scope: layer(physical); cover: entire_field; desc: "同步标志";
        field: data_field; type: RawData; length: 4byte; scope: layer(application); cover: entire_field; desc: "数据域";
        field: checksum; type: Uint16; length: 2byte; scope: layer(data_link); cover: entire_field; desc: "校验和";
        rule: validation(field: checksum; algorithm: crc16_verification; range: from(sync_flag) to(data_field); desc: "CRC验证");
        "#;

        let _syntax_units = parser.parse_protocol_structure(dsl)?;
        let semantic_rules = parser.parse_semantic_rules(dsl)?;
        assert_eq!(semantic_rules.len(), 1);

        if let SemanticRule::Validation {
            field_name,
            algorithm,
            range_start,
            range_end,
            description,
        } = &semantic_rules[0]
        {
            assert_eq!(field_name, "checksum");
            assert_eq!(algorithm, "crc16_verification");
            assert_eq!(range_start, "sync_flag");
            assert_eq!(range_end, "data_field");
            assert_eq!(description, "CRC验证");
            println!("   ✓ Validation规则解析正确");
        } else {
            panic!("Expected Validation rule");
        }
    }

    // 测试FieldMapping规则
    {
        println!("测试FieldMapping规则...");
        let dsl = r#"
        rule: field_mapping(source_package: "source_pkt"; target_package: "target_pkt"; mappings: [{source_field: "src_fid", target_field: "tgt_fid", mapping_logic: "hash_mod_64", default_value: "0", enum_mappings: []}]; desc: "字段映射示例")
        "#;

        let _syntax_units = parser.parse_protocol_structure(dsl)?;
        let semantic_rules = parser.parse_semantic_rules(dsl)?;
        assert_eq!(semantic_rules.len(), 1);

        if let SemanticRule::FieldMapping {
            source_package,
            target_package,
            mappings,
            description,
        } = &semantic_rules[0]
        {
            assert_eq!(source_package, "source_pkt");
            assert_eq!(target_package, "target_pkt");
            assert_eq!(description, "字段映射示例");
            assert_eq!(mappings.len(), 1);
            assert_eq!(mappings[0].source_field, "src_fid");
            assert_eq!(mappings[0].target_field, "tgt_fid");
            assert_eq!(mappings[0].mapping_logic, "hash_mod_64");
            assert_eq!(mappings[0].default_value, "0");
            println!("   ✓ FieldMapping规则解析正确");
        } else {
            panic!("Expected FieldMapping rule");
        }
    }

    println!("\n=== 逐个语义规则验证完成 ===");
    Ok(())
}

#[test]
fn test_rule_integration_with_frame_assembly() -> Result<(), ProtocolError> {
    println!("\n=== 测试规则与帧组装集成 ===\n");

    let parser = DslParserImpl::new();
    let mut assembler = FrameAssembler::new();

    // 定义带有多种规则的协议
    let dsl = r#"
    field: sync_flag; type: Uint16; length: 2byte; scope: layer(physical); cover: entire_field; constraint: fixed(0xEB90); desc: "同步标志";
    field: data_field; type: RawData; length: dynamic; scope: layer(application); cover: entire_field; desc: "数据域";
    field: checksum; type: Uint16; length: 2byte; scope: layer(data_link); cover: entire_field; desc: "校验和";
    field: length_field; type: Uint16; length: 2byte; scope: layer(data_link); cover: entire_field; desc: "长度字段";
    
    rule: checksum_range(start: sync_flag to data_field);  // 校验范围规则
    rule: length_rule(field: length_field equals "4");    // 长度规则
    rule: order(first: sync_flag before data_field);      // 顺序规则
    "#;

    // 解析并添加到组装器
    let syntax_units = parser.parse_protocol_structure(dsl)?;
    let semantic_rules = parser.parse_semantic_rules(dsl)?;

    for unit in syntax_units {
        assembler.add_field(unit);
    }

    for rule in semantic_rules {
        assembler.add_semantic_rule(rule);
    }

    // 设置字段值
    assembler.set_field_value("sync_flag", &[0xEB, 0x90])?;
    assembler.set_field_value("data_field", &[0xDE, 0xAD, 0xBE, 0xEF])?;

    // 组装帧
    let frame = assembler.assemble_frame()?;
    println!("组装的帧长度: {} 字节", frame.len());
    println!("组装的帧内容: {frame:02X?}");

    // 解析帧
    let parsed_fields = assembler.parse_frame(&frame)?;
    println!("解析的字段数量: {}", parsed_fields.len());

    for (field_name, field_value) in parsed_fields {
        println!("  {field_name}: {field_value:02X?}");
    }

    println!("\n=== 规则与帧组装集成测试完成 ===");
    Ok(())
}
