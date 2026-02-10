//! 端到端集成测试：发送端组装 → 接收端拆包
//!
//! 验证CCSDS协议的完整收发流程

use apdl_core::*;
use apdl_lsk::{extract_bit_field, FrameDisassembler, FrameSynchronizer, ReceiveBuffer, SyncMode};
use apdl_poem::standard_units::frame_assembler::core::FrameAssembler;

#[test]
fn test_end_to_end_ccsds_space_packet() {
    println!("\n=== 测试CCSDS Space Packet端到端收发 ===\n");

    // ============================================
    // 1. 发送端：组装CCSDS Space Packet
    // ============================================
    println!("【发送端】组装CCSDS Space Packet");

    let mut tx_assembler = FrameAssembler::new();

    // CCSDS Space Packet主头部字段定义
    // 字节0-1: version(3bit) + type(1bit) + sec_hdr_flag(1bit) + apid(11bit)
    // 字节2-3: seq_flags(2bit) + pkt_seq_cnt(14bit)
    // 字节4-5: pkt_len(16bit)

    let version_field = SyntaxUnit {
        field_id: "pkt_version".to_string(),
        unit_type: UnitType::Bit(3),
        length: LengthDesc {
            size: 3,
            unit: LengthUnit::Bit,
        },
        scope: ScopeDesc::Global("space_packet".to_string()),
        cover: CoverDesc::EntireField,
        constraint: Some(Constraint::FixedValue(0)),
        alg: None,
        associate: vec![],
        desc: "Packet Version".to_string(),
    };

    let type_field = SyntaxUnit {
        field_id: "pkt_type".to_string(),
        unit_type: UnitType::Bit(1),
        length: LengthDesc {
            size: 1,
            unit: LengthUnit::Bit,
        },
        scope: ScopeDesc::Global("space_packet".to_string()),
        cover: CoverDesc::EntireField,
        constraint: None,
        alg: None,
        associate: vec![],
        desc: "Packet Type".to_string(),
    };

    let sec_hdr_flag_field = SyntaxUnit {
        field_id: "sec_hdr_flag".to_string(),
        unit_type: UnitType::Bit(1),
        length: LengthDesc {
            size: 1,
            unit: LengthUnit::Bit,
        },
        scope: ScopeDesc::Global("space_packet".to_string()),
        cover: CoverDesc::EntireField,
        constraint: None,
        alg: None,
        associate: vec![],
        desc: "Secondary Header Flag".to_string(),
    };

    let apid_field = SyntaxUnit {
        field_id: "apid".to_string(),
        unit_type: UnitType::Bit(11),
        length: LengthDesc {
            size: 11,
            unit: LengthUnit::Bit,
        },
        scope: ScopeDesc::Global("space_packet".to_string()),
        cover: CoverDesc::EntireField,
        constraint: None,
        alg: None,
        associate: vec![],
        desc: "Application Process ID".to_string(),
    };

    let seq_flags_field = SyntaxUnit {
        field_id: "seq_flags".to_string(),
        unit_type: UnitType::Bit(2),
        length: LengthDesc {
            size: 2,
            unit: LengthUnit::Bit,
        },
        scope: ScopeDesc::Global("space_packet".to_string()),
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
        scope: ScopeDesc::Global("space_packet".to_string()),
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
        scope: ScopeDesc::Global("space_packet".to_string()),
        cover: CoverDesc::EntireField,
        constraint: None,
        alg: None,
        associate: vec![],
        desc: "Packet Data Length".to_string(),
    };

    let data_field = SyntaxUnit {
        field_id: "pkt_data".to_string(),
        unit_type: UnitType::RawData,
        length: LengthDesc {
            size: 10,
            unit: LengthUnit::Byte,
        },
        scope: ScopeDesc::Global("space_packet".to_string()),
        cover: CoverDesc::EntireField,
        constraint: None,
        alg: None,
        associate: vec![],
        desc: "Packet Data".to_string(),
    };

    // 添加所有字段
    tx_assembler.add_field(version_field.clone());
    tx_assembler.add_field(type_field.clone());
    tx_assembler.add_field(sec_hdr_flag_field.clone());
    tx_assembler.add_field(apid_field.clone());
    tx_assembler.add_field(seq_flags_field.clone());
    tx_assembler.add_field(pkt_seq_cnt_field.clone());
    tx_assembler.add_field(pkt_len_field.clone());
    tx_assembler.add_field(data_field.clone());

    // 设置字段值
    tx_assembler
        .set_field_value("pkt_version", &[0x00])
        .unwrap();
    tx_assembler.set_field_value("pkt_type", &[0x00]).unwrap();
    tx_assembler
        .set_field_value("sec_hdr_flag", &[0x01])
        .unwrap();
    tx_assembler.set_field_value("apid", &[0x02, 0x45]).unwrap(); // 0x245
    tx_assembler.set_field_value("seq_flags", &[0x03]).unwrap(); // 二进制11
    tx_assembler
        .set_field_value("pkt_seq_cnt", &[0x12, 0x34])
        .unwrap(); // 0x1234
    tx_assembler
        .set_field_value("pkt_len", &[0x00, 0x09])
        .unwrap(); // 数据长度-1
    tx_assembler
        .set_field_value(
            "pkt_data",
            &[0xDE, 0xAD, 0xBE, 0xEF, 0xCA, 0xFE, 0xBA, 0xBE, 0x12, 0x34],
        )
        .unwrap();

    // 组装帧
    let tx_frame = tx_assembler.assemble_frame().unwrap();

    println!("发送帧数据 ({} 字节):", tx_frame.len());
    for (i, chunk) in tx_frame.chunks(16).enumerate() {
        print!("  {:02X}: ", i * 16);
        for b in chunk {
            print!("{b:02X} ");
        }
        println!();
    }

    // 验证发送端组装的帧格式
    assert_eq!(tx_frame[0], 0x0A); // version(000) + type(0) + flag(1) + apid高3位(010)
    assert_eq!(tx_frame[1], 0x45); // apid低8位
    assert_eq!(tx_frame[2], 0xD2); // seq_flags(11) + seq_cnt高6位
    assert_eq!(tx_frame[3], 0x34); // seq_cnt低8位

    // ============================================
    // 2. 接收端：接收和帧边界识别
    // ============================================
    println!("\n【接收端】接收数据流");

    // 模拟接收数据（可能包含噪声、分段接收等）
    let mut rx_buffer = ReceiveBuffer::new(1024);

    // 模拟分段接收：先接收一些噪声数据
    rx_buffer.append(&[0xFF, 0xFF, 0x00, 0x00]);
    println!("接收噪声数据: FF FF 00 00");

    // 然后接收实际的帧数据
    rx_buffer.append(&tx_frame);
    println!("接收完整帧: {} 字节", tx_frame.len());

    println!("当前缓冲区总长度: {} 字节", rx_buffer.len());

    // 对于CCSDS Space Packet，我们可以直接从缓冲区提取（假设已经识别出起始位置）
    // 实际场景中可能需要同步字搜索
    rx_buffer.discard(4); // 丢弃噪声数据

    let received_frame = rx_buffer.extract_frame(tx_frame.len()).unwrap();

    println!("\n提取的帧数据:");
    for (i, chunk) in received_frame.chunks(16).enumerate() {
        print!("  {:02X}: ", i * 16);
        for b in chunk {
            print!("{b:02X} ");
        }
        println!();
    }

    // ============================================
    // 3. 接收端：拆包和字段提取
    // ============================================
    println!("\n【接收端】拆包并提取字段");

    let mut rx_disassembler = FrameDisassembler::new();
    rx_disassembler.add_field(version_field);
    rx_disassembler.add_field(type_field);
    rx_disassembler.add_field(sec_hdr_flag_field);
    rx_disassembler.add_field(apid_field);
    rx_disassembler.add_field(seq_flags_field);
    rx_disassembler.add_field(pkt_seq_cnt_field);
    rx_disassembler.add_field(pkt_len_field);
    rx_disassembler.add_field(data_field);

    let fields = rx_disassembler.disassemble_frame(&received_frame).unwrap();

    // 打印提取的字段
    println!("\n提取的字段值:");
    for field_name in rx_disassembler.get_field_names() {
        if let Some(value) = fields.get(field_name) {
            print!("  {}: ", field_name);
            for b in value {
                print!("{b:02X} ");
            }
            println!();
        }
    }

    // ============================================
    // 4. 验证字段值
    // ============================================
    println!("\n【验证】检查字段值是否正确");

    // 验证version
    let version = extract_bit_field(&received_frame, 0, 3).unwrap();
    assert_eq!(version, 0);
    println!("✓ pkt_version = 0");

    // 验证type
    let pkt_type = extract_bit_field(&received_frame, 3, 1).unwrap();
    assert_eq!(pkt_type, 0);
    println!("✓ pkt_type = 0");

    // 验证sec_hdr_flag
    let sec_hdr_flag = extract_bit_field(&received_frame, 4, 1).unwrap();
    assert_eq!(sec_hdr_flag, 1);
    println!("✓ sec_hdr_flag = 1");

    // 验证apid
    let apid = extract_bit_field(&received_frame, 5, 11).unwrap();
    assert_eq!(apid, 0x245);
    println!("✓ apid = 0x{apid:03X}");

    // 验证seq_flags
    let seq_flags = extract_bit_field(&received_frame, 16, 2).unwrap();
    assert_eq!(seq_flags, 0x03);
    println!("✓ seq_flags = 0x{seq_flags:X}");

    // 验证pkt_seq_cnt
    let pkt_seq_cnt = extract_bit_field(&received_frame, 18, 14).unwrap();
    assert_eq!(pkt_seq_cnt, 0x1234);
    println!("✓ pkt_seq_cnt = 0x{pkt_seq_cnt:04X}");

    // 验证应用数据
    let app_data = fields.get("pkt_data").unwrap();
    assert_eq!(
        app_data,
        &vec![0xDE, 0xAD, 0xBE, 0xEF, 0xCA, 0xFE, 0xBA, 0xBE, 0x12, 0x34]
    );
    println!("✓ 应用数据完整性验证通过");

    println!("\n=== 端到端测试成功！ ===\n");
}

