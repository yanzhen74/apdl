# APDL 重构后架构总结

## 重构背景

根据用户需求，我们将APDL（APDS Protocol Definition Language）的实现从帧级协议单元重构为字段级语法单元，使DSL能够描述完整的航天协议帧、包以及帧包之间的嵌套关系，由程序根据DSL自动进行仿真组帧。

## 重构前问题

- 实现了特定的CCSDS协议单元（如ccsds_ap.rs、ccsds_dl.rs等）
- 语法单元是帧级别的，而非字段级别
- 无法通过DSL动态定义协议结构

## 重构后架构

### 1. 字段级语法单元 (FieldUnit)

```rust
pub struct FieldUnit {
    pub meta: UnitMeta,  // 公开访问
    params: HashMap<String, String>,
    field_value: Vec<u8>, // 字段的实际值
    field_constraints: Vec<Constraint>, // 字段约束
}
```

- 最小粒度的语法单元，代表协议中的单个字段
- 支持多种字段类型（Uint8, Uint16, Uint32, Uint64, Bit, Bytes, Variable等）
- 内置约束验证功能（范围、固定值、枚举等）

### 2. 帧组装器 (FrameAssembler)

```rust
pub struct FrameAssembler {
    meta: UnitMeta,
    params: HashMap<String, String>,
    fields: Vec<FieldUnit>,
    protocol_structure: Vec<String>, // 字段组装顺序
}
```

- 根据DSL定义动态组装协议帧
- 支持添加多个字段单元
- 提供组装和解析功能

### 3. DSL解析器增强

- 支持解析单个字段定义
- 支持解析多个字段定义（协议结构）
- 支持多种类型、长度、约束和作用域定义

## DSL语法示例

```dsl
field: sync_flag; type: Uint16; length: 2byte; scope: layer(physical); cover: entire_field; constraint: fixed(0xEB90); desc: "同步标志 0xEB90";
field: version; type: Uint8; length: 1byte; scope: layer(data_link); cover: entire_field; constraint: range(0..=7); desc: "版本号";
field: data; type: RawData; length: dynamic; scope: layer(application); cover: entire_field; desc: "数据域"
```

## 核心功能

### 1. 字段级操作
- 字段创建与配置
- 约束验证
- 打包/解包操作

### 2. 动态协议组装
- 根据DSL定义动态构建协议结构
- 支持字段值设置和获取
- 完整帧组装和解析

### 3. 协议仿真能力
- 无需硬编码特定协议实现
- 通过DSL描述即可实现协议处理
- 支持协议结构的灵活定义

## 优势

1. **灵活性**：通过DSL定义任意协议结构，无需修改代码
2. **可扩展性**：支持新协议只需编写相应的DSL定义
3. **一致性**：统一的字段级处理机制
4. **可验证性**：内置约束验证功能

## 文件结构变化

```
apdl-poem/
├── standard_units/
│   ├── field_unit.rs      # 字段级语法单元
│   ├── frame_assembler.rs # 帧组装器
│   └── mod.rs            # 模块定义
├── dsl/
│   └── parser.rs         # DSL解析器
└── protocol_unit/
    └── mod.rs            # 协议单元管理
```

## 示例输出

运行示例程序 `cargo run --example ccsds_tm_demo` 显示：

```
APDL - CCSDS TM 帧构建演示
==========================
1. DSL定义的CCSDS TM帧组装:
   成功解析 7 个字段
   帧组装器已配置，包含字段: ["sync_flag", "version", "sc_id", "vc_id", "ocff_flag", "frame_len", "data"]
   组装完成，帧大小: 13 字节
   帧内容: [EB, 90, 00, 00, 01, 05, 00, 00, 07, DE, AD, BE, EF]
   组装器验证通过
   帧解析成功
   同步标志: 0xEB90
```

## 总结

重构后的架构完全符合用户需求：
- ✅ 最小语法单元为字段（field）级别
- ✅ DSL描述完整的航天协议帧结构
- ✅ 程序根据DSL自动进行仿真组帧
- ✅ 不需要特定的协议实现代码（如ccsds_ap.rs等）
- ✅ 支持协议帧、包以及嵌套关系的描述