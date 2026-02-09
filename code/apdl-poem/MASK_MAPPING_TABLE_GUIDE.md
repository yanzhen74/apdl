# 掩码映射表功能说明

## 概述

掩码映射表（Mask Mapping Table）提供了一种基于位掩码的字段映射机制，允许将源字段值应用掩码后查表映射到目标值。

## 使用场景

适用于需要根据字段的部分位进行映射的场景，如：
- APID (Application Process ID) 到 VCID (Virtual Channel ID) 的映射
- 协议类型字段的高位到分组通道的映射
- 任何需要忽略字段某些位进行分类映射的场景

## DSL语法

### 基本语法

```apdl
rule: field_mapping(
    source_package: "源包名";
    target_package: "目标包名";
    mappings: [{
        source_field: "源字段名",
        target_field: "目标字段名",
        mapping_logic: "mask_table",
        default_value: "默认值",
        mask_mapping_table: [
            {mask: [掩码字节数组], src_masked: [掩码后期望值], dst: [目标映射值]},
            ...
        ]
    }];
    desc: "描述信息"
)
```

### 完整示例

```apdl
rule: field_mapping(
    source_package: "mpdu_packet";
    target_package: "tm_frame";
    mappings: [{
        source_field: "apid",
        target_field: "vcid",
        mapping_logic: "mask_table",
        default_value: "0x00",
        mask_mapping_table: [
            {mask: [0xFF, 0xF0], src_masked: [0x04, 0x80], dst: [0x35]},
            {mask: [0xFF, 0xF0], src_masked: [0x04, 0x90], dst: [0x36]},
            {mask: [0xFF, 0xFF], src_masked: [0x05, 0x00], dst: [0x37]}
        ]
    }];
    desc: "APID到VCID的掩码映射"
)
```

## 映射逻辑说明

### 工作原理

1. **读取源字段值**：从源包中获取指定字段的值
2. **遍历映射表**：按顺序检查每个映射条目
3. **应用掩码**：将源值与掩码进行按位与操作
4. **匹配判断**：将掩码后的结果与期望值比较
5. **返回目标值**：匹配成功则返回对应的目标值
6. **默认值处理**：所有条目都不匹配时使用默认值

### 示例说明

假设源字段 `apid = [0x04, 0x81]`：

**条目1检查**：
- 掩码：`[0xFF, 0xF0]`
- 掩码操作：`[0x04, 0x81] & [0xFF, 0xF0] = [0x04, 0x80]`
- 期望值：`[0x04, 0x80]`
- **匹配成功** ✓
- 返回目标值：`[0x35]`

如果 `apid = [0x04, 0x92]`：
- 条目1：`[0x04, 0x92] & [0xFF, 0xF0] = [0x04, 0x90]` ≠ `[0x04, 0x80]` ✗
- 条目2：`[0x04, 0x92] & [0xFF, 0xF0] = [0x04, 0x90]` = `[0x04, 0x90]` ✓
- 返回目标值：`[0x36]`

## 数据格式支持

### 十六进制格式（推荐）

```apdl
mask_mapping_table: [
    {mask: [0xFF, 0xF0], src_masked: [0x04, 0x80], dst: [0x35]}
]
```

### 十进制格式

```apdl
mask_mapping_table: [
    {mask: [255, 240], src_masked: [4, 128], dst: [53]}
]
```

两种格式可以混用，但建议在同一项目中保持一致。

## 数据结构

### Rust 结构定义

```rust
/// 掩码映射表条目
pub struct MaskMappingEntry {
    pub mask: Vec<u8>,       // 掩码，如 [0xFF, 0xF0]
    pub src_masked: Vec<u8>, // 源值应用掩码后的期望值，如 [0x04, 0x80]
    pub dst: Vec<u8>,        // 目标映射值，如 [0x35]
}

/// 字段映射条目
pub struct FieldMappingEntry {
    pub source_field: String,
    pub target_field: String,
    pub mapping_logic: String,                         // "mask_table"
    pub default_value: String,                         
    pub enum_mappings: Option<Vec<EnumMappingEntry>>,  
    pub mask_mapping_table: Option<Vec<MaskMappingEntry>>, // 掩码映射表
}
```

## 注意事项

1. **长度一致性**：掩码、期望值和源值的字节长度必须一致
2. **顺序匹配**：映射表按顺序匹配，第一个匹配的条目生效
3. **默认值**：建议始终提供合理的默认值
4. **掩码设计**：合理设计掩码以避免歧义

## 性能考虑

- 映射表查找是线性搜索，时间复杂度 O(n)
- 建议将常用映射条目放在前面
- 映射表条目数量建议不超过100个
- 对于大量映射关系，考虑使用哈希映射

## 相关功能

- `identity`：恒等映射（直接传递）
- `hash_mod_64`：哈希取模映射
- `enum_mappings`：枚举值映射（支持通配符）

## 测试验证

参见测试文件：`apdl-poem/tests/mask_mapping_table_test.rs`

```rust
// 运行测试
cargo test --package apdl-poem --test mask_mapping_table_test
```
