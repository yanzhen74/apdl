# APDL 交互访问模块 (APDL IAM)

APDL 交互访问模块 (Interaction Access Module) 是 APDL (APDS Protocol Definition Language) 系统的一部分，提供用户交互和访问接口。

## 概述

APDL IAM 模块提供以下功能：

1. **API 接口** - 提供 REST API 接口用于外部系统集成
2. **命令行界面** - 提供命令行工具用于协议操作和管理
3. **图形用户界面** - 基于 egui/eframe 的可视化界面，便于用户交互

## 架构

```
apdl-iam/
├── src/
│   ├── api/           # REST API 实现
│   ├── cli/           # 命令行界面实现
│   ├── gui/           # 图形用户界面实现
│   ├── lib.rs         # 模块导出
│   └── main.rs        # GUI 应用入口
├── Cargo.toml         # 依赖配置
└── README.md          # 本文档
```

## 功能特性

- **跨平台 GUI** - 使用 egui/eframe 实现的原生跨平台图形界面
- **RESTful API** - 支持协议定义、解析和验证的 REST 接口
- **命令行工具** - 支持批处理和脚本化的命令行操作
- **实时预览** - 协议定义的实时语法检查和预览

## 快速开始

### 运行 GUI 应用

```bash
cargo run
```

### 构建项目

```bash
cargo build
```

## 依赖

- `egui` - 现代化、便携式即时模式GUI工具包
- `eframe` - egui的框架，支持Web和原生应用
- `apdl-core` - APDL核心库

## 开发

### 模块职责

- `api` - 处理REST API请求和响应
- `cli` - 处理命令行参数和控制台输出
- `gui` - 处理图形界面组件和用户交互

## 贡献

欢迎提交 Issue 和 Pull Request 来改进本项目。