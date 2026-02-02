# APDL 分层与连接器 DSL 设计文档

## 概述

本设计文档描述了 APDL (APDS Protocol Definition Language) 的分层与连接器 DSL (Domain Specific Language)。该 DSL 层建立在现有的基础 DSL 之上，用于描述多层协议结构以及各层之间的连接器。

## 设计原则

1. **分层平等建模原则**: 每层都是语义完整、无隶属关系的自治单元
2. **并列包结构**: 支持同一层内的并列（复接）包结构
3. **连接器机制**: 定义层间数据流转的连接器
4. **DSL 自治性**: 不依赖硬编码的帧实现文件，完全由 DSL 自动完成组帧逻辑

## DSL 语法设计

### 1. 层定义语法

```
layer <layer_name> {
    description: "层描述信息";
    protocol_type: <protocol_identifier>;
    units: [<unit_list>];
    connections: [<connection_list>];
}
```

### 2. 并列包定义语法

```
parallel <parallel_group_name> {
    description: "并列包组描述";
    packages: [<package_list>];
    multiplexing_algorithm: <algorithm_name>;
    priority: <priority_value>;
}
```

### 3. 连接器定义语法

```
connector <connector_name> {
    description: "连接器描述";
    source_layer: <source_layer_name>;
    target_layer: <target_layer_name>;
    mapping_rules: [
        field_map(<source_field>, <target_field>),
        algorithm_map(<source_algorithm>, <target_algorithm>)
    ];
    transmission_rules: [
        <transmission_rule_list>
    ];
}
```

### 4. 完整示例

```
// 定义物理层
layer physical_layer {
    description: "Physical Layer for Space Communications";
    protocol_type: "spacewire";
    units: [physical_frame_unit];
    connections: [phy_to_datalink_connector];
}

// 定义数据链路层
layer datalink_layer {
    description: "Data Link Layer";
    protocol_type: "ccsds_datalink";
    units: [tc_frame_unit, tm_frame_unit];
    connections: [datalink_to_network_connector];
}

// 定义网络层
layer network_layer {
    description: "Network Layer";
    protocol_type: "ccsds_network";
    units: [packet_unit];
    connections: [];
}

// 定义并列包组（例如，同时传输TM和TC帧）
parallel frame_mux_group {
    description: "Multiplex TM and TC frames";
    packages: [tm_frame_package, tc_frame_package];
    multiplexing_algorithm: "time_division";
    priority: 5;
}

// 定义连接器
connector phy_to_datalink_connector {
    description: "Connect Physical to DataLink Layer";
    source_layer: physical_layer;
    target_layer: datalink_layer;
    mapping_rules: [
        field_map("physical_header.crc", "datalink_header.crc"),
        algorithm_map("physical_encoding", "datalink_encoding")
    ];
    transmission_rules: [
        rule: dependency(field: seq_count depends_on apid),
        rule: checksum_range(start: sync_flag to data_field),
        rule: validation(field: apid algorithm: range_check range: "0x000-0x7FF")
    ];
}
```

## 技术实现架构

### 1. 解析器模块
- `layer_parser.rs` - 解析层定义
- `parallel_parser.rs` - 解析并列包定义
- `connector_parser.rs` - 解析连接器定义

### 2. 结构体定义
- `LayerDefinition` - 层定义结构
- `ParallelPackageGroup` - 并列包组结构
- `ConnectorDefinition` - 连接器定义结构
- `ConnectionRule` - 连接规则结构

### 3. 处理器模块
- `layer_processor.rs` - 处理层间的逻辑关系
- `multiplexer_processor.rs` - 处理并列包的复接逻辑
- `connector_processor.rs` - 处理连接器的数据流转

## 开发计划

### Phase 1: DSL 语法设计与解析器实现 (Week 1)
1. 完善 DSL 语法规范
2. 实现基础解析器
   - `layer_parser.rs`
   - `parallel_parser.rs`
   - `connector_parser.rs`
3. 编写语法验证测试

### Phase 2: 数据结构定义 (Week 1)
1. 定义核心数据结构
   - `LayerDefinition`
   - `ParallelPackageGroup`
   - `ConnectorDefinition`
2. 实现数据结构的序列化/反序列化

### Phase 3: 连接器逻辑实现 (Week 2)
1. 实现 `connector_processor.rs`
2. 实现层间数据流转机制
3. 实现字段映射逻辑
4. 实现算法映射逻辑

### Phase 4: 复接逻辑实现 (Week 2)
1. 实现 `multiplexer_processor.rs`
2. 实现并列包的复接算法
3. 实现优先级调度机制

### Phase 5: 集成与测试 (Week 3)
1. 实现完整的层处理链
2. 编写端到端测试
3. 性能优化
4. 文档完善

### Phase 6: 示例与演示 (Week 3)
1. 创建完整的 CCSDS 多层协议示例
2. 实现 CCSDS TM/TC 复接示例
3. 编写用户指南

## 预期功能

1. **多层协议支持**: 支持任意层级的协议定义
2. **并列包处理**: 支持同一层内多个包的并列传输
3. **灵活连接**: 支持层间的灵活连接和数据映射
4. **自动化组帧**: 根据 DSL 自动完成跨层组帧逻辑
5. **协议复用**: 支持协议层的复用和组合

## 与现有系统的集成

新的分层与连接器 DSL 将与现有基础 DSL 无缝集成，通过以下方式：
1. 重用现有的语义规则系统
2. 与现有的 Frame Assembler 集成
3. 保持与 ProtocolUnit 接口的兼容性
4. 利用现有的错误处理机制

## 扩展示例

```
// 复杂的多层协议栈示例
layer_stack ccsds_protocol_stack {
    layers: [
        physical_layer,
        datalink_layer,
        network_layer,
        transport_layer
    ];
    connections: [
        phy_to_datalink_connector,
        datalink_to_network_connector,
        network_to_transport_connector
    ];
    parallel_groups: [
        tm_tc_mux_group
    ];
}
```

该设计遵循了 APDL 的分层平等建模原则，确保每一层都是语义完整的自治单元，同时提供了灵活的连接器机制来处理层间数据流转。