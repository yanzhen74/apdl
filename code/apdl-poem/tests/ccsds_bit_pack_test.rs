//! CCSDS Space Packet Bit字段打包测试
//!
//! 测试CCSDS Space Packet主头部的bit字段是否正确打包

use apdl_core::{Constraint, CoverDesc, LengthDesc, LengthUnit, ScopeDesc, SyntaxUnit, UnitType};
use apdl_poem::standard_units::frame_assembler::core::FrameAssembler;

#[test]
fn test_ccsds_space_packet_bit_packing() {
    println!("\n=== CCSDS Space Packet Bit字段打包测试 ===\n");

    // 创建CCSDS Space Packet主头部字段（前4字节）
    // 字节0-1: pkt_version(3bit) + pkt_type(1bit) + sec_hdr_flag(1bit) + apid(11bit)
    // 字节2-3: seq_flags(2bit) + pkt_seq_cnt(14bit)

    let pkt_version = SyntaxUnit {
        field_id: "pkt_version".to_string(),
        unit_type: UnitType::Bit(3),
        length: LengthDesc {
            size: 3,
            unit: LengthUnit::Bit,
        },
        scope: ScopeDesc::Global("test".to_string()),
        cover: CoverDesc::EntireField,
        constraint: Some(Constraint::FixedValue(0)), // 000
        alg: None,
        associate: vec![],
        desc: "数据包版本号".to_string(),
    };

    let pkt_type = SyntaxUnit {
        field_id: "pkt_type".to_string(),
        unit_type: UnitType::Bit(1),
        length: LengthDesc {
            size: 1,
            unit: LengthUnit::Bit,
        },
        scope: ScopeDesc::Global("test".to_string()),
        cover: CoverDesc::EntireField,
        constraint: Some(Constraint::FixedValue(0)), // 0 = 遥测包
        alg: None,
        associate: vec![],
        desc: "包类型".to_string(),
    };

    let sec_hdr_flag = SyntaxUnit {
        field_id: "sec_hdr_flag".to_string(),
        unit_type: UnitType::Bit(1),
        length: LengthDesc {
            size: 1,
            unit: LengthUnit::Bit,
        },
        scope: ScopeDesc::Global("test".to_string()),
        cover: CoverDesc::EntireField,
        constraint: Some(Constraint::FixedValue(1)), // 1 = 存在二级头
        alg: None,
        associate: vec![],
        desc: "二级头标志".to_string(),
    };

    let apid = SyntaxUnit {
        field_id: "apid".to_string(),
        unit_type: UnitType::Bit(11),
        length: LengthDesc {
            size: 11,
            unit: LengthUnit::Bit,
        },
        scope: ScopeDesc::Global("test".to_string()),
        cover: CoverDesc::EntireField,
        constraint: None,
        alg: None,
        associate: vec![],
        desc: "应用进程ID".to_string(),
    };

    let seq_flags = SyntaxUnit {
        field_id: "seq_flags".to_string(),
        unit_type: UnitType::Bit(2),
        length: LengthDesc {
            size: 2,
            unit: LengthUnit::Bit,
        },
        scope: ScopeDesc::Global("test".to_string()),
        cover: CoverDesc::EntireField,
        constraint: Some(Constraint::FixedValue(3)), // 11 = 独立包
        alg: None,
        associate: vec![],
        desc: "序列标志".to_string(),
    };

    let pkt_seq_cnt = SyntaxUnit {
        field_id: "pkt_seq_cnt".to_string(),
        unit_type: UnitType::Bit(14),
        length: LengthDesc {
            size: 14,
            unit: LengthUnit::Bit,
        },
        scope: ScopeDesc::Global("test".to_string()),
        cover: CoverDesc::EntireField,
        constraint: None,
        alg: None,
        associate: vec![],
        desc: "包序列计数".to_string(),
    };

    let mut assembler = FrameAssembler::new();
    assembler.add_field(pkt_version);
    assembler.add_field(pkt_type);
    assembler.add_field(sec_hdr_flag);
    assembler.add_field(apid);
    assembler.add_field(seq_flags);
    assembler.add_field(pkt_seq_cnt);

    // 设置字段值
    // apid = 0x0245 (581 = 01001000101)
    assembler.set_field_value("apid", &[0x02, 0x45]).unwrap();
    println!("设置 apid = 0x0245 (581)");

    // pkt_seq_cnt = 0x1234 (4660 = 01001000110100)
    assembler
        .set_field_value("pkt_seq_cnt", &[0x12, 0x34])
        .unwrap();
    println!("设置 pkt_seq_cnt = 0x1234 (4660)");

    // 组装帧
    let frame = assembler.assemble_frame().unwrap();
    println!("\n组装结果: {frame:02X?}");
    println!("帧长度: {} 字节\n", frame.len());

    // 预期结果分析：
    // 字节0-1: pkt_version(000) + pkt_type(0) + sec_hdr_flag(1) + apid(01001000101)
    //         = 0000 1010 0100 0101 = 0x0A45
    // 字节2-3: seq_flags(11) + pkt_seq_cnt(01001000110100)
    //         = 1101 0010 0011 0100 = 0xD234

    println!("预期结果:");
    println!("  字节0-1: pkt_version(000) + pkt_type(0) + sec_hdr_flag(1) + apid(01001000101)");
    println!("          = 0000_1010_0100_0101 = 0x0A45");
    println!("  字节2-3: seq_flags(11) + pkt_seq_cnt(01001000110100)");
    println!("          = 1101_0010_0011_0100 = 0xD234\n");

    assert_eq!(frame.len(), 4, "帧应该是4字节");
    assert_eq!(frame[0], 0x0A, "字节0应该是0x0A");
    assert_eq!(frame[1], 0x45, "字节1应该是0x45");
    assert_eq!(frame[2], 0xD2, "字节2应该是0xD2");
    assert_eq!(frame[3], 0x34, "字节3应该是0x34");

    println!("✓ CCSDS Space Packet Bit字段打包测试通过！");
}
