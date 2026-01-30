# APDL DSL语法规范

## 文档修订历史
- 版本: 1.0
- 日期: 2026年1月29日
- 描述: 当前APDL DSL语法规范定义

## 1. 语法概述

APDL（APDS Protocol Definition Language）是一种领域特定语言，用于定义航天通信协议的语法单元和语义规则。语法设计遵循MVP原则，支持字段级协议定义和丰富的语义规则。

## 2. 字段定义语法

### 2.1 基本字段定义
```
field: field_name; type: type_spec; length: length_spec; scope: scope_spec; cover: cover_spec; [optional_params]; desc: "description";
```

### 2.2 参数说明
- `field_name`: 字段名称，遵循标识符命名规范
- `type_spec`: 类型规范，如 `Uint8`, `Uint16`, `Uint32`, `RawData` 等
- `length_spec`: 长度规范，如 `1byte`, `2byte`, `dynamic` 等
- `scope_spec`: 作用域规范，如 `layer(physical)`, `layer(data_link)` 等
- `cover_spec`: 覆盖范围，如 `entire_field`
- `optional_params`: 可选参数，如 `constraint`, `alg` 等
- `description`: 字段描述

### 2.3 语法示例
```
// CCSDS同步标志字段
field: sync_flag; type: Uint16; length: 2byte; scope: layer(physical); cover: entire_field; constraint: fixed(0xEB90); desc: "CCSDS同步标志";

// 动态长度数据字段
field: data_field; type: RawData; length: dynamic; scope: layer(application); cover: entire_field; desc: "数据域";

// CRC校验字段
field: checksum; type: Uint16; length: 2byte; scope: layer(data_link); cover: entire_field; alg: crc16; desc: "帧错误控制字段";
```

## 3. 约束条件语法

### 3.1 固定值约束
```
constraint: fixed(value)
```
- `value`: 固定值，支持十进制和十六进制（如0xEB90）

### 3.2 范围约束
```
constraint: range(start..=end)
```
- `start`, `end`: 起始和结束值

### 3.3 枚举约束
```
constraint: enum(enum1=value1, enum2=value2, ...)
```

### 3.4 约束示例
```
// 固定值约束
constraint: fixed(0xEB90)
constraint: fixed(123)

// 范围约束
constraint: range(0..=255)
constraint: range(0x00..=0xFF)

// 枚举约束
constraint: enum(IDLE=0, ACTIVE=1, STANDBY=2)
```

## 4. 语义规则语法

### 4.1 校验范围规则

有两种校验范围规则，用于定义不同类型的校验算法：

#### 4.1.1 CRC校验范围规则
```rust
rule: crc_range(start: start_field to end_field);
```
- 使用CRC算法（如CRC16、CRC32、CRC15）进行校验
- 适用于需要标准CRC校验的场景

#### 4.1.2 校验和范围规则
```rust
rule: checksum_range(start: start_field to end_field);
```
- 使用XOR算法进行校验和计算
- 适用于简单的校验和计算场景

**注意**: 为确保算法名称与实现一致，请根据所需算法选择合适的规则类型。如果字段声明为`alg: Crc16`，建议使用`crc_range`规则；如果字段声明为`alg: XorSum`，建议使用`checksum_range`规则。


### 4.2 依赖规则
```
rule: dependency(field: dependent_field depends_on dependency_field);
```

### 4.3 顺序规则
```
rule: order(first: first_field before second_field);
```

### 4.4 指针规则
```
rule: pointer(field: pointer_field points_to target_field);
```

### 4.5 长度规则
```
rule: length_rule(field: field_name equals "expression");
```

### 4.6 路由分发规则
```
rule: routing_dispatch(field: field_name; algorithm: algorithm_name; desc: "description");
```

### 4.7 序列控制规则
```
rule: sequence_control(field: field_name; trigger: condition; algorithm: algorithm_name; desc: "description");
```

### 4.8 验证规则
```
rule: validation(field: field_name; algorithm: algorithm_name; range: from(start_field) to(end_field); desc: "description");
```

### 4.9 同步规则
```
rule: synchronization(field: field_name; algorithm: algorithm_name; desc: "description");
```

### 4.10 长度验证规则
```
rule: length_validation(field: field_name; condition: condition_desc; desc: "description");
```

