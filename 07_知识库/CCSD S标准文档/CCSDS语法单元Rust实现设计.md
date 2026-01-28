# CCSDS语法单元Rust实现设计

## 1. 语法单元抽象设计

### 1.1 核心Trait定义

```rust
use std::collections::HashMap;

/// 语法单元基础接口 - 实现平台与语法单元的分离
pub trait ProtocolUnit: Send + Sync {
    /// 获取单元元数据
    fn get_meta(&self) -> &UnitMeta;
    
    /// 打包：将SDU（服务数据单元）封装为PDU（协议数据单元）
    fn pack(&self, sdu: &[u8]) -> Result<Vec<u8>, ProtocolError>;
    
    /// 拆包：将PDU解析为SDU
    fn unpack(&self, pdu: &[u8]) -> Result<(Vec<u8>, &[u8]), ProtocolError>;
    
    /// 单元合法性校验
    fn validate(&self) -> Result<(), ProtocolError>;
    
    /// 获取单元配置参数
    fn get_params(&self) -> &HashMap<String, String>;
    
    /// 设置单元配置参数
    fn set_param(&mut self, key: &str, value: &str) -> Result<(), ProtocolError>;
}

/// 单元元数据结构
#[derive(Debug, Clone)]
pub struct UnitMeta {
    pub id: String,                    // 单元唯一标识
    pub name: String,                  // 单元名称
    pub version: String,               // 版本号
    pub description: String,           // 描述
    pub standard: String,              // 遵循的标准 (如 "CCSDS 132.0-B-3")
    pub layer: ProtocolLayer,          // 协议层
    pub fields: Vec<FieldDefinition>,  // 字段定义
    pub constraints: Vec<Constraint>,  // 约束条件
}

/// 协议层枚举
#[derive(Debug, Clone, PartialEq)]
pub enum ProtocolLayer {
    Physical,     // 物理层
    DataLink,     // 数据链路层
    Network,      // 网络层
    Transport,    // 传输层
    Application,  // 应用层
}

/// 字段定义
#[derive(Debug, Clone)]
pub struct FieldDefinition {
    pub name: String,      // 字段名称
    pub field_type: FieldType,  // 字段类型
    pub bit_length: u32,   // 位长度
    pub byte_offset: u32,  // 字节偏移
    pub description: String, // 描述
}

/// 字段类型
#[derive(Debug, Clone)]
pub enum FieldType {
    UInt(u32),      // 无符号整数 (位数)
    Int(u32),       // 有符号整数 (位数)
    Float,          // 浮点数
    Bytes(u32),     // 字节数组 (长度)
    BitField(u32),  // 位字段 (位数)
    Enum(Vec<String>), // 枚举类型
}
```

## 2. 标准语法单元实现

### 2.1 TM Transfer Frame 语法单元

