//! 自定义数据导入模块
//!
//! 支持从多种格式导入实际业务数据：二进制文件、十六进制字符串、文本数据

use std::fs;
use std::path::Path;

/// 数据导入错误
#[derive(Debug, Clone)]
pub enum ImportError {
    /// 文件读取错误
    FileError(String),
    /// 格式解析错误
    ParseError(String),
    /// 数据长度错误
    LengthError(String),
}

impl std::fmt::Display for ImportError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ImportError::FileError(msg) => write!(f, "文件错误: {}", msg),
            ImportError::ParseError(msg) => write!(f, "解析错误: {}", msg),
            ImportError::LengthError(msg) => write!(f, "长度错误: {}", msg),
        }
    }
}

impl std::error::Error for ImportError {}

/// 数据导入器
pub struct DataImporter;

impl DataImporter {
    /// 从二进制文件导入数据
    ///
    /// # 参数
    /// - `path`: 文件路径
    ///
    /// # 返回
    /// - `Ok(Vec<u8>)`: 导入的字节数据
    /// - `Err(ImportError)`: 导入失败
    ///
    /// # 示例
    /// ```
    /// use apdl_lsk::data_generator::DataImporter;
    ///
    /// // let data = DataImporter::import_from_file("data.bin").unwrap();
    /// ```
    pub fn import_from_file<P: AsRef<Path>>(path: P) -> Result<Vec<u8>, ImportError> {
        fs::read(&path).map_err(|e| {
            ImportError::FileError(format!("无法读取文件 '{}': {}", path.as_ref().display(), e))
        })
    }

    /// 从十六进制字符串导入数据
    ///
    /// # 参数
    /// - `hex_str`: 十六进制字符串（支持空格分隔、0x前缀、无分隔符）
    ///
    /// # 返回
    /// - `Ok(Vec<u8>)`: 导入的字节数据
    /// - `Err(ImportError)`: 解析失败
    ///
    /// # 支持的格式
    /// - "DEADBEEF"（无分隔符）
    /// - "DE AD BE EF"（空格分隔）
    /// - "0xDE 0xAD 0xBE 0xEF"（0x前缀）
    /// - "DE:AD:BE:EF"（冒号分隔）
    ///
    /// # 示例
    /// ```
    /// use apdl_lsk::data_generator::DataImporter;
    ///
    /// let data = DataImporter::import_from_hex("DEADBEEF").unwrap();
    /// assert_eq!(data, vec![0xDE, 0xAD, 0xBE, 0xEF]);
    /// ```
    pub fn import_from_hex(hex_str: &str) -> Result<Vec<u8>, ImportError> {
        // 移除所有0x前缀（支持"0xDE 0xAD"格式）
        let without_prefix = hex_str.replace("0x", "").replace("0X", "");
        
        // 移除所有非十六进制字符（保留十六进制数字）
        let cleaned: String = without_prefix
            .chars()
            .filter(|c| c.is_ascii_hexdigit())
            .collect();

        if cleaned.is_empty() {
            return Err(ImportError::ParseError(
                "没有有效的十六进制字符".to_string(),
            ));
        }

        if cleaned.len() % 2 != 0 {
            return Err(ImportError::ParseError(
                "十六进制字符串长度必须是偶数".to_string(),
            ));
        }

        let mut result = Vec::with_capacity(cleaned.len() / 2);

        for i in (0..cleaned.len()).step_by(2) {
            let byte_str = &cleaned[i..i + 2];
            match u8::from_str_radix(byte_str, 16) {
                Ok(byte) => result.push(byte),
                Err(e) => {
                    return Err(ImportError::ParseError(format!(
                        "无法解析字节 '{}': {}",
                        byte_str, e
                    )))
                }
            }
        }

        Ok(result)
    }

