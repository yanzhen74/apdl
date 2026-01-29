# CCSDS协议语法单元分析

## 1. CCSDS协议概述

CCSDS（Consultative Committee for Space Data Systems，空间数据系统咨询委员会）是由世界主要航天国家组成的国际组织，致力于制定空间数据通信标准。CCSDS协议族是航天领域最重要的通信协议标准，广泛应用于卫星通信、深空探测等任务中。

## 2. CCSDS协议层次结构

CCSDS协议采用分层架构，主要包括以下几个层次：

### 2.1 物理层（Physical Layer）
- 负责信号的调制、解调和传输
- 定义了信号频率、调制方式、功率等参数

### 2.2 数据链路层（Data Link Layer）
- 包括TM（Telemetry）和TC（Telecommand）协议
- 负责数据的可靠传输
- 主要协议：
  - CCSDS 132.0-B-3: TM Transfer Frame
  - CCSDS 232.0-B-3: TC Transfer Frame
  - CCSDS 231.0-B-3: TC Synchronization and Channel Coding

### 2.3 网络层（Network Layer）
- AOS（Advanced Orbiting Systems）协议
- 提供多路复用和路由功能

### 2.4 传输层（Transport Layer）
- CCSDS文件传送协议（CFDP, CCSDS 727.0-B-5）
- 提供可靠的文件传输服务

### 2.5 应用层（Application Layer）
- 任务特定的应用协议
- 如科学数据处理、遥测控制等

## 3. DSL规范与JSON协议描述

### 3.1 语法单元DSL通用规范

#### 3.1.1 基础结构
```
field: [唯一 ID]; // 必选，格式：协议_类型_功能
type: [数据类型]; // 必选，如 Uint16/Bit(2)/RawData/Ip6Addr
length: [长度]; // 必选，如 2byte/dynamic/calc(公式)
scope: [作用范围]; // 必选，如 layer(link)/cross_layer(net→link)/global(end2end)
cover: [数据覆盖]; // 必选，如 frame_header[0..1]/$cover
[可选字段]; // constraint/alg/associate/desc/role 等
```

#### 3.1.2 数据类型与约束规则
| 数据类型 | 约束规则示例 |
|----------|-------------|
| UintN | fixed(0xEB90)/range(0..=1023) |
| Bit(N) | enum(00=single,01=first) |
| RawData | dynamic，关联其他单元 |
| Ip6Addr | scps_ip_format |

### 3.2 CCSDS分包遥测协议JSON描述