```rust
/// TM Transfer Frame 语法单元实现
pub struct TmTransferFrameUnit {
    meta: UnitMeta,
    virtual_channel_id: u8,
    spacecraft_id: u16,
    frame_length: u16,
    has_secondary_header: bool,
    error_control: ErrorControlType,
}

impl TmTransferFrameUnit {
    pub fn new(config: &TmConfig) -> Self {
        let meta = UnitMeta {
            id: "tm_transfer_frame".to_string(),
            name: "TM Transfer Frame".to_string(),
            version: "1.0".to_string(),
            description: "CCSDS TM Transfer Frame according to CCSDS 132.0-B-3".to_string(),
            standard: "CCSDS 132.0-B-3".to_string(),
            layer: ProtocolLayer::DataLink,
            fields: vec![
                FieldDefinition {
                    name: "Transfer Frame Primary Header".to_string(),
                    field_type: FieldType::Bytes(6),
                    bit_length: 48,
                    byte_offset: 0,
                    description: "包含帧ID、虚拟信道ID等信息".to_string(),
                },
                FieldDefinition {
                    name: "Transfer Frame Data Field".to_string(),
                    field_type: FieldType::Bytes(config.frame_length),
                    bit_length: config.frame_length as u32 * 8,
                    byte_offset: 6,
                    description: "包含实际传输的数据".to_string(),
                },
            ],
            constraints: vec![],
        };

        Self {
            meta,
            virtual_channel_id: config.virtual_channel_id,
            spacecraft_id: config.spacecraft_id,
            frame_length: config.frame_length,
            has_secondary_header: config.has_secondary_header,
            error_control: config.error_control.clone(),
        }
    }
}

impl ProtocolUnit for TmTransferFrameUnit {
    fn get_meta(&self) -> &UnitMeta {
        &self.meta
    }

    fn pack(&self, sdu: &[u8]) -> Result<Vec<u8>, ProtocolError> {
        let mut pdu = Vec::with_capacity(self.frame_length as usize);
        
        // 构建主头部 (6字节)
        let frame_id = ((self.spacecraft_id as u32) << 8 | self.virtual_channel_id as u32) as u16;
        pdu.extend_from_slice(&frame_id.to_be_bytes());  // Spacecraft ID + VC ID
        
        // 序列标志和包名
        let sequence_flags = 0b01;  // 第一个段
        let packet_name = 0;        // 包名
        let seq_info = ((sequence_flags as u16) << 14) | (packet_name & 0x3FFF);
        pdu.extend_from_slice(&seq_info.to_be_bytes());
        
        // 数据字段长度
        let data_len = (sdu.len() - 1) as u16;  // CCSDS长度为(实际长度-1)
        pdu.extend_from_slice(&data_len.to_be_bytes());
        
        // 添加数据
        pdu.extend_from_slice(sdu);
        
        // 添加错误控制
        if matches!(self.error_control, ErrorControlType::BCH) {
            let ecc = calculate_bch_ecc(&pdu);
            pdu.extend_from_slice(&ecc);
        }
        
        // 确保帧长度正确
        while pdu.len() < self.frame_length as usize {
            pdu.push(0);  // 填充零
        }
        
        Ok(pdu)
    }

    fn unpack(&self, pdu: &[u8]) -> Result<(Vec<u8>, &[u8]), ProtocolError> {
        if pdu.len() < 6 {
            return Err(ProtocolError::InvalidFormat("PDU too short".to_string()));
        }
        
        // 解析主头部
        let frame_id = u16::from_be_bytes([pdu[0], pdu[1]]);
        let seq_info = u16::from_be_bytes([pdu[2], pdu[3]]);
        let data_len = u16::from_be_bytes([pdu[4], pdu[5]]) + 1;  // 长度需加1
        
        // 验证帧ID
        let expected_vc_id = frame_id as u8;
        if expected_vc_id != self.virtual_channel_id {
            return Err(ProtocolError::ValidationError(
                "Virtual channel ID mismatch".to_string()
            ));
        }
        
        // 提取数据字段
        let start = 6;
        let end = std::cmp::min(start + data_len as usize, pdu.len());
        let sdu = pdu[start..end].to_vec();
        
        // 返回(SDU, 剩余数据)
        Ok((sdu, &pdu[end..]))
    }

    fn validate(&self) -> Result<(), ProtocolError> {
        if self.virtual_channel_id > 63 {
            return Err(ProtocolError::ValidationError(
                "Virtual channel ID must be <= 63".to_string()
            ));
        }
        if self.spacecraft_id > 0x3FF {
            return Err(ProtocolError::ValidationError(
                "Spacecraft ID must be <= 1023".to_string()
            ));
        }
        if self.frame_length < 7 || self.frame_length > 1024 {
            return Err(ProtocolError::ValidationError(
                "Frame length must be between 7 and 1024 bytes".to_string()
            ));
        }
        Ok(())
    }

    fn get_params(&self) -> &HashMap<String, String> {
        let mut params = HashMap::new();
        params.insert("virtual_channel_id".to_string(), self.virtual_channel_id.to_string());
        params.insert("spacecraft_id".to_string(), self.spacecraft_id.to_string());
        params.insert("frame_length".to_string(), self.frame_length.to_string());
        params.insert("has_secondary_header".to_string(), self.has_secondary_header.to_string());
        params
    }

    fn set_param(&mut self, key: &str, value: &str) -> Result<(), ProtocolError> {
        match key {
            "virtual_channel_id" => {
                self.virtual_channel_id = value.parse::<u8>()
                    .map_err(|_| ProtocolError::InvalidFormat("Invalid virtual channel ID".to_string()))?;
            }
            "spacecraft_id" => {
                self.spacecraft_id = value.parse::<u16>()
                    .map_err(|_| ProtocolError::InvalidFormat("Invalid spacecraft ID".to_string()))?;
            }
            "frame_length" => {
                self.frame_length = value.parse::<u16>()
                    .map_err(|_| ProtocolError::InvalidFormat("Invalid frame length".to_string()))?;
            }
            "has_secondary_header" => {
                self.has_secondary_header = value.parse::<bool>()
                    .map_err(|_| ProtocolError::InvalidFormat("Invalid boolean value".to_string()))?;
            }
            _ => return Err(ProtocolError::UnknownParameter(key.to_string())),
        }
        Ok(())
    }
}

/// TM配置结构
#[derive(Debug, Clone)]
pub struct TmConfig {
    pub virtual_channel_id: u8,
    pub spacecraft_id: u16,
    pub frame_length: u16,
    pub has_secondary_header: bool,
    pub error_control: ErrorControlType,
}

/// 错误控制类型
#[derive(Debug, Clone)]
pub enum ErrorControlType {
    None,  // 无错误控制
    CRC,   // 循环冗余校验
    BCH,   // BCH纠错码
}
```

