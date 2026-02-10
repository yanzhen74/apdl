//! 分层拆包引擎集成测试
//!
//! 测试完整的嵌套协议栈拆包流程

use apdl_core::*;
use apdl_lsk::{FrameDisassembler, LayeredDisassembler};

/// 创建CCSDS TM Frame拆包器
fn create_tm_frame_disassembler() -> FrameDisassembler {
    let mut disassembler = FrameDisassembler::new();

    // TM帧头部（6字节）
    let version_field = SyntaxUnit {
        field_id: "tm_version".to_string(),
        unit_type: UnitType::Bit(2),
        length: LengthDesc {
            size: 2,
            unit: LengthUnit::Bit,
        },
        scope: ScopeDesc::Global("TM Frame".to_string()),
        cover: CoverDesc::EntireField,
        constraint: None,
        alg: None,
        associate: vec![],
        desc: "TM Version".to_string(),
    };

    let scid_field = SyntaxUnit {
        field_id: "tm_scid".to_string(),
        unit_type: UnitType::Bit(10),
        length: LengthDesc {
            size: 10,
            unit: LengthUnit::Bit,
        },
        scope: ScopeDesc::Global("TM Frame".to_string()),
        cover: CoverDesc::EntireField,
        constraint: None,
        alg: None,
        associate: vec![],
        desc: "Spacecraft ID".to_string(),
    };

    let vcid_field = SyntaxUnit {
        field_id: "tm_vcid".to_string(),
        unit_type: UnitType::Bit(4),
        length: LengthDesc {
            size: 4,
            unit: LengthUnit::Bit,
        },
        scope: ScopeDesc::Global("TM Frame".to_string()),
        cover: CoverDesc::EntireField,
        constraint: None,
        alg: None,
        associate: vec![],
        desc: "Virtual Channel ID".to_string(),
    };

    let frame_seq_field = SyntaxUnit {
        field_id: "tm_frame_seq".to_string(),
        unit_type: UnitType::Uint(32),
        length: LengthDesc {
            size: 4,
            unit: LengthUnit::Byte,
        },
        scope: ScopeDesc::Global("TM Frame".to_string()),
        cover: CoverDesc::EntireField,
        constraint: None,
        alg: None,
        associate: vec![],
        desc: "Frame Sequence Number".to_string(),
    };

    // TM数据字段（净荷）
    let tm_data_field = SyntaxUnit {
        field_id: "tm_data_field".to_string(),
        unit_type: UnitType::RawData,
        length: LengthDesc {
            size: 0,
            unit: LengthUnit::Dynamic,
        },
        scope: ScopeDesc::Global("TM Frame".to_string()),
        cover: CoverDesc::EntireField,
        constraint: None,
        alg: None,
        associate: vec![],
        desc: "TM Data Field".to_string(),
    };

    disassembler.add_field(version_field);
    disassembler.add_field(scid_field);
    disassembler.add_field(vcid_field);
    disassembler.add_field(frame_seq_field);
    disassembler.add_field(tm_data_field);

    disassembler
}

