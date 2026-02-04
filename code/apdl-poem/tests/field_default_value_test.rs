//! 字段默认值功能测试
//!
//! 验证FrameAssembler在字段定义中有FixedValue约束时会使用默认值

use apdl_core::{Constraint, CoverDesc, LengthDesc, LengthUnit, ScopeDesc, SyntaxUnit, UnitType};
use apdl_poem::standard_units::frame_assembler::core::FrameAssembler;

#[test]
fn test_field_default_value_with_fixed_constraint() {
    // 1. 创建一个带固定值约束的字段
    let field_with_fixed_value = SyntaxUnit {
        field_id: "fixed_field".to_string(),
        unit_type: UnitType::Uint(16), // 16位无符号整数
        length: LengthDesc {
            size: 2,
            unit: LengthUnit::Byte,
        },
        scope: ScopeDesc::Global("test".to_string()),
        cover: CoverDesc::EntireField,
        constraint: Some(Constraint::FixedValue(0x1234)), // 固定值为0x1234
        alg: None,
        associate: vec![],
        desc: "Field with fixed value constraint".to_string(),
    };

    // 2. 创建另一个没有约束的字段
    let field_without_constraint = SyntaxUnit {
        field_id: "normal_field".to_string(),
        unit_type: UnitType::Uint(8), // 8位无符号整数
        length: LengthDesc {
            size: 1,
            unit: LengthUnit::Byte,
        },
        scope: ScopeDesc::Global("test".to_string()),
        cover: CoverDesc::EntireField,
        constraint: None, // 无约束
        alg: None,
        associate: vec![],
        desc: "Normal field without constraint".to_string(),
    };

    // 3. 创建FrameAssembler并添加字段
    let mut assembler = FrameAssembler::new();
    assembler.add_field(field_with_fixed_value);
    assembler.add_field(field_without_constraint);

    // 4. 不显式设置字段值，直接获取字段字节
    // 对于有FixedValue约束的字段，应该返回约束值
    let fixed_field_value = assembler.get_field_value("fixed_field").unwrap();
    assert_eq!(fixed_field_value, vec![0x12, 0x34]); // 0x1234 应该表示为 [0x12, 0x34]

    // 对于没有约束的字段，应该返回零填充的默认值
    let normal_field_value = assembler.get_field_value("normal_field").unwrap();
    assert_eq!(normal_field_value, vec![0x00]); // 零填充默认值

    println!("Fixed field default value: {:?}", fixed_field_value);
    println!("Normal field default value: {:?}", normal_field_value);

    // 5. 验证组装帧的行为
    let frame = assembler.assemble_frame().unwrap();
    // 帧应该包含固定值字段的值和普通字段的零值
    assert_eq!(frame, vec![0x12, 0x34, 0x00]);

    println!("Assembled frame with default values: {:?}", frame);
}

#[test]
fn test_field_override_fixed_value() {
    // 测试当字段有固定值约束但显式设置了不同值时，显式值应该覆盖固定值
    let field_with_fixed_value = SyntaxUnit {
        field_id: "fixed_field".to_string(),
        unit_type: UnitType::Uint(16),
        length: LengthDesc {
            size: 2,
            unit: LengthUnit::Byte,
        },
        scope: ScopeDesc::Global("test".to_string()),
        cover: CoverDesc::EntireField,
        constraint: Some(Constraint::FixedValue(0x1234)), // 固定值为0x1234
        alg: None,
        associate: vec![],
        desc: "Field with fixed value constraint".to_string(),
    };

    let mut assembler = FrameAssembler::new();
    assembler.add_field(field_with_fixed_value);

    // 显式设置一个不同于固定值的值
    assembler
        .set_field_value("fixed_field", &[0xAB, 0xCD])
        .unwrap();

    // 获取字段值，应该返回显式设置的值而不是固定值
    let field_value = assembler.get_field_value("fixed_field").unwrap();
    assert_eq!(field_value, vec![0xAB, 0xCD]); // 应该是显式设置的值

    println!("Overridden field value: {:?}", field_value);
}