### 2.2 Space Packet 语法单元

```rust
/// Space Packet 语法单元实现
pub struct SpacePacketUnit {
    meta: UnitMeta,
    apid: u16,  // Application Process ID
    packet_type: PacketType,
    secondary_header_flag: bool,
    sequence_flags: SequenceFlags,
}

impl SpacePacketUnit {
    pub fn new(apid: u16, packet_type: PacketType) -> Self {
        let meta = UnitMeta {
            id: format!("space_packet_{}", apid),
            name: format!("Space Packet APID {}", apid),
            version: "1.0".to_string(),
            description: "CCSDS Space Packet according to CCSDS 133.0-B-3".to_string(),
            standard: "CCSDS 133.0-B-3".to_string(),
            layer: ProtocolLayer::Network,
            fields: vec![
                FieldDefinition {
                    name: "Packet Primary Header".to_string(),
                    field_type: FieldType::Bytes(6),
                    bit_length: 48,
                    byte_offset: 0,
                    description: "包含版本号、包类型、APID等信息".to_string(),
                },
                FieldDefinition {
                    name: "Packet Data Field".to_string(),
                    field_type: FieldType::Bytes(0),  // 可变长度
                    bit_length: 0,  // 可变长度
                    byte_offset: 6,
                    description: "包含二级头部和用户数据".to_string(),
                },
            ],
            constraints: vec![],
        };

        Self {
            meta,
            apid,
            packet_type,
            secondary_header_flag: false,
            sequence_flags: SequenceFlags::First,
        }
    }
}

impl ProtocolUnit for SpacePacketUnit {
    fn get_meta(&self) -> &UnitMeta {
        &self.meta
    }

    fn pack(&self, sdu: &[u8]) -> Result<Vec<u8>, ProtocolError> {
        let mut pdu = Vec::with_capacity(sdu.len() + 10);  // 预留头部空间
        
        // 构建主头部 (6字节)
        // 版本号(3bit) + 包类型(1bit) + 二级头部标志(1bit) + APID(11bit)
        let version_and_apid = ((1u16 & 0x7) << 13) |  // 版本号 1
                              (((self.packet_type as u16) & 0x1) << 12) |  // 包类型
                              ((self.secondary_header_flag as u16) << 11) |  // 二级头部标志
                              (self.apid & 0x7FF);  // APID
        pdu.extend_from_slice(&version_and_apid.to_be_bytes());
        
        // 序列标志(2bit) + 序列计数器(14bit)
        let seq_flags_and_count = (((self.sequence_flags as u16) & 0x3) << 14) |
                                  (0 & 0x3FFF);  // 序列计数器，这里简化为0
        pdu.extend_from_slice(&seq_flags_and_count.to_be_bytes());
        
        // 包数据长度 (SDU长度 - 1)
        let data_len = (sdu.len() + if self.secondary_header_flag { 4 } else { 0 } - 1) as u16;
        pdu.extend_from_slice(&data_len.to_be_bytes());
        
        // 添加二级头部 (如果需要)
        if self.secondary_header_flag {
            // 这里可以添加时间戳或其他二级头部信息
            pdu.extend_from_slice(&[0, 0, 0, 0]);  // 示例二级头部
        }
        
        // 添加用户数据
        pdu.extend_from_slice(sdu);
        
        Ok(pdu)
    }

    fn unpack(&self, pdu: &[u8]) -> Result<(Vec<u8>, &[u8]), ProtocolError> {
        if pdu.len() < 6 {
            return Err(ProtocolError::InvalidFormat("PDU too short".to_string()));
        }
        
        // 解析主头部
        let version_and_apid = u16::from_be_bytes([pdu[0], pdu[1]]);
        let version = (version_and_apid >> 13) & 0x7;
        let packet_type_val = (version_and_apid >> 12) & 0x1;
        let sec_header_flag = (version_and_apid >> 11) & 0x1 == 1;
        let apid = version_and_apid & 0x7FF;
        
        // 验证APID
        if apid != self.apid {
            return Err(ProtocolError::ValidationError("APID mismatch".to_string()));
        }
        
        // 验证版本
        if version != 1 {
            return Err(ProtocolError::ValidationError("Unsupported version".to_string()));
        }
        
        let seq_info = u16::from_be_bytes([pdu[2], pdu[3]]);
        let data_len = u16::from_be_bytes([pdu[4], pdu[5]]) + 1;  // 长度需加1
        
        // 计算数据起始位置
        let mut start = 6;  // 跳过主头部
        if sec_header_flag {
            start += 4;  // 跳过二级头部
        }
        
        // 提取用户数据
        let end = std::cmp::min(start + (data_len as usize), pdu.len());
        let sdu = pdu[start..end].to_vec();
        
        // 返回(SDU, 剩余数据)
        Ok((sdu, &pdu[end..]))
    }

    fn validate(&self) -> Result<(), ProtocolError> {
        if self.apid > 0x7FF {  // 11位限制
            return Err(ProtocolError::ValidationError(
                "APID must be <= 2047".to_string()
            ));
        }
        Ok(())
    }

    fn get_params(&self) -> &HashMap<String, String> {
        let mut params = HashMap::new();
        params.insert("apid".to_string(), self.apid.to_string());
        params.insert("packet_type".to_string(), format!("{:?}", self.packet_type));
        params.insert("secondary_header_flag".to_string(), self.secondary_header_flag.to_string());
        params
    }

    fn set_param(&mut self, key: &str, value: &str) -> Result<(), ProtocolError> {
        match key {
            "apid" => {
                self.apid = value.parse::<u16>()
                    .map_err(|_| ProtocolError::InvalidFormat("Invalid APID".to_string()))?;
            }
            "packet_type" => {
                self.packet_type = match value {
                    "Telemetry" => PacketType::Telemetry,
                    "Telecommand" => PacketType::Telecommand,
                    _ => return Err(ProtocolError::InvalidFormat("Invalid packet type".to_string())),
                };
            }
            "secondary_header_flag" => {
                self.secondary_header_flag = value.parse::<bool>()
                    .map_err(|_| ProtocolError::InvalidFormat("Invalid boolean value".to_string()))?;
            }
            _ => return Err(ProtocolError::UnknownParameter(key.to_string())),
        }
        Ok(())
    }
}

/// 包类型枚举
#[derive(Debug, Clone)]
pub enum PacketType {
    Telemetry = 0,
    Telecommand = 1,
}

/// 序列标志枚举
#[derive(Debug, Clone)]
pub enum SequenceFlags {
    First = 0,
    Continue = 1,
    Last = 2,
    Standalone = 3,
}
```