#### 3.2.1 JSON协议描述结构
```json
{
  "protocol_meta": {
    "protocol_id": "CCSD_TM_PACKET_001",
    "protocol_name": "CCSDS分包遥测协议",
    "version": "V1.0",
    "base_standard": "CCSDS 132.0-B-2,CCSDS 143.0-B-1",
    "max_frame_len": 4096,
    "max_subpkg_count": 8
  },
  "syntax_units": [
    {
      "field_id": "SYNC_MARKER_001",
      "dsl_def": "field: SYNC_MARKER_001; type: Uint16; length: 2byte; scope: layer(ccsd_tm_frame_header); cover: ccsd_tm_frame_header[0..1]; constraint: fixed(0xEB90); desc: \"CCSDS 132.0-B-2 4.2.1\"; role: mandatory;"
    },
    {
      "field_id": "PRIMARY_PTR_001",
      "dsl_def": "field: PRIMARY_PTR_001; type: Uint16; length: 2byte; scope: layer(ccsd_tm_data_area); cover: ccsd_tm_data_area[0..1]; constraint: range(0..=4096); associate: TM_SUBPKG_001; desc: \"首个子包长度\"; role: primary_ptr;"
    }
  ],
  "frame_structure": {
    "frame_header": {
      "unit_list": [
        {"field_id": "SYNC_MARKER_001", "byte_offset": 0},
        {"field_id": "SEQ_FLAG_001", "byte_offset": 2},
        {"field_id": "SEQ_NUM_001", "byte_offset": 4}
      ],
      "total_len": 6
    },
    "frame_data_area": {
      "total_len_expr": "protocol_meta.max_frame_len - frame_header.total_len - frame_tail.total_len",
      "subpkg_nesting": {
        "subpkg_proto_id": "CCSD_TM_SUBPKG_001",
        "subpkg_structure": {
          "subpkg_header": {"unit_list": [{"field_id": "TM_SUBPKG_ID_001", "byte_offset": 0}], "total_len": 2},
          "subpkg_data": {"len_expr": "PRIMARY_PTR_001.value - 4", "data_type": "RawData"},
          "subpkg_tail": {"unit_list": [{"field_id": "TM_SUBPKG_CRC_001", "byte_offset": 0}], "total_len": 2}
        },
        "subpkg_location": [
          {"subpkg_id": "TM_SUBPKG_001", "start_expr": "frame_data_area.byte_offset + 2", "len_expr": "PRIMARY_PTR_001.value"}
        ]
      }
    },
    "frame_tail": {
      "unit_list": [{"field_id": "FRAME_CRC_001", "byte_offset": 0}],
      "total_len": 2
    }
  },
  "pack_unpack_spec": {
    "pack_order": [
      {"step":1, "operation":"assemble_frame_header", "target":"frame_header", "detail":"按 SYNC→FLAG→NUM 顺序组装"},
      {"step":2, "operation":"assemble_subpkg", "target":"frame_data_area", "detail":"计算子包长度→写入主指针→组装子包"},
      {"step":3, "operation":"calculate_crc", "target":"frame_tail", "detail":"计算帧头+数据区 CRC→写入 FRAME_CRC_001"},
      {"step":4, "operation":"merge_frame", "target":"ccsd_tm_frame", "detail":"拼接帧头→数据区→帧尾"}
    ],
    "unpack_order": [
      {"step":1, "operation":"extract_header", "target":"frame_header", "detail":"验证同步码→解析序列信息"},
      {"step":2, "operation":"validate_crc", "target":"frame_tail", "detail":"比对CRC→错误则丢弃"},
      {"step":3, "operation":"extract_subpkg", "target":"frame_data_area", "detail":"按主指针提取子包→验证子包 CRC"},
      {"step":4, "operation":"output", "target":"unpacked_data", "detail":"输出序列信息+有效子包"}
    ]
  }
}
```

## 4. Rust开发适配与模块接口

### 4.1 核心模块接口定义

#### 4.1.1 DSL引擎接口
```rust
pub trait DslParser {
    fn parse_dsl(&self, dsl_text: &str) -> Result<SyntaxUnit, DslParseError>;
    fn validate_dsl(&self, dsl_text: &str) -> Result<(), DslValidateError>;
}

#[derive(Debug, Clone)]
pub struct SyntaxUnit {
    pub field_id: String,
    pub unit_type: UnitType,
    pub length: LengthDesc,
    pub scope: ScopeDesc,
    pub cover: CoverDesc,
    pub constraint: Option<Constraint>,
    pub alg: Option<AlgorithmAst>,
    pub associate: Vec<String>,
    pub desc: String,
}
```

#### 4.1.2 协议组装平台接口
```rust
pub trait ProtocolAssembler {
    fn add_syntax_unit(&mut self, unit: SyntaxUnit);
    fn define_layer(&mut self, layer: ProtocolLayer);
    fn generate_spec(&self) -> Result<String, AssembleError>;
}

#[derive(Debug, Clone)]
pub struct ProtocolLayer {
    pub layer_id: String,
    pub unit_ids: Vec<String>,
    pub lower_layer: Option<String>,
}
```

### 4.2 Rust依赖库选择

| 需求场景 | Rust库选择 | 用途说明 |
|----------|------------|----------|
| DSL解析 | nom | 高效解析DSL语法，生成抽象语法树 |
| 多格式文档解析 | docx-rs / calamine / pdf-rs | 解析Word/Excel/PDF表格，提取字段 |
| CRC计算 | crc | 实现CCSDS/CAN标准CRC算法 |
| 异步链路仿真 | tokio + bytes | 高并发帧发送/接收，高效处理二进制数据 |
| 自定义脚本执行 | rhai | 安全执行用户自定义DSL算法逻辑 |
| 错误处理 | anyhow + thiserror | 统一错误类型，便于跨模块处理 |

## 5. 核心语法单元分类

### 5.1 帧结构基础单元（控制帧/数据帧通用骨架）

