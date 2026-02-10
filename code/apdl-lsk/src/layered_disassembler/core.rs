//! 分层拆包引擎核心实现

use apdl_core::ProtocolError;

use crate::frame_disassembler::FrameDisassembler;
use super::layer_data::{DisassembleResult, LayerData};

/// 分层拆包引擎
///
/// 自动识别协议层级关系，递归拆包直到应用数据层
pub struct LayeredDisassembler {
    /// 各层的拆包器（从外到内）
    layer_disassemblers: Vec<LayerDisassemblerInfo>,
}

/// 单层拆包器信息
struct LayerDisassemblerInfo {
    /// 层名称
    layer_name: String,
    /// 帧拆包器
    disassembler: FrameDisassembler,
    /// 净荷字段名（指向下一层的字段）
    payload_field_name: Option<String>,
}

impl LayeredDisassembler {
    /// 创建新的分层拆包引擎
    pub fn new() -> Self {
        Self {
            layer_disassemblers: Vec::new(),
        }
    }

    /// 添加一层拆包器
    ///
    /// # 参数
    /// - `layer_name`: 层名称
    /// - `disassembler`: 该层的帧拆包器
    /// - `payload_field_name`: 净荷字段名（如果有下一层）
    pub fn add_layer(
        &mut self,
        layer_name: String,
        disassembler: FrameDisassembler,
        payload_field_name: Option<String>,
    ) {
        self.layer_disassemblers.push(LayerDisassemblerInfo {
            layer_name,
            disassembler,
            payload_field_name,
        });
    }

    /// 完整的分层拆包流程
    ///
    /// # 参数
    /// - `raw_data`: 原始数据（最外层）
    ///
    /// # 返回
    /// - `Ok(DisassembleResult)`: 拆包结果
    /// - `Err(ProtocolError)`: 拆包失败
    ///
    /// # 示例
    /// ```no_run
    /// use apdl_lsk::layered_disassembler::LayeredDisassembler;
    /// # use apdl_lsk::FrameDisassembler;
    ///
    /// let mut layered = LayeredDisassembler::new();
    ///
    /// // 添加TM帧层
    /// # let tm_disassembler = FrameDisassembler::new();
    /// layered.add_layer(
    ///     "TM Frame".to_string(),
    ///     tm_disassembler,
    ///     Some("tm_data_field".to_string())
    /// );
    ///
    /// // 添加Space Packet层
    /// # let sp_disassembler = FrameDisassembler::new();
    /// layered.add_layer(
    ///     "Space Packet".to_string(),
    ///     sp_disassembler,
    ///     Some("pkt_data".to_string())
    /// );
    ///
    /// // 拆包
    /// # let raw_data = vec![0u8; 100];
    /// let result = layered.disassemble_layers(&raw_data)?;
    /// # Ok::<(), apdl_core::ProtocolError>(())
    /// ```
    pub fn disassemble_layers(
        &self,
        raw_data: &[u8],
    ) -> Result<DisassembleResult, ProtocolError> {
        let mut result = DisassembleResult::new();
        let mut current_data = raw_data.to_vec(); // 使用拥有的数据

        // 逐层拆包
        for (layer_index, layer_info) in self.layer_disassemblers.iter().enumerate() {
            // 拆包当前层
            let fields = layer_info.disassembler.disassemble_frame(&current_data)?;

            // 创建层数据
            let mut layer_data = LayerData::new(layer_info.layer_name.clone(), layer_index);

            // 添加所有字段
            for (field_name, value) in &fields {
                layer_data.add_field(field_name.clone(), value.clone());
            }

            // 提取净荷（如果有）
            if let Some(ref payload_field) = layer_info.payload_field_name {
                if let Some(payload) = fields.get(payload_field) {
                    layer_data.set_payload(payload_field.clone(), payload.clone());
                    current_data = payload.clone(); // 下一层使用净荷数据
                } else {
                    return Err(ProtocolError::FieldNotFound(format!(
                        "Payload field '{}' not found in layer {}",
                        payload_field, layer_info.layer_name
                    )));
                }
            }

            result.add_layer(layer_data);
        }

        // 最后一层的净荷就是应用数据
        if !current_data.is_empty() {
            result.set_application_data(current_data);
        }

        Ok(result)
    }