/// 创建CCSDS Space Packet拆包器
fn create_space_packet_disassembler() -> FrameDisassembler {
    let mut disassembler = FrameDisassembler::new();

    // Space Packet Primary Header（6字节）
    let pkt_version_field = SyntaxUnit {
        field_id: "pkt_version".to_string(),
        unit_type: UnitType::Bit(3),
        length: LengthDesc {
            size: 3,
            unit: LengthUnit::Bit,
        },
        scope: ScopeDesc::Global("Space Packet".to_string()),
        cover: CoverDesc::EntireField,
        constraint: None,
        alg: None,
        associate: vec![],
        desc: "Packet Version".to_string(),
    };

    let pkt_type_field = SyntaxUnit {
        field_id: "pkt_type".to_string(),
        unit_type: UnitType::Bit(1),
        length: LengthDesc {
            size: 1,
            unit: LengthUnit::Bit,
        },
        scope: ScopeDesc::Global("Space Packet".to_string()),
        cover: CoverDesc::EntireField,
        constraint: None,
        alg: None,
        associate: vec![],
        desc: "Packet Type".to_string(),
    };

    let sec_hdr_flag_field = SyntaxUnit {
        field_id: "pkt_sec_hdr_flag".to_string(),
        unit_type: UnitType::Bit(1),
        length: LengthDesc {
            size: 1,
            unit: LengthUnit::Bit,
        },
        scope: ScopeDesc::Global("Space Packet".to_string()),
        cover: CoverDesc::EntireField,
        constraint: None,
        alg: None,
        associate: vec![],
        desc: "Secondary Header Flag".to_string(),
    };

    let apid_field = SyntaxUnit {
        field_id: "pkt_apid".to_string(),
        unit_type: UnitType::Bit(11),
        length: LengthDesc {
            size: 11,
            unit: LengthUnit::Bit,
        },
        scope: ScopeDesc::Global("Space Packet".to_string()),
        cover: CoverDesc::EntireField,
        constraint: None,
        alg: None,
        associate: vec![],
        desc: "Application Process ID".to_string(),
    };

    let seq_flags_field = SyntaxUnit {
        field_id: "pkt_seq_flags".to_string(),
        unit_type: UnitType::Bit(2),
        length: LengthDesc {
            size: 2,
            unit: LengthUnit::Bit,
        },
        scope: ScopeDesc::Global("Space Packet".to_string()),
        cover: CoverDesc::EntireField,
        constraint: None,
        alg: None,
        associate: vec![],
        desc: "Sequence Flags".to_string(),
    };

    let pkt_seq_cnt_field = SyntaxUnit {
        field_id: "pkt_seq_cnt".to_string(),
        unit_type: UnitType::Bit(14),
        length: LengthDesc {
            size: 14,
            unit: LengthUnit::Bit,
        },
        scope: ScopeDesc::Global("Space Packet".to_string()),
        cover: CoverDesc::EntireField,
        constraint: None,
        alg: None,
        associate: vec![],
        desc: "Packet Sequence Count".to_string(),
    };

    let pkt_len_field = SyntaxUnit {
        field_id: "pkt_len".to_string(),
        unit_type: UnitType::Uint(16),
        length: LengthDesc {
            size: 2,
            unit: LengthUnit::Byte,
        },
        scope: ScopeDesc::Global("Space Packet".to_string()),
        cover: CoverDesc::EntireField,
        constraint: None,
        alg: None,
        associate: vec![],
        desc: "Packet Length".to_string(),
    };

    // 包数据（净荷）
    let pkt_data_field = SyntaxUnit {
        field_id: "pkt_data".to_string(),
        unit_type: UnitType::RawData,
        length: LengthDesc {
            size: 0,
            unit: LengthUnit::Dynamic,
        },
        scope: ScopeDesc::Global("Space Packet".to_string()),
        cover: CoverDesc::EntireField,
        constraint: None,
        alg: None,
        associate: vec![],
        desc: "Packet Data".to_string(),
    };

    disassembler.add_field(pkt_version_field);
    disassembler.add_field(pkt_type_field);
    disassembler.add_field(sec_hdr_flag_field);
    disassembler.add_field(apid_field);
    disassembler.add_field(seq_flags_field);
    disassembler.add_field(pkt_seq_cnt_field);
    disassembler.add_field(pkt_len_field);
    disassembler.add_field(pkt_data_field);

    disassembler
}