#### 5.1.1 LLC帧同步序列单元
- **功能**：帧边界检测，提供同步标记
- **字段约束**：固定值 0xEB90（2字节）
- **DSL描述**：
```
field: llc_sync_marker, 
type: Uint16, 
length: 2byte, 
constraint: fixed(0xEB90), 
desc: "CCSDS 132.0-B-2 4.2.1"
```

#### 5.1.2 LLC帧类型标识单元
- **功能**：区分数据帧与控制帧
- **字段约束**：1比特，0=数据帧，1=控制帧
- **DSL描述**：
```
field: llc_frame_type, 
type: Bit(1), 
length: 1bit, 
constraint: enum(0=data,1=control), 
desc: "CCSDS 132.0-B-2 4.2.2.1"
```

#### 5.1.3 LLC帧长度字段单元
- **功能**：标识帧的实际长度
- **字段约束**：16比特，0~1023，实际长度=字段值+1
- **DSL描述**：
```
field: llc_frame_len, 
type: Uint16, 
length: 2byte, 
constraint: range(0..=1023), 
calc: actual_len=value+1, 
desc: "CCSDS 132.0-B-2 4.2.2.3"
```

### 5.2 信道与地址单元（数据路由定位）

#### 5.2.1 虚拟信道标识单元
- **功能**：标识数据传输的虚拟信道
- **字段约束**：6比特，0~63，其中0~3为预留信道
- **DSL描述**：
```
field: net_vc_id, 
type: Bit(6), 
length: 6bit, 
constraint: range(0..=63), 
reserve: 0~3, 
desc: "CCSDS 142.0-B-1 5.3.1"
```

#### 5.2.2 目标地址单元
- **功能**：指定数据包的目标地址
- **字段约束**：32比特，IPv6压缩格式，符合SCPS-IP规则
- **DSL描述**：
```
field: net_dst_addr, 
type: Ip6Addr, 
length: 4byte, 
constraint: scps_ip_format, 
desc: "CCSDS 142.0-B-1 6.2.2"
```

### 5.3 数据完整性单元（错误检测/重传）

#### 5.3.1 CRC-16校验单元
- **功能**：检测数据传输错误
- **字段约束**：16比特，多项式0x1021，覆盖帧控制→用户数据
- **DSL描述**：
```
field: llc_crc16, 
type: Uint16, 
length: 2byte, 
constraint: crc(poly=0x1021, cover=frame_ctrl..user_data), 
desc: "CCSDS 132.0-B-2 4.2.4"
```

#### 5.3.2 帧序号单元
- **功能**：检测丢帧情况
- **字段约束**：8比特，0~255，递增循环
- **DSL描述**：
```
field: llc_frame_seq, 
type: Uint8, 
length: 1byte, 
constraint: increment(step=1, wrap=255), 
desc: "CCSDS 132.0-B-2 4.2.2.2"
```

### 5.4 数据控制单元（分段/复接逻辑）

#### 5.4.1 分段标识单元
- **功能**：标识数据包的分段类型，用于子包重组
- **字段约束**：2比特，00=首段，01=中段，10=尾段，11=单段
- **DSL描述**：
```
field: sp_seg_flag, 
type: Bit(2), 
length: 2bit, 
constraint: enum(00=first,01=middle,10=last,11=single), 
desc: "CCSDS 143.0-B-1 4.2.1"
```

#### 5.4.2 数据长度指示单元
- **功能**：定位数据边界
- **字段约束**：16比特，0~4095
- **DSL描述**：
```
field: dli_len, 
type: Uint16, 
length: 2byte, 
constraint: range(0..=4095), 
desc: "CCSDS 143.0-B-1 4.2.3"
```

### 5.5 用户自定义扩展单元（协议扩展）

#### 5.5.1 设备状态码单元
- **功能**：扩展设备状态信息
- **字段约束**：8比特，0=正常，1=告警，2=故障，复用链路层扩展字段
- **DSL描述**：
```
field: user_dev_status, 
type: Uint8, 
length: 1byte, 
constraint: enum(0=normal,1=alert,2=fault), 
extend: yes, 
desc: "CCSDS 132.0-B-2 4.2.5 用户扩展"
```

#### 5.5.2 自定义时间戳单元
- **功能**：添加时间标记到遥测数据
- **字段约束**：32比特，UNIX秒级时间戳，新增于用户数据区头部
- **DSL描述**：
```
field: user_timestamp, 
type: Uint32, 
length: 4byte, 
constraint: unix_time(sec), 
extend: yes, 
desc: "遥测数据时间标记"
```