    /// 从文本数据导入（UTF-8编码）
    ///
    /// # 参数
    /// - `text`: 文本字符串
    ///
    /// # 返回
    /// 文本的UTF-8字节表示
    ///
    /// # 示例
    /// ```
    /// use apdl_lsk::data_generator::DataImporter;
    ///
    /// let data = DataImporter::import_from_text("Hello");
    /// assert_eq!(data, vec![0x48, 0x65, 0x6C, 0x6C, 0x6F]);
    /// ```
    pub fn import_from_text(text: &str) -> Vec<u8> {
        text.as_bytes().to_vec()
    }

    /// 从Base64字符串导入数据
    ///
    /// # 参数
    /// - `base64_str`: Base64编码字符串
    ///
    /// # 返回
    /// - `Ok(Vec<u8>)`: 解码后的字节数据
    /// - `Err(ImportError)`: 解码失败
    pub fn import_from_base64(base64_str: &str) -> Result<Vec<u8>, ImportError> {
        use base64::{engine::general_purpose::STANDARD, Engine};
        
        STANDARD.decode(base64_str).map_err(|e| {
            ImportError::ParseError(format!("Base64解码失败: {}", e))
        })
    }

    /// 调整数据长度（截断或填充）
    ///
    /// # 参数
    /// - `data`: 原始数据
    /// - `target_length`: 目标长度
    /// - `fill_byte`: 填充字节（默认0）
    ///
    /// # 返回
    /// 调整长度后的数据
    ///
    /// # 示例
    /// ```
    /// use apdl_lsk::data_generator::DataImporter;
    ///
    /// // 截断
    /// let data = vec![0x01, 0x02, 0x03, 0x04];
    /// let adjusted = DataImporter::adjust_length(&data, 2, 0);
    /// assert_eq!(adjusted, vec![0x01, 0x02]);
    ///
    /// // 填充
    /// let data = vec![0x01, 0x02];
    /// let adjusted = DataImporter::adjust_length(&data, 4, 0xFF);
    /// assert_eq!(adjusted, vec![0x01, 0x02, 0xFF, 0xFF]);
    /// ```
    pub fn adjust_length(data: &[u8], target_length: usize, fill_byte: u8) -> Vec<u8> {
        if data.len() == target_length {
            data.to_vec()
        } else if data.len() > target_length {
            // 截断
            data[..target_length].to_vec()
        } else {
            // 填充
            let mut result = data.to_vec();
            result.resize(target_length, fill_byte);
            result
        }
    }

    /// 合并多个数据段
    ///
    /// # 参数
    /// - `segments`: 数据段列表
    ///
    /// # 返回
    /// 合并后的数据
    pub fn merge_segments(segments: &[Vec<u8>]) -> Vec<u8> {
        let total_len: usize = segments.iter().map(|s| s.len()).sum();
        let mut result = Vec::with_capacity(total_len);
        
        for segment in segments {
            result.extend_from_slice(segment);
        }
        
        result
    }

