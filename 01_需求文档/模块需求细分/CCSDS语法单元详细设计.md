# CCSDS协议语法单元详细设计

## 1. 设计概述

### 1.1 目标
本设计文档详细描述了APDL系统中CCSDS协议语法单元的实现方案，确保协议单元能够正确实现CCSDS标准，并支持灵活的组合和扩展。

### 1.2 设计原则
- 标准兼容性：严格遵循CCSDS标准规范
- 模块化：每个语法单元独立实现
- 可扩展：支持自定义语法单元
- 互操作：单元间接口统一

## 2. 语法单元架构设计

### 2.1 核心接口定义

```rust
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
```

### 2.2 元数据结构

```rust
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
```

## 3. 标准语法单元详细设计

### 3.1 TM Transfer Frame 语法单元

#### 3.1.1 功能描述
实现CCSDS 132.0-B-3标准的TM Transfer Frame，用于遥测数据的传输。

#### 3.1.2 数据结构
```rust
pub struct TmTransferFrameUnit {
    meta: UnitMeta,
    virtual_channel_id: u8,        // 虚拟通道ID (0-63)
    spacecraft_id: u16,            // 航天器ID (0-1023)
    frame_length: u16,             // 帧长度
    has_secondary_header: bool,    // 是否有二级头部
    error_control: ErrorControlType, // 错误控制类型
}
```

#### 3.1.3 字段定义
- **Transfer Frame Primary Header** (6字节):
  - Transfer Frame ID: 16位 (包含Spacecraft ID和VC ID)
  - Sequence Flags: 2位
  - Packet Name: 14位
  - Data Field Length: 16位

#### 3.1.4 约束条件
- Virtual Channel ID: 0 ≤ VC ID ≤ 63
- Spacecraft ID: 0 ≤ SC ID ≤ 1023
- Frame Length: 7 ≤ Frame Length ≤ 1024 (以字节为单位)

#### 3.1.5 打包流程
1. 验证输入数据和单元配置
2. 构建主头部 (6字节)
3. 添加数据字段
4. 应用错误控制
5. 填充至指定长度

#### 3.1.6 拆包流程
1. 验证PDU长度
2. 解析主头部
3. 验证帧ID匹配
4. 提取数据字段
5. 返回(SDU, 剩余数据)

### 3.2 TC Transfer Frame 语法单元

#### 3.2.1 功能描述
实现CCSDS 232.0-B-3标准的TC Transfer Frame，用于遥控命令的传输。

#### 3.2.2 数据结构
```rust
pub struct TcTransferFrameUnit {
    meta: UnitMeta,
    spacecraft_id: u16,            // 航天器ID
    virtual_channel_id: u8,        // 虚拟通道ID
    has_segmentation_hdr: bool,    // 是否有分段头部
    has_farm_hdr: bool,            // 是否有FARM头部
    has_farm_crc: bool,            // 是否有FARM CRC
}
```

#### 3.2.3 字段定义
- **Primary Header** (5字节):
  - Version Number: 2位
  - Type: 1位 (始终为1表示TC)
  - Sec. Header Flag: 1位
  - TC Spacecraft ID: 10位
  - Virtual Channel ID: 3位
  - Retransmit Flag: 1位
  - TC Packet Name: 16位
  - Segmentation Flags: 2位
  - Packet Data Length: 14位

#### 3.2.4 约束条件
- Spacecraft ID: 0 ≤ SC ID ≤ 1023
- Virtual Channel ID: 0 ≤ VC ID ≤ 7
- Packet Data Length: 0 ≤ Length ≤ 1017 (当使用BCH-24)

#### 3.2.5 处理流程
- 打包: 构建头部 → 添加数据 → 应用安全机制
- 拆包: 解析头部 → 验证身份 → 提取数据

### 3.3 Space Packet 语法单元

#### 3.3.1 功能描述
实现CCSDS 133.0-B-3标准的Space Packet，用于应用数据的封装。

#### 3.3.2 数据结构
```rust
pub struct SpacePacketUnit {
    meta: UnitMeta,
    apid: u16,                     // 应用进程ID
    packet_type: PacketType,       // 包类型 (TM/TC)
    secondary_header_flag: bool,   // 二级头部标志
    sequence_flags: SequenceFlags, // 序列标志
}
```

#### 3.3.3 字段定义
- **Primary Header** (6字节):
  - Version Number: 3位
  - Packet Type: 1位
  - Secondary Header Flag: 1位
  - APID: 11位
  - Sequence Flags: 2位
  - Packet Sequence Count: 14位
  - Packet Data Length: 16位

