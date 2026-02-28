# APDL Protocol Editor

APDL协议编辑器 - 基于Tauri + React + TypeScript的协议定义可视化编辑工具。

## 功能特性

- **协议定义编辑**: 使用Monaco编辑器编辑JSON格式的协议定义
- **实时验证**: 基于JSON Schema验证协议定义的正确性
- **示例加载**: 内置CCSDS协议示例
- **分层显示**: 支持协议层级结构可视化（开发中）

## 技术栈

- **后端**: Rust + Tauri
- **前端**: React + TypeScript + Vite
- **编辑器**: Monaco Editor
- **验证**: JSON Schema (Draft-07)

## 开发命令

```bash
# 安装依赖
npm install

# 开发模式
npm run tauri dev

# 构建
npm run tauri build
```

## 项目结构

```
src-tauri/
  src/
    lib.rs          # Rust后端命令
  schema/
    apdl-protocol-schema-v1.json  # 协议定义Schema
    examples/                      # 示例协议定义
src/
  App.tsx         # React主组件
```

## 支持的协议定义

- 语法单元 (SyntaxUnit)
- 字段定义 (FieldDefinition)
- 打包/拆包规范 (PackUnpackSpec)
- 语义规则 (SemanticRule)
- 帧包关系 (FramePacketRelationship)
- 字段映射 (FieldMapping)
- 协议层级 (ProtocolHierarchy)
