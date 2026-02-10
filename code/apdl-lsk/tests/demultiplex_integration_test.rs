//! 解复接和异常处理集成测试

use apdl_lsk::{Demultiplexer, ReorderBuffer, SequenceValidator, ValidationResult};

#[test]
fn test_demultiplex_multiple_channels() {
    println!("\n=== 测试多通道解复接 ===\n");

    let mut demux = Demultiplexer::new(100);

    // 模拟接收来自不同VCID的帧
    println!("【接收】来自不同虚拟通道的帧");
    
    // VCID 0的帧序列
    demux
        .demultiplex(0, 0, vec![0xA0, 0x00])
        .unwrap();
    demux
        .demultiplex(0, 1, vec![0xA0, 0x01])
        .unwrap();
    demux
        .demultiplex(0, 2, vec![0xA0, 0x02])
        .unwrap();

    // VCID 1的帧序列
    demux
        .demultiplex(1, 0, vec![0xB0, 0x00])
        .unwrap();
    demux
        .demultiplex(1, 1, vec![0xB0, 0x01])
        .unwrap();

    // VCID 2的帧序列
    demux
        .demultiplex(2, 0, vec![0xC0, 0x00])
        .unwrap();

    println!("✓ 已接收来自3个虚拟通道的帧");

    // 验证通道统计
    let stats = demux.get_statistics();
    println!("\n【统计】各通道接收情况:");
    for (vcid, stat) in &stats {
        println!(
            "  VCID {}: 接收{}帧，队列长度{}",
            vcid, stat.frame_count, stat.queue_length
        );
    }

    assert_eq!(stats.get(&0).unwrap().frame_count, 3);
    assert_eq!(stats.get(&1).unwrap().frame_count, 2);
    assert_eq!(stats.get(&2).unwrap().frame_count, 1);

    // 从各通道提取PDU
    println!("\n【提取】从各通道提取PDU:");
    
    let pdu = demux.extract_pdu(0).unwrap();
    println!("  VCID 0: {pdu:02X?}");
    assert_eq!(pdu, vec![0xA0, 0x00]);

    let pdu = demux.extract_pdu(1).unwrap();
    println!("  VCID 1: {pdu:02X?}");
    assert_eq!(pdu, vec![0xB0, 0x00]);

    let pdu = demux.extract_pdu(2).unwrap();
    println!("  VCID 2: {pdu:02X?}");
    assert_eq!(pdu, vec![0xC0, 0x00]);

    println!("\n✓ 多通道解复接测试成功！\n");
}

#[test]
fn test_frame_loss_detection() {
    println!("\n=== 测试帧丢失检测 ===\n");

    let mut demux = Demultiplexer::new(100);

    println!("【接收】正常序列 0, 1, 2");
    demux.demultiplex(0, 0, vec![0x00]).unwrap();
    demux.demultiplex(0, 1, vec![0x01]).unwrap();
    demux.demultiplex(0, 2, vec![0x02]).unwrap();

    println!("✓ 序列正常");

    // 跳过序列号3，直接接收4（丢失1帧）
    println!("\n【异常】跳过序列号3，直接接收序列号4");
    let result = demux.demultiplex(0, 4, vec![0x04]).unwrap();

    match result {
        ValidationResult::FrameLost(count) => {
            println!("✓ 检测到丢失 {} 帧", count);
            assert_eq!(count, 1);
        }
        _ => panic!("应该检测到帧丢失"),
    }

    // 跳过序列号5-8，直接接收9（丢失4帧）
    println!("\n【异常】跳过序列号5-8，直接接收序列号9");
    let result = demux.demultiplex(0, 9, vec![0x09]).unwrap();

    match result {
        ValidationResult::FrameLost(count) => {
            println!("✓ 检测到丢失 {} 帧", count);
            assert_eq!(count, 4);
        }
        _ => panic!("应该检测到帧丢失"),
    }

    // 检查通道状态
    let state = demux.get_channel_state(0).unwrap();
    println!("\n【统计】VCID 0 状态:");
    println!("  接收帧数: {}", state.frame_count);
    println!("  丢失帧数: {}", state.lost_frame_count);
    println!("  丢帧率: {:.2}%", state.get_loss_rate() * 100.0);

    assert_eq!(state.frame_count, 5);
    assert_eq!(state.lost_frame_count, 5); // 丢失序列号3和5-8共5帧

    println!("\n✓ 帧丢失检测测试成功！\n");
}