## 6. 帧结构关键单元深化分析

### 6.1 帧数据区嵌套单元（导头指针）

#### 6.1.1 主导头指针单元
- **功能**：定位帧数据区的首个子包
- **字段约束**：16比特，范围0到帧数据区总长度，关联子包数据区
- **作用范围**：帧数据区层
- **DSL描述**：
```
field: subpkg_primary_ptr,
type: Uint16,
length: 2byte,
scope: layer(frame_data),
cover: frame_data[0..1],
constraint: range(0..=frame_data.total_len),
associate: subpkg_data_area,
desc: "帧数据区首个子包定位，标识子包长度"
```

#### 6.1.2 副导头指针单元
- **功能**：定位帧数据区的后续子包
- **字段约束**：16比特，范围从主指针值之后到帧数据区末尾，关联主指针和第二个子包数据区
- **作用范围**：帧数据区层
- **DSL描述**：
```
field: subpkg_secondary_ptr,
type: Uint16,
length: 2byte,
scope: layer(frame_data),
cover: frame_data[2+primary_ptr.value .. 4+primary_ptr.value],
constraint: range(0..=frame_data.total_len - (4+primary_ptr.value)),
associate: subpkg_primary_ptr, subpkg_data_area_2,
desc: "帧数据区后续子包定位，标识下子包偏移"
```

### 6.2 帧序列管理单元（标志+序列号）

#### 6.2.1 帧序列标志单元
- **功能**：区分帧的角色（单帧/首帧/中间帧/尾帧）
- **字段约束**：2比特，枚举值00=单帧，01=首帧，10=中间帧，11=尾帧，关联帧序列号
- **作用范围**：帧头部层
- **DSL描述**：
```
field: frame_seq_flag,
type: Bit(2),
length: 2bit,
scope: layer(frame_header),
cover: frame_header[2..3],
constraint: enum(00=single,01=first,10=middle,11=last),
associate: frame_seq_num,
desc: "多帧重组标识，区分单帧/首帧/中间帧/尾帧"
```

#### 6.2.2 帧序列号单元
- **功能**：标识帧的顺序，用于丢帧检测和重复过滤
- **字段约束**：8比特，递增步长1，循环值255，关联帧序列标志，接收端检查规则
- **作用范围**：帧头部层
- **DSL描述**：
```
field: frame_seq_num,
type: Uint8,
length: 1byte,
scope: layer(frame_header),
cover: frame_header[4..5],
constraint: increment(step=1, wrap=255),
associate: frame_seq_flag,
check_rule: receiver_check(prev_num+1==current_num),
desc: "帧顺序标识，丢帧检测/重复过滤"
```

### 6.3 分路单元（数据分发）

#### 6.3.1 基础分路标识单元
- **功能**：提供基础的数据分路依据
- **字段约束**：6比特，枚举值0=TM主信道，1=TM备份信道，2=TC主信道，3=TC备份信道，必需字段
- **作用范围**：跨层（网络层→链路层）
- **DSL描述**：
```
field: basic_routing_vc_id,
type: Bit(6),
length: 6bit,
scope: cross_layer(net→link),
cover: net_header[0..5],
constraint: enum(0=tm_main,1=tm_backup,2=tc_main,3=tc_backup),
routing_role: mandatory,
desc: "CCSDS虚拟信道ID，基础分路依据"
```

#### 6.3.2 扩展分路标识单元
- **功能**：提供精细化的业务类型分路
- **字段约束**：8比特，枚举值0x01=温度，0x02=电压，0x03=电流，可选字段，与基础分路标识组合
- **作用范围**：应用层
- **DSL描述**：
```
field: ext_routing_service_type,
type: Uint8,
length: 1byte,
scope: layer(app),
cover: app_header[2..3],
constraint: enum(0x01=temp,0x02=voltage,0x03=current),
routing_role: optional,
routing_combine: basic_routing_vc_id + self,
desc: "业务类型标识，精细化分路"
```

## 7. CAN总线协议语法单元扩展

### 7.1 总线仲裁类单元（CAN独有）

