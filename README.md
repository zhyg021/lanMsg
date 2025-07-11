# lanMsg
基于ipMsg的命令行内网聊天软件

# LAN Messenger (基于ipMsg协议的Rust实现)

![Rust版本](https://img.shields.io/badge/Rust-1.70%2B-orange?logo=rust)
![许可证](https://img.shields.io/badge/许可证-MIT-blue.svg)

一个基于ipMsg协议的轻量级局域网通讯工具（Rust实现）

## 目录
- [功能特性](#功能特性)
- [项目结构](#项目结构)
- [安装指南](#安装指南)
- [使用说明](#使用说明)
- [配置说明](#配置说明)
- [开发进度](#开发进度)
- [参与贡献](#参与贡献)
- [许可证](#许可证)

## 功能特性

### 已实现
- ✅ 局域网用户发现与在线列表
- ✅ 基础文本消息接口
- ✅ 可配置用户信息
- ✅ 基于Rust的高效实现

### 开发中
- 🚧 文本消息优化（不稳定）

### 计划中
- ◻️ 文件传输功能
- ◻️ 消息确认机制
- ◻️ 界面交互优化

## 项目结构

```text
lanMsg/
├── src/
│   ├── main.rs          # 程序主入口
│   ├── cli.rs           # 命令行解析
│   ├── config.rs        # 配置管理
│   ├── net.rs           # 网络通信
│   └── protocol.rs      # 协议处理
├── config.toml          # 配置文件模板
├── Cargo.toml           # 项目配置
└── README.md            # 本文档
```

## 使用说明
1. 首先修改配置文件：\
    nano config.toml
2. 启动程序：\
./target/release/lanMsg
3. 可用命令：

```text    
list        显示在线用户（默认自动显示） 
send        <用户> <消息>  发送文本消息    
help        显示帮助信息 
exit        退出程序 
```
4. 运行
```text
lanMsg --name Alice --host PC-1 list
lanMsg --name Alice --host PC-1 send bob hello
lanMsg --name Alice --host PC-1 send 127.0.0.1 hello
```
## 许可证
本项目采用 MIT 许可证 - 详见 LICENSE 文件。
