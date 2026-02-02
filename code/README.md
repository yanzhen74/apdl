# APDL (APDS Protocol Definition Language) - MVP版本

## 项目概述

APDL (APDS Protocol Definition Language)是一个面向航天领域的协议定义与仿真验证一体化平台。本项目旨在提供一套标准化的协议定义、仿真验证和性能分析工具，满足航天领域复杂协议开发的需求。

## MVP阶段成果

在MVP（最小可行产品）阶段，我们成功实现了以下核心功能：

### 1. 语义规则处理器系统

- **23种语义规则处理器**: 实现了包括校验和、依赖关系、控制规则、流控、冗余、过滤、多路复用等在内的完整规则处理器系统
- **TODO标记系统**: 在所有规则处理器中添加了 TODO 注释，标记需要在实际应用中实现的功能点
- **智能规则解析**: 支持多种算法和策略的规则执行

### 2. CCSDS协议标准语法单元实现

- **CCSDS TM (Telemetry) Transfer Frame** (`ccsds_dl`): 实现了CCSDS 132.0-B-3 TM传输帧标准
- **CCSDS TC (Telecommand) Transfer Frame** (`ccsds_tc`): 实现了CCSDS 232.0-B-3 TC传输帧标准  
- **CCSDS Application Protocol** (`ccsds_ap`): 实现了应用层协议单元框架
- **CCSDS Transport Protocol** (`ccsds_tp`): 实现了传输层协议单元框架

每个语法单元都实现了统一的`ProtocolUnit`接口，支持打包、解包、验证等功能。

### 3. DSL (Domain Specific Language) 设计与解析

- 实现了APDL DSL语法解析器，基于nom库
- 支持协议字段定义、类型声明、长度描述、作用域定义等功能
- 支持约束条件、算法描述、关联关系等高级特性
- 提供了完整的DSL解析和验证功能

### 4. 核心架构设计

- **协议单元接口** (`ProtocolUnit`): 统一的协议单元接口，实现平台与语法单元的分离
- **模块化解耦**: 采用清晰的模块划分，便于扩展和维护
- **错误处理机制**: 完善的错误处理体系，包含协议错误、解析错误等

### 5. 工具函数库

- CRC校验算法实现
- 位操作工具
- 数据转换工具（十六进制/字节转换等）

## 核心组件

### apdl-core
核心库，定义了协议单元接口、元数据结构和基础工具函数。

### apdl-poem
协议对象与实体映射模块，实现了具体的协议语法单元、DSL解析器和语义规则处理器系统。

## 使用示例

```rust
use apdl_poem::standard_units::{CcsdsDlUnit, CcsdsTcUnit};
use apdl_core::ProtocolUnit;

// 创建CCSDS数据链路层单元
let config = CcsdsDlConfig::default();
let dl_unit = CcsdsDlUnit::new(config);

// 验证配置
assert!(dl_unit.validate().is_ok());

// 打包数据
let sdu = vec![0x01, 0x02, 0x03, 0x04];
let pdu = dl_unit.pack(&sdu).expect("Pack should succeed");

// 解包数据
let (unpacked_sdu, remaining) = dl_unit.unpack(&pdu).expect("Unpack should succeed");
assert_eq!(unpacked_sdu, sdu);
```

## 测试覆盖率

- 协议单元基本功能测试
- DSL解析器功能测试
- 语义规则验证测试
- 打包/解包循环测试
- 边界条件和错误处理测试

## 技术栈

- **编程语言**: Rust
- **解析器库**: nom 7.1
- **模块化架构**: 清晰的依赖关系和接口定义
- **错误处理**: 自定义错误类型系统

## 未来发展方向

MVP阶段为后续开发奠定了坚实基础，后续可以：

1. 扩展更多协议标准支持（如CAN、SpaceWire等）
2. 开发协议仿真验证引擎
3. 构建图形化协议定义工具
4. 实现性能分析和可视化功能
5. 集成形式化验证能力

## 项目结构

```
apdl/
├── apdl-core/          # 核心抽象和数据结构
├── apdl-poem/          # 协议对象与实体映射
│   ├── standard_units/ # 标准协议语法单元
│   ├── custom_units/   # 自定义协议语法单元
│   ├── dsl/           # DSL解析器实现
│   ├── standard_units/frame_assembler/ # 帧组装器及规则处理器
│   └── protocol_unit/ # 协议单元接口定义
```

## 编译与测试

```bash
# 构建项目
cargo build

# 运行测试
cargo test

# 构建特定包
cargo build -p apdl-core -p apdl-poem
```

---
*APDL MVP版本 - 航天协议定义与仿真验证平台*