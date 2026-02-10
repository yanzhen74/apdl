//! Bit字段提取器
//!
//! 提供精确的bit级字段提取功能，支持跨字节的bit字段

use apdl_core::ProtocolError;

/// 从帧数据中提取bit字段值
///
/// 支持跨字节的bit字段提取，例如11位的APID字段
///
/// # 参数
/// - `frame_data`: 原始帧数据
/// - `bit_offset`: bit偏移量（从0开始）
/// - `bit_length`: bit长度
///
/// # 返回
/// - `Ok(u64)`: 提取的bit字段值
/// - `Err(ProtocolError)`: 提取失败（超出边界）
///
/// # 示例
/// ```
/// use apdl_lsk::frame_disassembler::extract_bit_field;
///
/// // 提取CCSDS Space Packet的APID字段（11bit，从bit5开始）
/// let frame = vec![0x0A, 0x45]; // 0000_1010_0100_0101
/// let apid = extract_bit_field(&frame, 5, 11).unwrap();
/// assert_eq!(apid, 0x245); // 01001000101
/// ```
pub fn extract_bit_field(
    frame_data: &[u8],
    bit_offset: usize,
    bit_length: usize,
) -> Result<u64, ProtocolError> {
    if bit_length == 0 || bit_length > 64 {
        return Err(ProtocolError::InvalidFrameFormat(format!(
            "Invalid bit length: {}",
            bit_length
        )));
    }

    let start_byte = bit_offset / 8;
    let start_bit = bit_offset % 8;
    let end_byte = (bit_offset + bit_length - 1) / 8;

    if end_byte >= frame_data.len() {
        return Err(ProtocolError::InvalidFrameFormat(format!(
            "Bit field exceeds frame boundary: bit_offset={}, bit_length={}, frame_size={}",
            bit_offset,
            bit_length,
            frame_data.len()
        )));
    }

    // 读取涉及的所有字节并组合成一个大整数
    let mut value = 0u64;
    for byte_idx in start_byte..=end_byte {
        value = (value << 8) | (frame_data[byte_idx] as u64);
    }

    // 计算需要右移的位数以提取目标bit范围
    let total_bits = (end_byte - start_byte + 1) * 8;
    let shift = total_bits - start_bit - bit_length;
    let mask = (1u64 << bit_length) - 1;

    Ok((value >> shift) & mask)
}

/// 从字节数组中提取指定字节范围
///
/// # 参数
/// - `frame_data`: 原始帧数据
/// - `byte_offset`: 字节偏移量
/// - `byte_length`: 字节长度
///
/// # 返回
/// - `Ok(&[u8])`: 提取的字节切片
/// - `Err(ProtocolError)`: 提取失败（超出边界）
pub fn extract_byte_range(
    frame_data: &[u8],
    byte_offset: usize,
    byte_length: usize,
) -> Result<&[u8], ProtocolError> {
    if byte_offset + byte_length > frame_data.len() {
        return Err(ProtocolError::InvalidFrameFormat(format!(
            "Byte range exceeds frame boundary: offset={}, length={}, frame_size={}",
            byte_offset,
            byte_length,
            frame_data.len()
        )));
    }

    Ok(&frame_data[byte_offset..byte_offset + byte_length])
}