#[test]
fn test_two_layer_tm_frame_and_space_packet() {
    println!("\n=== 测试：TM Frame -> Space Packet 两层拆包 ===");

    // 创建分层拆包器
    let mut layered = LayeredDisassembler::new();

    // 添加TM Frame层
    let tm_disassembler = create_tm_frame_disassembler();
    layered.add_layer(
        "TM Frame".to_string(),
        tm_disassembler,
        Some("tm_data_field".to_string()),
    );

    // 添加Space Packet层
    let sp_disassembler = create_space_packet_disassembler();
    layered.add_layer(
        "Space Packet".to_string(),
        sp_disassembler,
        Some("pkt_data".to_string()),
    );

    // 构造测试数据
    // TM Frame Header (6字节):
    //   version(2bit)=0, scid(10bit)=0x3FF, vcid(4bit)=1
    //   前16位：00 1111111111 0001 = 0x3FF1
    //   frame_seq(32bit)=0x00000001
    // Space Packet Header (6字节):
    //   version(3bit)=0, type(1bit)=0, sec_hdr_flag(1bit)=1, apid(11bit)=0x123
    //   前16位：000 0 1 00100100011 = 0x0923
    //   seq_flags(2bit)=3, seq_cnt(14bit)=100
    //   接下来16位：11 00000001100100 = 0xC064
    //   pkt_len(16bit)=7 (表示数据区8字节)
    // Application Data (8字节):
    //   0x0102030405060708

    let test_frame = vec![
        // TM Frame Header (6字节)
        0x3F, 0xF1, // version=0, scid=0x3FF, vcid=1
        0x00, 0x00, 0x00, 0x01, // frame_seq=1
        // Space Packet Header (6字节)
        0x09, 0x23, // version=0, type=0, sec_hdr=1, apid=0x123
        0xC0, 0x64, // seq_flags=3, seq_cnt=100
        0x00, 0x07, // pkt_len=7 (表示数据区8字节)
        // Application Data (8字节)
        0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08,
    ];

    // 完整拆包
    let result = layered.disassemble_layers(&test_frame).unwrap();

    // 打印结果
    result.print();

    // 验证层数
    assert_eq!(result.layer_count(), 2);

    // 验证TM Frame层
    let tm_layer = result.get_layer(0).unwrap();
    assert_eq!(tm_layer.layer_name, "TM Frame");

    let scid = tm_layer.get_field("tm_scid").unwrap();
    // scid是10位bit字段，提取为2字节大端格式
    let scid_value = ((scid[0] as u16) << 8) | (scid[1] as u16);
    assert_eq!(scid_value, 0x03FF);

    let vcid = tm_layer.get_field("tm_vcid").unwrap();
    assert_eq!(vcid[0], 0x01);

    // 验证Space Packet层
    let sp_layer = result.get_layer(1).unwrap();
    assert_eq!(sp_layer.layer_name, "Space Packet");

    let apid = sp_layer.get_field("pkt_apid").unwrap();
    let apid_value = ((apid[0] as u16) << 8) | (apid[1] as u16);
    assert_eq!(apid_value, 0x0123);

    // 验证应用数据
    assert_eq!(
        result.application_data,
        vec![0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08]
    );
}