### 4.11 多路复用规则
```
rule: multiplexing(field: field_name; condition: condition_desc; route: route_target; desc: "description");
```

### 4.12 优先级处理规则
```
rule: priority_processing(field: field_name; algorithm: algorithm_name; desc: "description");
```

### 4.13 状态机规则
```
rule: state_machine(condition: condition_desc; algorithm: algorithm_name; desc: "description");
```

### 4.14 周期传输规则
```
rule: periodic_transmission(field: field_name; condition: condition_desc; algorithm: algorithm_name; desc: "description");
```

### 4.15 消息过滤规则
```
rule: message_filtering(condition: condition_desc; action: action_desc; desc: "description");
```

### 4.16 错误检测规则
```
rule: error_detection(algorithm: algorithm_name; desc: "description");
```

### 4.17 流量控制规则
```
rule: flow_control(field: field_name; algorithm: algorithm_name; desc: "description");
```

### 4.18 时间同步规则
```
rule: time_synchronization(field: field_name; algorithm: algorithm_name; desc: "description");
```

### 4.19 地址解析规则
```
rule: address_resolution(field: field_name; algorithm: algorithm_name; desc: "description");
```

### 4.20 安全规则
```
rule: security(field: field_name; algorithm: algorithm_name; desc: "description");
```

### 4.21 冗余规则
```
rule: redundancy(field: field_name; algorithm: algorithm_name; desc: "description");
```

## 5. 作用域规范

### 5.1 单层作用域
```
scope: layer(layer_name)
```
支持的层类型：
- `physical`: 物理层
- `data_link`: 数据链路层
- `network`: 网络层
- `transport`: 传输层
- `application`: 应用层

### 5.2 跨层作用域
```
scope: cross_layer(layer1→layer2)
```

### 5.3 全局作用域
```
scope: global(scope_name)
```

## 6. 覆盖范围规范

### 6.1 整个字段
```
cover: entire_field
```

### 6.2 范围表达式
```
cover: field_name[offset..length]
```

### 6.3 表达式覆盖
```
cover: expression_desc
```

## 7. 长度规范

### 7.1 固定字节长度
```
length: Nbyte  // N为正整数
```

### 7.2 固定比特长度
```
length: Nbit   // N为正整数
```

### 7.3 动态长度
```
length: dynamic
```

### 7.4 表达式长度
```
length: (expression)
```

## 8. 算法规范

### 8.1 支持的校验算法
- `crc16`: CRC16校验
- `crc32`: CRC32校验
- `crc15`: CRC15校验（CAN协议专用）
- `xor_sum`: XOR校验

### 8.2 自定义算法
```
alg: custom_algorithm_name
```

## 9. 注释语法

使用 `//` 开头的行会被视为注释：
```
// 这是一条注释
field: my_field; type: Uint16; length: 2byte; scope: layer(physical); cover: entire_field; desc: "我的字段";
```

## 10. 完整示例

```
// CCSDS传输帧定义
// 同步标志字段
field: sync_flag; type: Uint16; length: 2byte; scope: layer(physical); cover: entire_field; constraint: fixed(0xEB90); desc: "同步标志";

// 虚拟信道ID
field: vcid; type: Uint8; length: 1byte; scope: layer(data_link); cover: entire_field; constraint: range(0..=63); desc: "虚拟信道ID";

// 数据字段
field: data_field; type: RawData; length: dynamic; scope: layer(application); cover: entire_field; desc: "数据域";

// 帧错误控制字段
field: fecf; type: Uint16; length: 2byte; scope: layer(data_link); cover: entire_field; alg: crc16; desc: "帧错误控制字段";

// 语义规则定义
rule: routing_dispatch(field: vcid; algorithm: hash_vcid_to_route; desc: "根据虚拟信道进行分路");
rule: checksum_range(start: sync_flag to data_field); // CRC校验范围
rule: synchronization(field: sync_flag; algorithm: sync_pattern_match; desc: "同步标志识别");
rule: length_rule(field: data_field; expression: "(total_frame_size - 3)"); // 长度计算
```

## 11. 语法验证

DSL解析器会对以下方面进行验证：
- 语法正确性
- 字段引用有效性
- 表达式合法性
- 类型兼容性

## 12. 扩展性考虑

当前语法设计支持：
- 新字段类型的添加
- 新语义规则的扩展
- 新约束条件的定义
- 新算法的集成