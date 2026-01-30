use apdl_poem::dsl::DslParserImpl;
use apdl_poem::FrameAssembler;

fn main() {
    // 创建一个简单测试来检查data_len字段
    let dsl_content = r#"
    field: sync_flag; type: Uint16; length: 2byte; scope: layer(physical); cover: entire_field; constraint: fixed(0xEB90); desc: "CCSDS同步标志";
    field: version; type: Uint8; length: 1byte; scope: layer(data_link); cover: entire_field; constraint: range(0..=7); desc: "版本号";
    field: apid; type: Uint16; length: 2byte; scope: layer(data_link); cover: entire_field; constraint: range(0..=2047); desc: "应用进程ID";
    field: data_len; type: Uint16; length: 2byte; scope: layer(data_link); cover: entire_field; desc: "数据长度";
    field: data_field; type: RawData; length: dynamic; scope: layer(application); cover: entire_field; desc: "数据域";
    field: fecf; type: Uint16; length: 2byte; scope: layer(data_link); cover: entire_field; alg: Crc16; desc: "帧错误控制字段";

    rule: length_rule(field: data_len equals "(total_length - 10)");
    "#;

    let parser = DslParserImpl::new();
    let syntax_units = parser
        .parse_protocol_structure(dsl_content)
        .expect("解析语法单元失败");
    let semantic_rules = parser
        .parse_semantic_rules(dsl_content)
        .expect("解析语义规则失败");

    let mut assembler = FrameAssembler::new();

    // 添加所有字段
    for unit in &syntax_units {
        assembler.add_field(unit.clone());
    }

    // 添加所有语义规则
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
    assembler.set_field_value("fecf", &[0xFF, 0xFF]).unwrap();

    // 检查data_len字段在组装前的值
    println!(
        "组装前data_len字段值: {:?}",
        assembler.get_field_value("data_len")
    );

    // 组装帧
    let frame = assembler.assemble_frame().expect("帧组装失败");

    // 检查组装后的data_len字段值
    println!(
        "组装后data_len字段值: {:?}",
        assembler.get_field_value("data_len")
    );
    println!("帧长度: {}", frame.len());
    println!("帧内容: {:02X?}", frame);

    // 检查data_len字段在帧中的具体位置
    // 帧格式: sync_flag(2) + version(1) + apid(2) + data_len(2) + data_field(4) + fecf(2) = 13字节
    // 所以data_len应该在位置5-6，对应帧内容中的第3-4个字节之后
    if frame.len() >= 9 {
        println!(
            "data_len字段在帧中的值: [{:02X}, {:02X}]",
            frame[7], frame[8]
        );
        let data_len_value = ((frame[7] as u16) << 8) | (frame[8] as u16);
        println!("解码的data_len值: {}", data_len_value);
        println!("期望的data_len值: {}", frame.len() as u16 - 10);
    }
}
