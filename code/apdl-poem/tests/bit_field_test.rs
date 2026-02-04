//! Bit字段功能测试
//!
//! 验证FrameAssembler对bit级别字段的支持

use apdl_core::{Constraint, CoverDesc, LengthDesc, LengthUnit, ScopeDesc, SyntaxUnit, UnitType};
use apdl_poem::standard_units::frame_assembler::core::FrameAssembler;

#[test]
fn test_bit_field_handling() {
    // 1. 创建几个bit字段
    let bit_field_1 = SyntaxUnit {
        field_id: "flag1".to_string(),
        unit_type: UnitType::Bit(1), // 1 bit
        length: LengthDesc {
            size: 1, // 1 bit
            unit: LengthUnit::Bit,
        },
        scope: ScopeDesc::Global("test".to_string()),
        cover: CoverDesc::EntireField,
        constraint: Some(Constraint::FixedValue(1)), // 固定值为1
        alg: None,
        associate: vec![],
        desc: "1-bit flag field".to_string(),
    };

    let bit_field_2 = SyntaxUnit {
        field_id: "flag2".to_string(),
        unit_type: UnitType::Bit(1), // 1 bit
        length: LengthDesc {
            size: 1, // 1 bit
            unit: LengthUnit::Bit,
        },
        scope: ScopeDesc::Global("test".to_string()),
        cover: CoverDesc::EntireField,
        constraint: Some(Constraint::FixedValue(0)), // 固定值为0
        alg: None,
        associate: vec![],
        desc: "1-bit flag field".to_string(),
    };

    let bit_field_3 = SyntaxUnit {
        field_id: "multi_bit".to_string(),
        unit_type: UnitType::Bit(3), // 3 bits
        length: LengthDesc {
            size: 3, // 3 bits
            unit: LengthUnit::Bit,
        },
        scope: ScopeDesc::Global("test".to_string()),
        cover: CoverDesc::EntireField,
        constraint: Some(Constraint::FixedValue(5)), // 固定值为5 (二进制101)
        alg: None,
        associate: vec![],
        desc: "3-bit field".to_string(),
    };

    // 2. 创建FrameAssembler并添加字段
    let mut assembler = FrameAssembler::new();
    assembler.add_field(bit_field_1);
    assembler.add_field(bit_field_2);
    assembler.add_field(bit_field_3);

    // 3. 验证字段大小
    let flag1_size = assembler.get_field_size_by_name("flag1").unwrap();
    let flag2_size = assembler.get_field_size_by_name("flag2").unwrap();
    let multi_bit_size = assembler.get_field_size_by_name("multi_bit").unwrap();

    // 当前实现会将bit向上取整到字节，所以所有字段都会是1字节
    println!("Flag1 size: {} bytes", flag1_size);
    println!("Flag2 size: {} bytes", flag2_size);
    println!("Multi-bit size: {} bytes", multi_bit_size);

    // 4. 获取字段值（应该从固定值约束中获取）
    let flag1_value = assembler.get_field_value("flag1").unwrap();
    let flag2_value = assembler.get_field_value("flag2").unwrap();
    let multi_bit_value = assembler.get_field_value("multi_bit").unwrap();

    println!("Flag1 value: {:?}", flag1_value);
    println!("Flag2 value: {:?}", flag2_value);
    println!("Multi-bit value: {:?}", multi_bit_value);

    // 5. 组装帧
    let frame = assembler.assemble_frame().unwrap();
    println!("Assembled frame: {:?}", frame);
    println!("Frame length: {} bytes", frame.len());

    // 6. 由于当前实现将bit字段当作字节处理，结果可能不是最优的
    // 我们需要一个更好的bit字段处理机制
}

#[test]
fn test_bit_field_with_explicit_values() {
    // 测试显式设置bit字段值
    let bit_field = SyntaxUnit {
        field_id: "control_bits".to_string(),
        unit_type: UnitType::Bit(4), // 4 bits
        length: LengthDesc {
            size: 4, // 4 bits
            unit: LengthUnit::Bit,
        },
        scope: ScopeDesc::Global("test".to_string()),
        cover: CoverDesc::EntireField,
        constraint: None, // 无固定值
        alg: None,
        associate: vec![],
        desc: "4-bit control field".to_string(),
    };

    let mut assembler = FrameAssembler::new();
    assembler.add_field(bit_field);

    // 设置一个4-bit值 (比如二进制1011，即十进制11)
    assembler.set_field_value("control_bits", &[0x0B]).unwrap(); // 0x0B = 1011 in binary

    let value = assembler.get_field_value("control_bits").unwrap();
    assert_eq!(value, vec![0x0B]);

    println!("Control bits value: {:?}", value);

    let frame = assembler.assemble_frame().unwrap();
    println!("Frame with control bits: {:?}", frame);
}