#### 7.1.1 仲裁场单元
- **功能**：总线仲裁，确定消息优先级
- **字段约束**：11/29位ID，RTR位标识数据帧/远程帧，显性电平优先
- **DSL描述**：
```
field: can_arbitration_field, 
type: Bit(11)/Bit(29), 
length: 11/29bit, 
scope: layer(can_arbitration), 
constraint: enum(RTR=0=data_frame, RTR=1=remote_frame), 
desc: "CAN总线仲裁，显性电平优先"
```

#### 7.1.2 仲裁退避单元
- **功能**：仲裁失败后等待总线空闲并重新发送
- **字段约束**：8比特，范围0-10次
- **DSL描述**：
```
field: can_arbitration_backoff, 
type: Uint8, 
length: 1byte, 
scope: layer(can_control), 
constraint: range(0..=10), 
desc: "仲裁失败退避次数，最大10次"
```

### 7.2 帧控制与错误处理类单元

#### 7.2.1 DLC字段单元
- **功能**：数据长度码，标识数据字段长度
- **字段约束**：4比特，0~15，DLC=9~15对应8字节（传统CAN）或64字节（CAN FD）
- **DSL描述**：
```
field: can_dlc, 
type: Bit(4), 
length: 4bit, 
scope: layer(can_control), 
constraint: enum(0=0B,1=1B,..,8=8B,9=8B/64B), 
desc: "CAN数据长度码，支持传统CAN和CAN FD"
```

#### 7.2.2 位填充单元
- **功能**：连续5个相同位后插入相反位，用于接收端同步
- **字段约束**：1比特，触发条件为连续5个相同位
- **DSL描述**：
```
field: can_bit_stuffing, 
type: Bit(1), 
length: 1bit, 
scope: layer(can_physical), 
constraint: trigger=5_same_bits, 
desc: "CAN总线同步，填充位自动插入/删除"
```

#### 7.2.3 ACK应答单元
- **功能**：接收端确认机制
- **字段约束**：2比特，ACK位（发送端1→接收端0覆盖）+1位界定符
- **DSL描述**：
```
field: can_ack, 
type: Bit(2), 
length: 2bit, 
scope: layer(can_frame_tail), 
constraint: ack_bit=0(valid), ack_delimiter=1, 
desc: "CAN帧接收确认"
```

## 8. 语法单元作用范围分类

### 8.1 层内局部单元（单一层+部分字段）
- **作用范围**：单一层内的局部范围
- **覆盖范围**：单一层内的部分字段
- **示例**：帧序号（作用于当前帧）

### 8.2 层内全量单元（单一层+全量数据）
- **作用范围**：单一层内的全量数据
- **覆盖范围**：单一层的完整数据（可能排除自身）
- **示例**：链路层CRC-16（覆盖帧控制到用户数据，不含自身）

### 8.3 跨层穿透单元（相邻层+层间数据）
- **作用范围**：相邻层之间的数据传递
- **覆盖范围**：跨层的数据范围
- **示例**：SDU长度指示（从应用层到网络层）

### 8.4 全链路全局单元（全链路+端到端数据）
- **作用范围**：全链路的端到端数据
- **覆盖范围**：端到端的完整数据流
- **示例**：端到端CRC（覆盖整个传输过程的完整数据）

## 9. 算法类语法单元

### 9.1 DSL算法描述结构
```
field: <field_name>,
type: <data_type>,
length: <size>,
scope: <scope_type>,
cover: <data_coverage>,
alg: {
    type: [算法类型，如 crc16/crc32/custom/xor_sum], // 必选
    params: {[参数键值对，如 poly=0x1021, init=0xFFFF]}, // 可选
    data_range: [引用 cover 字段，如$cover], // 必选
    logic: [自定义逻辑，类 Rust 伪代码] // 仅 custom 类型需填
},
desc: <description>
```

### 9.2 标准算法实例 - CRC-16
```
field: link_crc16,
type: Uint16,
length: 2byte,
scope: layer(link),
cover: frame_ctrl..user_data,
exclude: self,
alg: {
    type: crc16,
    params: {poly:0x1021, init:0xFFFF, ref_in:false, ref_out:false, xor_out:0x0000},
    data_range: $cover
},
desc: "CCSDS 132.0-B-2 4.2.4 链路层 CRC-16"
```

## 10. 语法单元的可扩展性分析