#[test]
fn test_sequence_wraparound() {
    println!("\n=== 测试序列号回绕 ===\n");

    let mut validator = SequenceValidator::new(0x4000); // CCSDS 14位序列号

    // 接近序列号上限
    println!("【接收】接近序列号上限: 0x3FFE, 0x3FFF");
    validator.validate(0, 0x3FFE);
    let result = validator.validate(0, 0x3FFF);
    assert!(matches!(result, ValidationResult::Ok));

    // 序列号回绕到0
    println!("【回绕】序列号从0x3FFF回绕到0x0000");
    let result = validator.validate(0, 0);
    assert!(matches!(result, ValidationResult::Ok));
    println!("✓ 序列号正常回绕");

    // 继续正常序列
    let result = validator.validate(0, 1);
    assert!(matches!(result, ValidationResult::Ok));
    let result = validator.validate(0, 2);
    assert!(matches!(result, ValidationResult::Ok));

    println!("\n✓ 序列号回绕测试成功！\n");
}

#[test]
fn test_reorder_buffer_integration() {
    println!("\n=== 测试乱序重排缓冲区 ===\n");

    let mut buffer = ReorderBuffer::new(16, 0x4000);

    // 按序接收前3个PDU
    println!("【接收】按序: 0, 1, 2");
    buffer.insert(0, vec![0x00]);
    buffer.insert(1, vec![0x01]);
    buffer.insert(2, vec![0x02]);

    // 模拟乱序接收
    println!("\n【乱序】先收到序列号5");
    let output = buffer.insert(5, vec![0x05]);
    assert_eq!(output.len(), 0); // 缓冲等待
    println!("  → 缓冲区大小: {}", buffer.buffer_size());

    println!("再收到序列号4");
    let output = buffer.insert(4, vec![0x04]);
    assert_eq!(output.len(), 0); // 仍然等待序列号3
    println!("  → 缓冲区大小: {}", buffer.buffer_size());

    println!("最后收到序列号3");
    let output = buffer.insert(3, vec![0x03]);
    println!("  → 触发批量输出！");
    assert_eq!(output.len(), 3); // 输出3, 4, 5
    println!("  → 输出PDU: {:?}", output.len());
    
    assert_eq!(output[0], vec![0x03]);
    assert_eq!(output[1], vec![0x04]);
    assert_eq!(output[2], vec![0x05]);

    // 验证统计
    let stats = buffer.get_statistics();
    println!("\n【统计】重排缓冲区:");
    println!("  总接收: {}", stats.total_received);
    println!("  按序输出: {}", stats.in_order_output);
    println!("  重排输出: {}", stats.reordered_output);
    println!("  重排率: {:.1}%", buffer.get_reorder_rate() * 100.0);

    println!("\n✓ 乱序重排测试成功！\n");
}

#[test]
fn test_complete_demux_workflow() {
    println!("\n=== 测试完整解复接工作流 ===\n");

    let mut demux = Demultiplexer::new(100);
    let mut reorder_buffers: std::collections::HashMap<u16, ReorderBuffer> =
        std::collections::HashMap::new();

    // 模拟复杂的接收场景
    println!("【场景】模拟真实的多通道、乱序、丢帧场景");

    // VCID 0: 正常序列，无丢帧
    println!("\n1. VCID 0 - 正常序列");
    for seq in 0..5u16 {
        demux
            .demultiplex(0, seq, vec![0xA0, seq as u8])
            .unwrap();
    }

    // VCID 1: 有丢帧
    println!("2. VCID 1 - 有丢帧（跳过序列号2）");
    demux.demultiplex(1, 0, vec![0xB0, 0x00]).unwrap();
    demux.demultiplex(1, 1, vec![0xB0, 0x01]).unwrap();
    let result = demux
        .demultiplex(1, 3, vec![0xB0, 0x03])
        .unwrap(); // 跳过2
    match result {
        ValidationResult::FrameLost(count) => {
            println!("   ✓ 检测到丢失 {} 帧", count);
        }
        _ => {}
    }

    // VCID 2: 乱序接收
    println!("3. VCID 2 - 乱序接收");
    let buffer = reorder_buffers
        .entry(2)
        .or_insert_with(|| ReorderBuffer::new(16, 0x4000));

    demux.demultiplex(2, 2, vec![0xC0, 0x02]).unwrap();
    let pdu = demux.extract_pdu(2).unwrap();
    buffer.insert(2, pdu); // 先收到2

    demux.demultiplex(2, 0, vec![0xC0, 0x00]).unwrap();
    let pdu = demux.extract_pdu(2).unwrap();
    let output = buffer.insert(0, pdu); // 再收到0
    println!("   → 输出: {output:?}");

    demux.demultiplex(2, 1, vec![0xC0, 0x01]).unwrap();
    let pdu = demux.extract_pdu(2).unwrap();
    let output = buffer.insert(1, pdu); // 最后收到1，触发批量输出
    println!("   → 触发批量输出: {} PDUs", output.len());

    // 打印统计
    println!("\n【统计】各通道状态:");
    let stats = demux.get_statistics();
    for (vcid, stat) in &stats {
        println!(
            "  VCID {}: 接收{}帧, 丢失{}帧, 丢帧率{:.1}%",
            vcid,
            stat.frame_count,
            stat.lost_frame_count,
            stat.loss_rate * 100.0
        );
    }

    println!("\n✓ 完整工作流测试成功！\n");
}