#[test]
fn test_three_layer_protocol_stack() {
    println!("\n=== 测试：三层协议栈拆包 ===");

    // 创建分层拆包器
    let mut layered = LayeredDisassembler::new();

    // 第一层：简单外层协议（4字节头部）
    let mut outer_disassembler = FrameDisassembler::new();
    let outer_header = SyntaxUnit {
        field_id: "outer_header".to_string(),
        unit_type: UnitType::Uint(32),
        length: LengthDesc {
            size: 4,
            unit: LengthUnit::Byte,
        },
        scope: ScopeDesc::Global("Outer".to_string()),
        cover: CoverDesc::EntireField,
        constraint: None,
        alg: None,
        associate: vec![],
        desc: "Outer Header".to_string(),
    };
    let outer_payload = SyntaxUnit {
        field_id: "outer_payload".to_string(),
        unit_type: UnitType::RawData,
        length: LengthDesc {
            size: 0,
            unit: LengthUnit::Dynamic,
        },
        scope: ScopeDesc::Global("Outer".to_string()),
        cover: CoverDesc::EntireField,
        constraint: None,
        alg: None,
        associate: vec![],
        desc: "Outer Payload".to_string(),
    };
    outer_disassembler.add_field(outer_header);
    outer_disassembler.add_field(outer_payload);

    layered.add_layer(
        "Outer Layer".to_string(),
        outer_disassembler,
        Some("outer_payload".to_string()),
    );

    // 第二层：中间层协议（2字节头部）
    let mut middle_disassembler = FrameDisassembler::new();
    let middle_header = SyntaxUnit {
        field_id: "middle_header".to_string(),
        unit_type: UnitType::Uint(16),
        length: LengthDesc {
            size: 2,
            unit: LengthUnit::Byte,
        },
        scope: ScopeDesc::Global("Middle".to_string()),
        cover: CoverDesc::EntireField,
        constraint: None,
        alg: None,
        associate: vec![],
        desc: "Middle Header".to_string(),
    };
    let middle_payload = SyntaxUnit {
        field_id: "middle_payload".to_string(),
        unit_type: UnitType::RawData,
        length: LengthDesc {
            size: 0,
            unit: LengthUnit::Dynamic,
        },
        scope: ScopeDesc::Global("Middle".to_string()),
        cover: CoverDesc::EntireField,
        constraint: None,
        alg: None,
        associate: vec![],
        desc: "Middle Payload".to_string(),
    };
    middle_disassembler.add_field(middle_header);
    middle_disassembler.add_field(middle_payload);

    layered.add_layer(
        "Middle Layer".to_string(),
        middle_disassembler,
        Some("middle_payload".to_string()),
    );

    // 第三层：内层协议（1字节头部）
    let mut inner_disassembler = FrameDisassembler::new();
    let inner_header = SyntaxUnit {
        field_id: "inner_header".to_string(),
        unit_type: UnitType::Uint(8),
        length: LengthDesc {
            size: 1,
            unit: LengthUnit::Byte,
        },
        scope: ScopeDesc::Global("Inner".to_string()),
        cover: CoverDesc::EntireField,
        constraint: None,
        alg: None,
        associate: vec![],
        desc: "Inner Header".to_string(),
    };
    let inner_data = SyntaxUnit {
        field_id: "inner_data".to_string(),
        unit_type: UnitType::RawData,
        length: LengthDesc {
            size: 0,
            unit: LengthUnit::Dynamic,
        },
        scope: ScopeDesc::Global("Inner".to_string()),
        cover: CoverDesc::EntireField,
        constraint: None,
        alg: None,
        associate: vec![],
        desc: "Inner Data".to_string(),
    };
    inner_disassembler.add_field(inner_header);
    inner_disassembler.add_field(inner_data);

    layered.add_layer(
        "Inner Layer".to_string(),
        inner_disassembler,
        Some("inner_data".to_string()),
    );

    // 构造三层嵌套数据
    // Outer Header (4字节) + Middle Header (2字节) + Inner Header (1字节) + Data (5字节)
    let test_data = vec![
        0xAA, 0xBB, 0xCC, 0xDD, // Outer header
        0xEE, 0xFF, // Middle header
        0x99, // Inner header
        0x01, 0x02, 0x03, 0x04, 0x05, // Application data
    ];

    // 拆包
    let result = layered.disassemble_layers(&test_data).unwrap();

    // 打印结果
    result.print();

    // 验证层数
    assert_eq!(result.layer_count(), 3);

    // 验证各层
    assert_eq!(result.get_layer(0).unwrap().layer_name, "Outer Layer");
    assert_eq!(result.get_layer(1).unwrap().layer_name, "Middle Layer");
    assert_eq!(result.get_layer(2).unwrap().layer_name, "Inner Layer");

    // 验证应用数据
    assert_eq!(result.application_data, vec![0x01, 0x02, 0x03, 0x04, 0x05]);

    println!("\n✅ 三层协议栈拆包成功！");
}

#[test]
fn test_extract_application_data_directly() {
    println!("\n=== 测试：直接提取应用数据 ===");

    let mut layered = LayeredDisassembler::new();

    // 添加TM Frame层
    let tm_disassembler = create_tm_frame_disassembler();
    layered.add_layer(
        "TM Frame".to_string(),
        tm_disassembler,
        Some("tm_data_field".to_string()),
    );

    // 添加Space Packet层
    let sp_disassembler = create_space_packet_disassembler();
    layered.add_layer(
        "Space Packet".to_string(),
        sp_disassembler,
        Some("pkt_data".to_string()),
    );

    // 简单的测试帧
    let test_frame = vec![
        0x00, 0x01, 0x00, 0x00, 0x00, 0x01, // TM Frame Header (6字节)
        0x08, 0x00, 0x00, 0x00, 0x00, 0x03, // Space Packet Header (6字节)
        0xDE, 0xAD, 0xBE, 0xEF, // Application Data (4字节)
    ];

    // 直接提取应用数据
    let app_data = layered.extract_application_data(&test_frame).unwrap();

    println!("应用数据: {:02X?}", app_data);
    assert_eq!(app_data, vec![0xDE, 0xAD, 0xBE, 0xEF]);

    println!("\n✅ 应用数据提取成功！");
}