## 3. 语法单元组合设计

### 3.1 协议栈管理器

```rust
use std::sync::Arc;

/// 协议栈管理器 - 管理多个语法单元的组合
pub struct ProtocolStack {
    units: Vec<Arc<dyn ProtocolUnit>>,
    name: String,
    description: String,
}

impl ProtocolStack {
    pub fn new(name: &str, description: &str) -> Self {
        Self {
            units: Vec::new(),
            name: name.to_string(),
            description: description.to_string(),
        }
    }
    
    /// 添加语法单元到协议栈
    pub fn add_unit(&mut self, unit: Arc<dyn ProtocolUnit>) {
        self.units.push(unit);
    }
    
    /// 从SDU到PDU的完整打包过程（从底层到高层）
    pub fn pack(&self, mut sdu: Vec<u8>) -> Result<Vec<u8>, ProtocolError> {
        for unit in &self.units {
            sdu = unit.pack(&sdu)?;
        }
        Ok(sdu)
    }
    
    /// 从PDU到SDU的完整拆包过程（从高层到底层）
    pub fn unpack(&self, mut pdu: Vec<u8>) -> Result<Vec<u8>, ProtocolError> {
        for unit in self.units.iter().rev() {  // 反向处理
            let (sdu, _) = unit.unpack(&pdu)?;
            pdu = sdu;
        }
        Ok(pdu)
    }
    
    /// 获取协议栈信息
    pub fn get_info(&self) -> ProtocolStackInfo {
        ProtocolStackInfo {
            name: self.name.clone(),
            description: self.description.clone(),
            unit_count: self.units.len(),
            units: self.units.iter().map(|u| u.get_meta().name.clone()).collect(),
        }
    }
}

/// 协议栈信息
#[derive(Debug)]
pub struct ProtocolStackInfo {
    pub name: String,
    pub description: String,
    pub unit_count: usize,
    pub units: Vec<String>,
}
```

