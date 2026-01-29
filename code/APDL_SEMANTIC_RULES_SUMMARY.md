# APDL 语义规则实现总结

## 项目概述

APDL（APDS Protocol Definition Language）项目是一个面向航天领域的协议定义与仿真验证一体化平台。项目已完成从帧级协议单元到字段级语法单元的架构重构，实现了完整的协议定义语言和语义规则处理系统。

## 已实现的语义规则类型

项目共实现了22种不同的语义规则类型，涵盖航天通信协议的各个方面：

### 1. 基础语义规则
- **ChecksumRange**: 校验范围规则，定义校验和算法覆盖的数据范围
- **Dependency**: 依赖规则，定义字段间的依赖关系
- **Conditional**: 条件规则，根据条件决定字段处理方式
- **Order**: 顺序规则，确保字段的排列顺序
- **Pointer**: 指针规则，定义字段指向关系
- **Algorithm**: 算法规则，指定字段处理算法
- **LengthRule**: 长度规则，定义字段长度计算公式

### 2. CCSDS协议特有规则
- **RoutingDispatch**: 路由分发规则，根据虚拟信道和APID进行分路
- **SequenceControl**: 序列控制规则，根据APID设置包计数
- **Validation**: 验证规则，对指定范围进行校验
- **Synchronization**: 同步规则，根据同步标志进行同步
- **LengthValidation**: 长度验证规则，验证字段长度

### 3. CAN协议特有规则
- **Multiplexing**: 多路复用规则，根据条件进行数据路由
- **PriorityProcessing**: 优先级处理规则，根据优先级进行仲裁
- **StateMachine**: 状态机规则，控制协议状态转换
- **PeriodicTransmission**: 周期传输规则，定时发送数据
- **MessageFiltering**: 消息过滤规则，过滤特定消息
- **ErrorDetection**: 错误检测规则，检测协议错误

### 4. 通用语义规则
- **FlowControl**: 流量控制规则，调节数据传输速率
- **TimeSynchronization**: 时间同步规则，保持时钟同步
- **AddressResolution**: 地址解析规则，解析设备地址
- **Security**: 安全规则，提供数据保护
- **Redundancy**: 冗余规则，提供备份路径

## DSL语法示例

### 字段定义语法
```apdl
field: sync_flag; type: Uint16; length: 2byte; scope: layer(physical); cover: entire_field; constraint: fixed(0xEB90); desc: "同步标志";
```

### 语义规则定义语法
```apdl
// 校验范围规则
rule: checksum_range(start: sync_flag to data_field);

// 路由分发规则
rule: routing_dispatch(field: sync_flag; algorithm: hash_sync_to_route; desc: "根据同步标志进行分路");

// 序列控制规则
rule: sequence_control(field: data_field; trigger: on_change; algorithm: seq_counter; desc: "序列控制");

// 长度规则
rule: length_rule(field: length_field equals "(total_length - 3)");
```

## 协议支持

### CCSDS协议支持
- 同步标志字段（0xEB90）
- 虚拟信道和APID分路
- 序列计数控制
- CRC校验验证
- 帧长度计算

### CAN协议支持
- 29位扩展标识符
- J1939和CANopen标准
- 优先级仲裁
- PGN路由
- 错误检测

## 实现特性

### 1. DSL解析器
- 使用简单字符串处理替代复杂的nom库解析器
- 支持十六进制数值解析
- 正确处理嵌套括号表达式
- 支持注释和复杂语义规则

### 2. 协议资源库
- 标准资源：`apdl-resources/standard/`
- 自定义资源：支持用户自定义协议定义
- 分类管理：CCSDS和CAN协议分别定义

### 3. 语义规则处理
- 自动识别和分类22种语义规则类型
- 不需要修改代码即可扩展新规则
- 支持复杂表达式计算

### 4. 错误处理
- 实现完整的Error trait
- 支持类型转换（From<String>）
- 提供详细的错误信息

## 项目优势

1. **模块化设计**：字段级语法单元设计，便于维护和扩展
2. **平台无关**：平台-语法单元分离架构
3. **语义丰富**：支持多种航天协议的复杂语义规则
4. **易扩展性**：通过DSL定义新协议，无需修改核心代码
5. **标准化**：遵循CCSDS、CAN等国际标准

## 应用场景

- 航天器通信协议仿真
- 卫星数据传输验证
- 航天任务规划与仿真
- 通信协议合规性验证
- 故障注入与恢复测试

## 总结

APDL项目成功实现了面向航天领域的协议定义与仿真验证一体化平台的核心功能，支持CCSDS和CAN等航天领域常用协议，具备丰富的语义规则处理能力。平台具有良好的扩展性，可通过DSL定义新的协议和语义规则，满足航天通信协议的多样化需求。