/// 将bit字段值转换为字节数组
///
/// # 参数
/// - `value`: bit字段值
/// - `byte_size`: 目标字节数组大小
///
/// # 返回
/// - 字节数组（大端序）
pub fn bit_value_to_bytes(value: u64, byte_size: usize) -> Vec<u8> {
    let mut bytes = Vec::with_capacity(byte_size);
    for i in 0..byte_size {
        bytes.push(((value >> (8 * (byte_size - 1 - i))) & 0xFF) as u8);
    }
    bytes
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_bit_field_aligned() {
        // 测试字节对齐的bit字段
        let data = vec![0xFF, 0x00, 0xAB];

        // 提取第一个字节的所有8bit
        let value = extract_bit_field(&data, 0, 8).unwrap();
        assert_eq!(value, 0xFF);

        // 提取第二个字节的所有8bit
        let value = extract_bit_field(&data, 8, 8).unwrap();
        assert_eq!(value, 0x00);

        // 提取第三个字节的所有8bit
        let value = extract_bit_field(&data, 16, 8).unwrap();
        assert_eq!(value, 0xAB);
    }

    #[test]
    fn test_extract_bit_field_unaligned() {
        // 测试非字节对齐的bit字段
        // 0x0A45 = 0000_1010_0100_0101
        let data = vec![0x0A, 0x45];

        // 提取bit 0-2 (3bit): 000
        let value = extract_bit_field(&data, 0, 3).unwrap();
        assert_eq!(value, 0x00);

        // 提取bit 3 (1bit): 0
        let value = extract_bit_field(&data, 3, 1).unwrap();
        assert_eq!(value, 0x00);

        // 提取bit 4 (1bit): 1
        let value = extract_bit_field(&data, 4, 1).unwrap();
        assert_eq!(value, 0x01);

        // 提取bit 5-15 (11bit): 01001000101 = 0x245
        let value = extract_bit_field(&data, 5, 11).unwrap();
        assert_eq!(value, 0x245);
    }

    #[test]
    fn test_extract_bit_field_cross_bytes() {
        // 测试跨多个字节的bit字段
        // 0xD234 = 1101_0010_0011_0100
        let data = vec![0xD2, 0x34];

        // 提取bit 0-1 (2bit): 11
        let value = extract_bit_field(&data, 0, 2).unwrap();
        assert_eq!(value, 0x03);

        // 提取bit 2-15 (14bit): 01001000110100 = 0x1234
        let value = extract_bit_field(&data, 2, 14).unwrap();
        assert_eq!(value, 0x1234);
    }

    #[test]
    fn test_extract_bit_field_ccsds_example() {
        // CCSDS Space Packet 主头部实际例子
        // 字节0-1: 0x0A45 = version(000) + type(0) + flag(1) + apid(01001000101)
        // 字节2-3: 0xD234 = seq_flags(11) + seq_cnt(01001000110100)
        let data = vec![0x0A, 0x45, 0xD2, 0x34];

        // 提取version (3bit, offset=0)
        let version = extract_bit_field(&data, 0, 3).unwrap();
        assert_eq!(version, 0);

        // 提取type (1bit, offset=3)
        let pkt_type = extract_bit_field(&data, 3, 1).unwrap();
        assert_eq!(pkt_type, 0);

        // 提取flag (1bit, offset=4)
        let flag = extract_bit_field(&data, 4, 1).unwrap();
        assert_eq!(flag, 1);

        // 提取apid (11bit, offset=5)
        let apid = extract_bit_field(&data, 5, 11).unwrap();
        assert_eq!(apid, 0x245);

        // 提取seq_flags (2bit, offset=16)
        let seq_flags = extract_bit_field(&data, 16, 2).unwrap();
        assert_eq!(seq_flags, 0x03);

        // 提取seq_cnt (14bit, offset=18)
        let seq_cnt = extract_bit_field(&data, 18, 14).unwrap();
        assert_eq!(seq_cnt, 0x1234);
    }

    #[test]
    fn test_extract_bit_field_error_cases() {
        let data = vec![0xFF, 0x00];

        // 测试超出边界
        let result = extract_bit_field(&data, 10, 10);
        assert!(result.is_err());

        // 测试bit长度为0
        let result = extract_bit_field(&data, 0, 0);
        assert!(result.is_err());

        // 测试bit长度超过64
        let result = extract_bit_field(&data, 0, 65);
        assert!(result.is_err());
    }

    #[test]
    fn test_extract_byte_range() {
        let data = vec![0x01, 0x02, 0x03, 0x04, 0x05];

        // 提取前3字节
        let range = extract_byte_range(&data, 0, 3).unwrap();
        assert_eq!(range, &[0x01, 0x02, 0x03]);

        // 提取中间2字节
        let range = extract_byte_range(&data, 2, 2).unwrap();
        assert_eq!(range, &[0x03, 0x04]);

        // 测试超出边界
        let result = extract_byte_range(&data, 3, 5);
        assert!(result.is_err());
    }

    #[test]
    fn test_bit_value_to_bytes() {
        // 测试将bit值转换为字节数组
        let bytes = bit_value_to_bytes(0x1234, 2);
        assert_eq!(bytes, vec![0x12, 0x34]);

        let bytes = bit_value_to_bytes(0xDEADBEEF, 4);
        assert_eq!(bytes, vec![0xDE, 0xAD, 0xBE, 0xEF]);

        let bytes = bit_value_to_bytes(0x245, 2);
        assert_eq!(bytes, vec![0x02, 0x45]);
    }
}