#[test]
fn test_end_to_end_with_sync_marker() {
    println!("\n=== 测试带同步字的帧收发 ===\n");

    // ============================================
    // 1. 发送端：组装带同步字的帧
    // ============================================
    let mut tx_assembler = FrameAssembler::new();

    // 添加同步字字段
    let sync_field = SyntaxUnit {
        field_id: "sync_marker".to_string(),
        unit_type: UnitType::Uint(16),
        length: LengthDesc {
            size: 2,
            unit: LengthUnit::Byte,
        },
        scope: ScopeDesc::Global("test_frame".to_string()),
        cover: CoverDesc::EntireField,
        constraint: Some(Constraint::FixedValue(0xEB90)),
        alg: None,
        associate: vec![],
        desc: "Sync Marker".to_string(),
    };

    let frame_id_field = SyntaxUnit {
        field_id: "frame_id".to_string(),
        unit_type: UnitType::Uint(8),
        length: LengthDesc {
            size: 1,
            unit: LengthUnit::Byte,
        },
        scope: ScopeDesc::Global("test_frame".to_string()),
        cover: CoverDesc::EntireField,
        constraint: None,
        alg: None,
        associate: vec![],
        desc: "Frame ID".to_string(),
    };

    let data_field = SyntaxUnit {
        field_id: "data".to_string(),
        unit_type: UnitType::RawData,
        length: LengthDesc {
            size: 8,
            unit: LengthUnit::Byte,
        },
        scope: ScopeDesc::Global("test_frame".to_string()),
        cover: CoverDesc::EntireField,
        constraint: None,
        alg: None,
        associate: vec![],
        desc: "Data".to_string(),
    };

    tx_assembler.add_field(sync_field.clone());
    tx_assembler.add_field(frame_id_field.clone());
    tx_assembler.add_field(data_field.clone());

    tx_assembler
        .set_field_value("sync_marker", &[0xEB, 0x90])
        .unwrap();
    tx_assembler.set_field_value("frame_id", &[0x42]).unwrap();
    tx_assembler
        .set_field_value("data", &[0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08])
        .unwrap();

    let tx_frame = tx_assembler.assemble_frame().unwrap();
    println!("发送帧: {tx_frame:02X?}");

    // ============================================
    // 2. 接收端：使用同步字搜索帧边界
    // ============================================
    let mut rx_buffer = ReceiveBuffer::new(1024);
    let synchronizer = FrameSynchronizer::new(SyncMode::FixedMarker(vec![0xEB, 0x90]));
    rx_buffer.set_synchronizer(synchronizer);

    // 模拟接收：噪声 + 完整帧
    rx_buffer.append(&[0xFF, 0xFF, 0x00]); // 噪声
    rx_buffer.append(&tx_frame);

    println!("缓冲区长度: {} 字节", rx_buffer.len());

    // 搜索同步字
    let sync_pos = rx_buffer.find_sync_marker().unwrap();
    println!("找到同步字在偏移: {sync_pos}");
    assert_eq!(sync_pos, 3);

    // 丢弃噪声数据
    rx_buffer.discard(sync_pos);

    // 提取帧
    let received_frame = rx_buffer.extract_frame(tx_frame.len()).unwrap();
    println!("接收帧: {received_frame:02X?}");

    // ============================================
    // 3. 拆包验证
    // ============================================
    let mut rx_disassembler = FrameDisassembler::new();
    rx_disassembler.add_field(sync_field);
    rx_disassembler.add_field(frame_id_field);
    rx_disassembler.add_field(data_field);

    let fields = rx_disassembler.disassemble_frame(&received_frame).unwrap();

    // 验证字段值
    assert_eq!(fields.get("sync_marker").unwrap(), &vec![0xEB, 0x90]);
    assert_eq!(fields.get("frame_id").unwrap(), &vec![0x42]);
    assert_eq!(
        fields.get("data").unwrap(),
        &vec![0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08]
    );

    println!("\n✓ 带同步字的帧收发测试成功！\n");
}
