# APDL语义规则系统设计说明

## 文档修订历史
- 版本: 1.0
- 日期: 2026年1月29日
- 描述: 反映当前APDL语义规则系统实现状态

## 1. 系统概述

APDL（APDS Protocol Definition Language）语义规则系统是一个面向航天领域的协议定义与仿真验证平台的核心组件。系统实现了字段级语法单元架构，支持CCSDS和CAN等航天领域常用协议的语义规则处理。

## 2. 核心架构

### 2.1 架构演进
- **原架构**: 帧级协议单元（如ccsds_ap.rs等特定协议实现）
- **现架构**: 字段级语法单元（语法单元与平台分离）

### 2.2 主要组件
1. **DSL解析器** (`DslParserImpl`): 解析协议定义语言
2. **语法单元** (`SyntaxUnit`): 描述协议字段
3. **语义规则** (`SemanticRule`): 定义协议处理逻辑
4. **帧组装器** (`FrameAssembler`): 实现协议组装和解析

## 3. 语义规则分类

### 3.1 数据流向控制类
- **RoutingDispatch**: 路由分发规则 - 根据字段值进行分路
- **Multiplexing**: 多路复用规则 - 根据条件路由数据

### 3.2 数据完整性保障类
- **ChecksumRange**: 校验范围规则 - 定义XOR校验和覆盖范围
- **CrcRange**: CRC校验范围规则 - 定义CRC校验算法覆盖范围
- **Validation**: 验证规则 - 对指定范围进行校验
- **ErrorDetection**: 错误检测规则 - 检测协议错误

### 3.3 数据流状态管理类
- **SequenceControl**: 序列控制规则 - 管理包序列计数
- **PeriodicTransmission**: 周期传输规则 - 定时发送数据
- **StateMachine**: 状态机规则 - 控制协议状态转换

### 3.4 数据结构映射类
- **Pointer**: 指针规则 - 定义字段指向关系
- **Order**: 顺序规则 - 确保字段排列顺序
- **LengthRule**: 长度规则 - 定义字段长度计算

### 3.5 业务逻辑处理类
- **Dependency**: 依赖规则 - 定义字段间依赖关系
- **Algorithm**: 算法规则 - 指定字段处理算法
- **Security**: 安全规则 - 提供数据保护

## 4. DSL语法规范

### 4.1 字段定义语法
```
field: field_name; type: Uint16; length: 2byte; scope: layer(physical); cover: entire_field; constraint: fixed(0xEB90); desc: "字段描述";
```

### 4.2 语义规则定义语法
```
// 校验范围规则
rule: checksum_range(start: field1 to field2);

// 路由分发规则
rule: routing_dispatch(field: field1; algorithm: algo_name; desc: "描述");

// 序列控制规则
rule: sequence_control(field: field1; trigger: condition; algorithm: algo_name; desc: "描述");

// 长度规则
rule: length_rule(field: field1 equals "expression");
```

## 5. 实现特点

### 5.1 DSL解析器优化
- 使用简单字符串处理替代复杂nom库解析器
- 支持十六进制数值解析
- 正确处理嵌套括号表达式
- 支持注释和复杂语义规则

### 5.2 语义规则处理
- 自动识别和分类22种语义规则类型
- 支持字段名前缀清理（如"start:", "end:", "field:"等）
- 统一的规则处理接口

### 5.3 协议资源库
- 标准资源: `apdl-resources/standard/`
- 支持CCSDS和CAN协议定义
- 分类管理不同协议标准

## 6. 支持的协议标准

### 6.1 CCSDS协议支持
- 同步标志字段（0xEB90）
- 虚拟信道和APID分路
- 序列计数控制
- CRC校验验证
- 帧长度计算

### 6.2 CAN协议支持
- 29位扩展标识符
- J1939和CANopen标准
- 优先级仲裁
- PGN路由
- 错误检测

## 7. 扩展性设计

### 7.1 现有扩展机制
- 通过DSL定义新协议，无需修改核心代码
- 语义规则类型可扩展（枚举结构）
- 统一的规则处理接口

### 7.2 未来扩展方向
- 规则元数据系统（执行优先级、依赖关系）
- 规则组合框架（复合规则定义）
- 规则模板机制（常见场景模板）

## 8. MVP实现状态

### 8.1 已完成功能
- [x] 字段级语法单元架构重构
- [x] 22种语义规则类型实现
- [x] DSL解析器重写
- [x] 帧组装器实现
- [x] CCSDS和CAN协议支持
- [x] 协议资源库组织
- [x] 完整测试验证

### 8.2 验证结果
- DSL解析器成功解析协议定义
- 语义规则正确应用
- 帧组装和解析功能正常
- 所有22种规则类型通过测试

## 9. 技术栈

- **编程语言**: Rust
- **核心组件**: DslParser, FrameAssembler, SyntaxUnit
- **协议标准**: CCSDS, CAN (J1939, CANopen)
- **架构模式**: 策略模式, 配置驱动

## 10. 应用场景

- 航天器通信协议仿真
- 卫星数据传输验证
- 航天任务规划与仿真
- 通信协议合规性验证
- 故障注入与恢复测试