    /// 解析指定层的净荷字段
    ///
    /// # 参数
    /// - `layer_index`: 层索引（0=最外层）
    /// - `frame_data`: 该层的帧数据
    ///
    /// # 返回
    /// - `Ok(Vec<u8>)`: 净荷数据
    /// - `Err(ProtocolError)`: 提取失败
    pub fn extract_payload(
        &self,
        layer_index: usize,
        frame_data: &[u8],
    ) -> Result<Vec<u8>, ProtocolError> {
        let layer_info = self
            .layer_disassemblers
            .get(layer_index)
            .ok_or_else(|| ProtocolError::Other(format!("Layer {} not found", layer_index)))?;

        // 拆包获取所有字段
        let fields = layer_info.disassembler.disassemble_frame(frame_data)?;

        // 获取净荷字段
        if let Some(ref payload_field) = layer_info.payload_field_name {
            fields
                .get(payload_field)
                .cloned()
                .ok_or_else(|| {
                    ProtocolError::FieldNotFound(format!("Payload field '{}' not found", payload_field))
                })
        } else {
            Err(ProtocolError::Other("No payload field defined".to_string()))
        }
    }

    /// 递归拆包到应用层
    ///
    /// # 参数
    /// - `raw_data`: 原始数据
    ///
    /// # 返回
    /// - `Ok(Vec<u8>)`: 应用数据
    /// - `Err(ProtocolError)`: 拆包失败
    pub fn extract_application_data(&self, raw_data: &[u8]) -> Result<Vec<u8>, ProtocolError> {
        let result = self.disassemble_layers(raw_data)?;
        Ok(result.application_data)
    }

    /// 获取层数
    pub fn layer_count(&self) -> usize {
        self.layer_disassemblers.len()
    }

    /// 获取层名称列表
    pub fn get_layer_names(&self) -> Vec<&str> {
        self.layer_disassemblers
            .iter()
            .map(|info| info.layer_name.as_str())
            .collect()
    }
}

impl Default for LayeredDisassembler {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use apdl_core::*;

    fn create_test_layer(
        layer_name: &str,
        header_size: usize,
        payload_field_name: Option<&str>,
    ) -> (FrameDisassembler, Option<String>) {
        let mut disassembler = FrameDisassembler::new();

        // 添加简单的头部字段（使用正确的类型）
        let header_field = SyntaxUnit {
            field_id: format!("{}_header", layer_name),
            unit_type: UnitType::Uint((header_size * 8) as u8), // 转换为bit数
            length: LengthDesc {
                size: header_size,
                unit: LengthUnit::Byte,
            },
            scope: ScopeDesc::Global(layer_name.to_string()),
            cover: CoverDesc::EntireField,
            constraint: None,
            alg: None,
            associate: vec![],
            desc: "Header".to_string(),
        };

        disassembler.add_field(header_field);

        // 如果有净荷字段，添加之
        let payload_field = if let Some(field_name) = payload_field_name {
            let payload_field = SyntaxUnit {
                field_id: field_name.to_string(),
                unit_type: UnitType::RawData,
                length: LengthDesc {
                    size: 0,
                    unit: LengthUnit::Dynamic,
                },
                scope: ScopeDesc::Global(layer_name.to_string()),
                cover: CoverDesc::EntireField,
                constraint: None,
                alg: None,
                associate: vec![],
                desc: "Payload".to_string(),
            };
            disassembler.add_field(payload_field);
            Some(field_name.to_string())
        } else {
            None
        };

        (disassembler, payload_field)
    }