    /// 分割数据为固定大小的块
    ///
    /// # 参数
    /// - `data`: 原始数据
    /// - `chunk_size`: 每个块的大小
    ///
    /// # 返回
    /// 数据块列表
    pub fn split_into_chunks(data: &[u8], chunk_size: usize) -> Vec<Vec<u8>> {
        if chunk_size == 0 {
            return vec![];
        }

        data.chunks(chunk_size)
            .map(|chunk| chunk.to_vec())
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_import_from_hex_no_separator() {
        let data = DataImporter::import_from_hex("DEADBEEF").unwrap();
        assert_eq!(data, vec![0xDE, 0xAD, 0xBE, 0xEF]);
    }

    #[test]
    fn test_import_from_hex_with_spaces() {
        let data = DataImporter::import_from_hex("DE AD BE EF").unwrap();
        assert_eq!(data, vec![0xDE, 0xAD, 0xBE, 0xEF]);
    }

    #[test]
    fn test_import_from_hex_with_prefix() {
        let data = DataImporter::import_from_hex("0xDE 0xAD 0xBE 0xEF").unwrap();
        assert_eq!(data, vec![0xDE, 0xAD, 0xBE, 0xEF]);
    }

    #[test]
    fn test_import_from_hex_with_colons() {
        let data = DataImporter::import_from_hex("DE:AD:BE:EF").unwrap();
        assert_eq!(data, vec![0xDE, 0xAD, 0xBE, 0xEF]);
    }

    #[test]
    fn test_import_from_hex_odd_length() {
        let result = DataImporter::import_from_hex("DEADB");
        assert!(result.is_err());
    }

    #[test]
    fn test_import_from_hex_with_invalid_char_filtered() {
        // 无效字符被过滤掉，只解析有效的十六进制部分
        let result = DataImporter::import_from_hex("DEADGG");
        // GG不是有效十六进制，被过滤后剩下DEAD（4个字符）
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), vec![0xDE, 0xAD]);
    }

    #[test]
    fn test_import_from_hex_all_invalid() {
        // 全部是无效字符
        let result = DataImporter::import_from_hex("GGHHZZ");
        // 过滤后为空，应该报错
        assert!(result.is_err());
    }

    #[test]
    fn test_import_from_text() {
        let data = DataImporter::import_from_text("Hello");
        assert_eq!(data, vec![0x48, 0x65, 0x6C, 0x6C, 0x6F]);
    }

    #[test]
    fn test_import_from_text_chinese() {
        let data = DataImporter::import_from_text("中");
        // UTF-8编码的中文字符
        assert_eq!(data, vec![0xE4, 0xB8, 0xAD]);
    }

    #[test]
    fn test_adjust_length_truncate() {
        let data = vec![0x01, 0x02, 0x03, 0x04];
        let adjusted = DataImporter::adjust_length(&data, 2, 0);
        assert_eq!(adjusted, vec![0x01, 0x02]);
    }

    #[test]
    fn test_adjust_length_pad() {
        let data = vec![0x01, 0x02];
        let adjusted = DataImporter::adjust_length(&data, 4, 0xFF);
        assert_eq!(adjusted, vec![0x01, 0x02, 0xFF, 0xFF]);
    }

    #[test]
    fn test_adjust_length_exact() {
        let data = vec![0x01, 0x02, 0x03];
        let adjusted = DataImporter::adjust_length(&data, 3, 0);
        assert_eq!(adjusted, vec![0x01, 0x02, 0x03]);
    }

    #[test]
    fn test_merge_segments() {
        let segments = vec![
            vec![0x01, 0x02],
            vec![0x03, 0x04],
            vec![0x05, 0x06],
        ];
        let merged = DataImporter::merge_segments(&segments);
        assert_eq!(merged, vec![0x01, 0x02, 0x03, 0x04, 0x05, 0x06]);
    }

    #[test]
    fn test_split_into_chunks() {
        let data = vec![0x01, 0x02, 0x03, 0x04, 0x05, 0x06];
        let chunks = DataImporter::split_into_chunks(&data, 2);
        assert_eq!(chunks, vec![
            vec![0x01, 0x02],
            vec![0x03, 0x04],
            vec![0x05, 0x06],
        ]);
    }

    #[test]
    fn test_split_into_chunks_uneven() {
        let data = vec![0x01, 0x02, 0x03, 0x04, 0x05];
        let chunks = DataImporter::split_into_chunks(&data, 2);
        assert_eq!(chunks, vec![
            vec![0x01, 0x02],
            vec![0x03, 0x04],
            vec![0x05],
        ]);
    }

    #[test]
    fn test_import_from_base64() {
        let data = DataImporter::import_from_base64("SGVsbG8gV29ybGQh").unwrap();
        assert_eq!(data, vec![0x48, 0x65, 0x6C, 0x6C, 0x6F, 0x20, 0x57, 0x6F, 0x72, 0x6C, 0x64, 0x21]);
    }
}