#[test]
fn test_mixed_bit_and_byte_fields() {
    // 测试bit字段和byte字段的混合，按顺序添加bit field1、byte field、bit field2、bit field3
    // 期望的打包方式：byte field (8 bits = 255) -> 0xFF
    // 然后bit_field_1 (1 bit = 1) + bit_field_2 (2 bits = 2 = 10) + bit_field_3 (5 bits = 15 = 01111) -> 11001111 = 0xCF
    // 总共2字节: 0xFFCF

    let bit_field_1 = SyntaxUnit {
        field_id: "flag1".to_string(),
        unit_type: UnitType::Bit(1), // 1 bit
        length: LengthDesc {
            size: 1, // 1 bit
            unit: LengthUnit::Bit,
        },
        scope: ScopeDesc::Global("test".to_string()),
        cover: CoverDesc::EntireField,
        constraint: Some(Constraint::FixedValue(1)), // 二进制1
        alg: None,
        associate: vec![],
        desc: "1-bit flag 1".to_string(),
    };

    let byte_field = SyntaxUnit {
        field_id: "data".to_string(),
        unit_type: UnitType::Uint(8), // 8 bits = 1 byte
        length: LengthDesc {
            size: 1, // 1 byte
            unit: LengthUnit::Byte,
        },
        scope: ScopeDesc::Global("test".to_string()),
        cover: CoverDesc::EntireField,
        constraint: Some(Constraint::FixedValue(255)), // 固定值255
        alg: None,
        associate: vec![],
        desc: "1-byte data".to_string(),
    };

    let bit_field_2 = SyntaxUnit {
        field_id: "flag2".to_string(),
        unit_type: UnitType::Bit(2), // 2 bits
        length: LengthDesc {
            size: 2, // 2 bits
            unit: LengthUnit::Bit,
        },
        scope: ScopeDesc::Global("test".to_string()),
        cover: CoverDesc::EntireField,
        constraint: Some(Constraint::FixedValue(2)), // 二进制10
        alg: None,
        associate: vec![],
        desc: "2-bit flag 2".to_string(),
    };

    let bit_field_3 = SyntaxUnit {
        field_id: "flag3".to_string(),
        unit_type: UnitType::Bit(5), // 5 bits
        length: LengthDesc {
            size: 5, // 5 bits
            unit: LengthUnit::Bit,
        },
        scope: ScopeDesc::Global("test".to_string()),
        cover: CoverDesc::EntireField,
        constraint: Some(Constraint::FixedValue(15)), // 二进制1111
        alg: None,
        associate: vec![],
        desc: "5-bit flag 3".to_string(),
    };

    let mut assembler = FrameAssembler::new();
    // 按照要求的顺序添加字段: bit field1, byte field, bit field2, bit field3
    assembler.add_field(bit_field_1);
    assembler.add_field(byte_field);
    assembler.add_field(bit_field_2);
    assembler.add_field(bit_field_3);

    let frame = assembler.assemble_frame().unwrap();
    println!("Mixed field frame: {:?}", frame);
    println!("Mixed field frame length: {} bytes", frame.len());

    // 验证frame长度为2字节 (byte field 1字节 + 所有bit字段打包成1字节)
    assert_eq!(
        frame.len(),
        2,
        "Frame should be 2 bytes long (byte field 1 byte + all bit fields packed into 1 byte)"
    );

    // 验证期望的2字节结果: 0xFFCF
    // frame[0] = byte field = 255 = 0xFF
    // frame[1] = bit_field_1(1) + bit_field_2(2) + bit_field_3(15) = 1 + 2 bits(10) + 5 bits(01111) = 11001111 = 0xCF
    assert_eq!(
        frame[0], 0xFF,
        "First byte should be the byte field value (255 = 0xFF)"
    );
    assert_eq!(frame[1], 0xCF, "Second byte should contain all bit fields packed: bit1(1) + bit2(10) + bit3(01111) = 11001111 = 0xCF");

    // 测试通过，证明frame assembler已按要求实现bit字段的紧凑打包
    println!("Expected: [0xFF, 0xCF] = [255, 207], Actual: {:?}", frame);
}