    #[test]
    fn test_layered_disassembler_two_layers() {
        // 创建两层协议栈
        let mut layered = LayeredDisassembler::new();

        // 外层（4字节头部 + 净荷）
        let (outer_disassembler, outer_payload) =
            create_test_layer("outer", 4, Some("outer_payload"));
        layered.add_layer("Outer Layer".to_string(), outer_disassembler, outer_payload);

        // 内层（2字节头部 + 数据）
        let (inner_disassembler, inner_payload) =
            create_test_layer("inner", 2, Some("inner_data"));
        layered.add_layer("Inner Layer".to_string(), inner_disassembler, inner_payload);

        // 构造测试数据
        // 外层头部(4字节) + 内层头部(2字节) + 应用数据(4字节)
        let test_data = vec![
            0xAA, 0xBB, 0xCC, 0xDD, // 外层头部
            0xEE, 0xFF, // 内层头部
            0x01, 0x02, 0x03, 0x04, // 应用数据
        ];

        // 拆包
        let result = layered.disassemble_layers(&test_data).unwrap();

        // 验证层数
        assert_eq!(result.layer_count(), 2);

        // 验证外层
        let outer_layer = result.get_layer(0).unwrap();
        assert_eq!(outer_layer.layer_name, "Outer Layer");
        assert_eq!(outer_layer.fields.len(), 2); // header + payload

        // 验证内层
        let inner_layer = result.get_layer(1).unwrap();
        assert_eq!(inner_layer.layer_name, "Inner Layer");

        // 验证应用数据
        // 外层拆包：header=[0xAA,0xBB,0xCC,0xDD], outer_payload=[0xEE,0xFF,0x01,0x02,0x03,0x04]
        // 内层拆包：header=[0xEE,0xFF], inner_data=[0x01,0x02,0x03,0x04]
        assert_eq!(result.application_data, vec![0x01, 0x02, 0x03, 0x04]);
    }

    #[test]
    fn test_extract_application_data() {
        let mut layered = LayeredDisassembler::new();

        // 添加两层
        let (layer1, payload1) = create_test_layer("layer1", 2, Some("payload1"));
        layered.add_layer("Layer 1".to_string(), layer1, payload1);

        let (layer2, payload2) = create_test_layer("layer2", 1, Some("payload2"));
        layered.add_layer("Layer 2".to_string(), layer2, payload2);

        // 测试数据：2字节层1头部 + 1字节层2头部 + 4字节应用数据
        let test_data = vec![0xAA, 0xBB, 0xCC, 0xDE, 0xAD, 0xBE, 0xEF];

        // 直接提取应用数据
        let app_data = layered.extract_application_data(&test_data).unwrap();
        
        // 第一层拆包后：payload1 = [0xCC, 0xDE, 0xAD, 0xBE, 0xEF] (剩余所有数据)
        // 第二层拆包后：payload2 = [0xDE, 0xAD, 0xBE, 0xEF] (剩余所有数据)
        assert_eq!(app_data, vec![0xDE, 0xAD, 0xBE, 0xEF]);
    }

    #[test]
    fn test_extract_payload() {
        let mut layered = LayeredDisassembler::new();

        let (layer1, payload1) = create_test_layer("layer1", 3, Some("payload1"));
        layered.add_layer("Layer 1".to_string(), layer1, payload1);

        // 测试数据：3字节头部 + 5字节净荷
        let test_data = vec![0xAA, 0xBB, 0xCC, 0x01, 0x02, 0x03, 0x04, 0x05];

        // 提取第0层的净荷
        // 第一层：header = [0xAA, 0xBB, 0xCC], payload1 = [0x01, 0x02, 0x03, 0x04, 0x05]
        let payload = layered.extract_payload(0, &test_data).unwrap();
        assert_eq!(payload, vec![0x01, 0x02, 0x03, 0x04, 0x05]);
    }

    #[test]
    fn test_layer_names() {
        let mut layered = LayeredDisassembler::new();

        let (layer1, payload1) = create_test_layer("layer1", 1, Some("p1"));
        layered.add_layer("Layer A".to_string(), layer1, payload1);

        let (layer2, payload2) = create_test_layer("layer2", 1, Some("p2"));
        layered.add_layer("Layer B".to_string(), layer2, payload2);

        let names = layered.get_layer_names();
        assert_eq!(names, vec!["Layer A", "Layer B"]);
        assert_eq!(layered.layer_count(), 2);
    }
}