## 4. DSL语法设计

### 4.1 协议定义DSL

```rust
/// 协议定义DSL示例
/*
ProtocolDef "CCSDS_TM_STACK" {
    Version: "1.0",
    Desc: "CCSDS TM协议栈：Space Packet封装在TM帧中",
    
    Layers {
        // Space Packet层（应用层）
        Layer SP [Standard(CCSDS_133_0_B_3)] {
            Params: {
                APID: 100,
                Type: Telemetry,
                SecondaryHeader: false
            },
            Fields {
                Version: u3 [Range(0, 7)],
                Type: u1 [Range(0, 1)],
                APID: u11 [Range(0, 2047)],
                SeqFlags: u2 [Enum(First, Continue, Last, Standalone)],
                SeqCount: u14,
                DataLen: u16
            }
        }
        
        // TM Transfer Frame层（数据链路层）
        Layer TM [Standard(CCSDS_132_0_B_3)] {
            Params: {
                VC_ID: 5,
                SC_ID: 123,
                FrameLength: 1024,
                ErrorControl: BCH
            },
            Fields {
                FrameID: u16,
                VC_ID: u6 [Range(0, 63)],
                FrameSeq: u2 [Enum(First, Continue, Last, Unused)],
                DataLen: u16
            }
        }
    }
    
    // 层间关系定义
    Relation {
        SP -> TM: "Encapsulation"  // Space Packet封装到TM帧中
    }
    
    // 注册为可复用单元
    RegisterUnit to StandardUnitLib;
}
*/
```

## 5. 项目需求合理性验证

### 5.1 技术合理性
1. **模块化设计**：每个语法单元独立实现，便于测试和维护
2. **接口统一**：所有单元实现统一的ProtocolUnit接口，保证互操作性
3. **扩展性良好**：支持自定义语法单元，只需实现ProtocolUnit接口
4. **内存安全**：利用Rust的所有权系统，避免内存安全问题

### 5.2 功能合理性
1. **标准兼容**：严格按照CCSDS标准实现，确保协议兼容性
2. **层次清晰**：按协议层次组织代码结构，符合协议设计原理
3. **组合灵活**：支持多种语法单元的自由组合，满足不同应用场景

### 5.3 实现可行性
1. **Rust优势**：利用Rust的性能和安全性，适合航天领域的高可靠性要求
2. **Trait系统**：充分利用Rust的Trait系统实现多态，支持语法单元的动态组合
3. **生态系统**：Rust丰富的生态支持各种协议解析需求

通过这种设计，我们可以确保项目在需求层面具有高度的合理性和可实现性，为后续的开发工作奠定坚实基础。