#### 3.3.4 约束条件
- APID: 0 ≤ APID ≤ 2047
- Version Number: 0 ≤ Version ≤ 7
- Sequence Flags: 0 ≤ Flags ≤ 3

### 3.4 AOS Transfer Frame 语法单元

#### 3.4.1 功能描述
实现AOS协议的Transfer Frame，用于高级在轨系统数据传输。

#### 3.4.2 数据结构
```rust
pub struct AosTransferFrameUnit {
    meta: UnitMeta,
    master_channel_id: u8,         // 主信道ID
    virtual_channel_id: u8,        // 虚拟信道ID
    frame_length: u16,             // 帧长度
    has_insert_zone: bool,         // 是否有插入区
}
```

## 4. 语法单元组合设计

### 4.1 协议栈管理器
```rust
pub struct ProtocolStack {
    units: Vec<Arc<dyn ProtocolUnit>>, // 语法单元列表
    name: String,                      // 协议栈名称
    description: String,               // 描述
}
```

### 4.2 组合策略
1. **垂直组合**: 上层协议封装在下层协议中
   - 例如：Space Packet → TM Transfer Frame
   
2. **水平组合**: 同层协议的并行处理
   - 例如：多路TM虚拟信道复接

3. **混合组合**: 复杂的协议栈结构
   - 例如：多层协议的嵌套和复用

## 5. DSL语法设计

### 5.1 协议定义语法
```
ProtocolDef "PROTOCOL_NAME" {
    Version: "VERSION_STRING",
    Desc: "DESCRIPTION",
    
    Layers {
        Layer LAYER_NAME [Standard(CCSDS_STANDARD)] {
            Params: {
                PARAM_NAME: PARAM_VALUE,
                ...
            },
            Fields {
                FIELD_NAME: TYPE [CONSTRAINTS],
                ...
            }
        }
    }
    
    Relation {
        LAYER1 -> LAYER2: "RELATION_TYPE"
    }
    
    RegisterUnit to UNIT_LIBRARY;
}
```

### 5.2 示例：TM协议栈
```
ProtocolDef "CCSDS_TM_STACK" {
    Version: "1.0",
    Desc: "CCSDS TM协议栈：Space Packet封装在TM帧中",
    
    Layers {
        Layer SP [Standard(CCSDS_133_0_B_3)] {
            Params: {
                APID: 100,
                Type: Telemetry,
                SecondaryHeader: false
            }
        }
        
        Layer TM [Standard(CCSDS_132_0_B_3)] {
            Params: {
                VC_ID: 5,
                SC_ID: 123,
                FrameLength: 1024,
                ErrorControl: BCH
            }
        }
    }
    
    Relation {
        SP -> TM: "Encapsulation"
    }
    
    RegisterUnit to StandardUnitLib;
}
```

## 6. 错误处理设计

### 6.1 错误类型定义
```rust
#[derive(Debug)]
pub enum ProtocolError {
    InvalidFormat(String),
    ValidationError(String),
    UnknownParameter(String),
    NotImplemented,
    IoError(std::io::Error),
}
```

### 6.2 错误处理策略
- 输入验证：在处理前验证参数和数据
- 早期失败：发现问题立即返回错误
- 详细信息：提供足够的错误上下文信息
- 可恢复性：设计可恢复的错误处理机制

## 7. 性能优化考虑

### 7.1 内存管理
- 预分配缓冲区减少内存分配
- 重用中间数据结构
- 使用零拷贝技术

### 7.2 计算优化
- 批量处理数据
- 并行处理多路数据流
- 优化关键算法

### 7.3 并发设计
- 使用Rust的并发原语
- 避免共享状态
- 利用异步处理

## 8. 测试策略

### 8.1 单元测试
- 每个语法单元独立测试
- 边界条件测试
- 错误路径测试

### 8.2 集成测试
- 语法单元组合测试
- 端到端协议栈测试
- 性能基准测试

### 8.3 标准合规性测试
- 与CCSDS标准一致性验证
- 与参考实现对比测试
- 交叉验证测试

## 9. 扩展性设计

### 9.1 插件架构
- 支持第三方语法单元
- 动态加载机制
- 版本兼容性

### 9.2 配置管理
- 运行时配置更新
- 参数验证机制
- 配置持久化

## 10. 安全性考虑

### 10.1 输入验证
- 严格的输入数据验证
- 防止缓冲区溢出
- 类型安全保证

### 10.2 内存安全
- 利用Rust所有权系统
- 避免未定义行为
- 防止内存泄漏

通过以上详细设计，我们确保了CCSDS协议语法单元的实现既符合标准要求，又具备良好的扩展性和可维护性，从而保证了项目需求的合理性。