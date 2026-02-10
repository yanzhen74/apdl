//! 接收解包引擎性能基准测试

use apdl_core::*;
use apdl_lsk::{FrameDisassembler, LayeredDisassembler, ReorderBuffer};
use std::time::Instant;

/// 创建CCSDS TM Frame拆包器（用于基准测试）
fn create_tm_frame_disassembler() -> FrameDisassembler {
    let mut disassembler = FrameDisassembler::new();

    // TM帧头部字段
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

/// 创建测试帧数据
fn create_test_frame(data_size: usize) -> Vec<u8> {
    let mut frame = vec![
        0x3F, 0xF1, // version=0, scid=0x3FF, vcid=1
        0x00, 0x00, 0x00, 0x01, // frame_seq=1
    ];
    // 添加指定大小的数据
    frame.extend(vec![0xAA; data_size]);
    frame
}

#[test]
fn benchmark_frame_disassembler() {
    println!("\n=== FrameDisassembler性能基准测试 ===");

    let disassembler = create_tm_frame_disassembler();
    let iterations = 10000;

    // 测试不同大小的帧
    for data_size in [64, 256, 1024, 4096] {
        let test_frame = create_test_frame(data_size);

        let start = Instant::now();
        for _ in 0..iterations {
            let _ = disassembler.disassemble_frame(&test_frame).unwrap();
        }
        let elapsed = start.elapsed();

        let avg_time = elapsed.as_micros() as f64 / iterations as f64;
        let throughput = (data_size + 6) as f64 / avg_time; // MB/s

        println!(
            "帧大小: {:4} 字节 | 平均时间: {:6.2} μs | 吞吐量: {:6.2} MB/s",
            data_size + 6,
            avg_time,
            throughput
        );
    }
}

#[test]
fn benchmark_layered_disassembler() {
    println!("\n=== LayeredDisassembler性能基准测试 ===");

    let mut layered = LayeredDisassembler::new();
    let tm_disassembler = create_tm_frame_disassembler();
    layered.add_layer(
        "TM Frame".to_string(),
        tm_disassembler,
        Some("tm_data_field".to_string()),
    );

    let iterations = 5000;

    // 测试不同大小的帧
    for data_size in [64, 256, 1024, 4096] {
        let test_frame = create_test_frame(data_size);

        let start = Instant::now();
        for _ in 0..iterations {
            let _ = layered.disassemble_layers(&test_frame).unwrap();
        }
        let elapsed = start.elapsed();

        let avg_time = elapsed.as_micros() as f64 / iterations as f64;
        let throughput = (data_size + 6) as f64 / avg_time; // MB/s

        println!(
            "帧大小: {:4} 字节 | 平均时间: {:6.2} μs | 吞吐量: {:6.2} MB/s",
            data_size + 6,
            avg_time,
            throughput
        );
    }
}

#[test]
fn benchmark_reorder_buffer() {
    println!("\n=== ReorderBuffer性能基准测试 ===");

    let iterations = 10000;
    let pdu_size = 256;

    // 测试按序接收
    {
        let mut buffer = ReorderBuffer::new(16, 0x4000);
        let start = Instant::now();

        for i in 0..iterations {
            let pdu = vec![0xAA; pdu_size];
            let _ = buffer.insert(i as u32, pdu);
        }

        let elapsed = start.elapsed();
        let avg_time = elapsed.as_nanos() as f64 / iterations as f64;
        println!("按序接收 | 平均时间: {:6.2} ns/PDU", avg_time);
    }

    // 测试乱序接收
    {
        let mut buffer = ReorderBuffer::new(16, 0x4000);
        let start = Instant::now();

        for i in 0..iterations {
            let seq = if i % 3 == 0 {
                i + 2
            } else if i % 3 == 1 {
                i - 1
            } else {
                i
            };
            let pdu = vec![0xBB; pdu_size];
            let _ = buffer.insert(seq as u32, pdu);
        }

        let elapsed = start.elapsed();
        let avg_time = elapsed.as_nanos() as f64 / iterations as f64;
        println!("乱序接收 | 平均时间: {:6.2} ns/PDU", avg_time);
    }
}

#[test]
fn benchmark_batch_processing() {
    println!("\n=== 批量处理性能对比 ===");

    let disassembler = create_tm_frame_disassembler();
    let batch_sizes = [1, 10, 100, 1000];
    let frame_size = 256;

    for batch_size in batch_sizes {
        let test_frame = create_test_frame(frame_size);
        let iterations = 10000 / batch_size.max(1);

        let start = Instant::now();
        for _ in 0..iterations {
            for _ in 0..batch_size {
                let _ = disassembler.disassemble_frame(&test_frame).unwrap();
            }
        }
        let elapsed = start.elapsed();

        let total_frames = iterations * batch_size;
        let avg_time = elapsed.as_micros() as f64 / total_frames as f64;
        let throughput = (frame_size + 6) as f64 / avg_time; // MB/s

        println!(
            "批量大小: {:4} | 平均时间: {:6.2} μs/帧 | 吞吐量: {:6.2} MB/s",
            batch_size, avg_time, throughput
        );
    }
}