### 10.1 标准扩展机制
- **Secondary Header**：允许在标准头部基础上扩展自定义字段
- **用户数据区域**：可根据任务需求自定义数据格式
- **虚拟信道机制**：支持不同类型数据的并行传输

### 10.2 自定义语法单元设计原则
1. **兼容性**：自定义单元需与现有CCSDS标准兼容
2. **可扩展性**：预留扩展字段和接口
3. **互操作性**：确保与其他系统的数据交换能力
4. **标准化**：遵循CCSDS的命名和结构规范

## 11. 多协议融合支持

### 11.1 CCSDS与CAN协议融合
- 支持CCSDS协议与CAN协议的混合使用
- 提供统一的语法单元接口，支持不同协议的语法单元
- 支持跨协议的数据转换和桥接功能

### 11.2 协议桥接示例
- 将CCSDS协议数据封装为CAN帧进行传输
- 在地面站或航天器上实现协议转换
- 支持不同协议间的时序同步

## 12. 语法单元在项目中的应用

### 12.1 标准语法单元实现
- 将各层协议实现为独立的语法单元（ProtocolUnit）
- 每个单元实现统一的接口（trait）
- 支持单元的动态组合和嵌套

### 12.2 自定义语法单元设计
- 基于标准语法单元进行扩展
- 支持特定任务需求的定制化协议
- 保持与标准单元的互操作性

### 12.3 语法单元组合策略
- **垂直组合**：上下层协议的封装关系（如Space Packet封装在TM帧中）
- **水平组合**：同层协议的复用关系（如多路虚拟信道）
- **跨协议组合**：不同协议间的桥接和转换（如CCSDS与CAN协议桥接）
- **混合组合**：复杂的协议栈结构

## 13. 项目需求合理性分析

### 13.1 技术可行性
- CCSDS协议结构清晰，适合模块化实现
- 语法单元边界明确，便于接口定义
- 标准成熟稳定，文档齐全

### 13.2 功能完整性
- 涵盖航天通信的主要协议层
- 支持标准和自定义协议的混合使用
- 提供完整的仿真验证能力

### 13.3 扩展性保障
- 基于Trait的接口设计保证扩展性
- 语法单元分离设计支持协议定制
- DSL描述语言支持复杂协议组合

### 13.4 多协议支持
- 支持CCSDS与CAN协议的混合使用
- 提供统一的语法单元接口
- 实现跨协议的数据转换和桥接

### 13.5 帧结构深化设计
- 支持帧数据区嵌套单元（导头指针）
- 支持帧序列管理单元（标志+序列号）
- 支持分路单元（数据分发）

### 13.6 DSL规范与JSON协议描述
- 支持语法单元的DSL描述
- 支持协议的JSON格式描述
- 提供可直接落地的协议描述

### 13.7 Rust开发适配
- 支持DSL解析器接口定义
- 支持协议组装平台接口定义
- 选择合适的Rust依赖库

## 14. 关键设计决策

### 14.1 语法单元划分
- 按功能维度划分基础语法单元
- 每个单元负责单一协议功能
- 通过组合实现复杂协议栈

### 14.2 作用范围定义
- 明确定义每个语法单元的作用范围
- 支持层内局部、层内全量、跨层穿透、全链路全局四种范围
- 保证数据处理的精确性

### 14.3 接口标准化
- 定义统一的语法单元接口
- 规范打包/拆包、校验等核心方法
- 确保单元间的互操作性

### 14.4 配置灵活性
- 通过DSL支持协议参数配置
- 允许运行时协议栈重组
- 提供可视化配置界面

### 14.5 多协议扩展
- 支持不同协议栈的扩展
- 提供统一的接口抽象
- 实现协议间的桥接机制

### 14.6 帧结构深化
- 支持帧数据区嵌套和指针定位
- 实现帧序列管理和分路功能
- 提供精细化的数据处理能力

### 14.7 DSL与JSON支持
- 支持语法单元的DSL描述
- 支持协议的JSON格式描述
- 提供可直接落地的协议描述

### 14.8 Rust开发适配
- 定义DSL引擎接口
- 定义协议组装平台接口
- 选择合适的依赖库

通过以上分析，我们可以确保项目在需求层面的合理性，为后续的架构设计和实现提供坚实的基础。