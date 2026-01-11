# 灵视 Glimpse 项目结构分析

## 项目概述
灵视 Glimpse 是一个基于 Tauri 2.0 的桌面应用，提供屏幕二维码识别和 OCR 文字识别功能。

## 目录结构

```
Wscan/
├── src/                    # 前端源码
│   ├── index.html         # 主页面
│   ├── main.js            # 主逻辑
│   ├── styles.css         # 样式
│   └── assets/            # 静态资源
├── src-tauri/             # Rust 后端
│   ├── src/               # Rust 源码
│   │   ├── lib.rs         # 库入口
│   │   ├── main.rs        # 程序入口
│   │   ├── ocr.rs         # OCR 功能
│   │   ├── qr_scanner.rs  # 二维码扫描
│   │   ├── screenshot.rs  # 截图功能
│   │   ├── selector_window.rs # 区域选择
│   │   └── tray.rs        # 系统托盘
│   ├── icons/             # 应用图标
│   ├── models/            # OCR 模型文件
│   ├── Cargo.toml         # Rust 依赖
│   └── tauri.conf.json    # Tauri 配置
├── dist-portable/         # 便携版输出
├── build.bat              # 构建脚本
├── dev.bat                # 开发脚本
└── package.json           # Node 依赖
```

## 核心功能
1. 全屏二维码扫描
2. 区域二维码扫描
3. OCR 文字识别
4. 系统托盘支持
5. 全局快捷键

## 技术栈
- 前端: HTML/CSS/JavaScript
- 后端: Rust + Tauri 2.0
- OCR: PaddleOCR (ONNX)
- 二维码: rxing