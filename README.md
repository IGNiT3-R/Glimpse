# 🔍 灵视 Glimpse - Windows 屏幕内容识别工具

> 基于 Rust + Tauri 2.0 开发的轻量级桌面屏幕识别工具，支持二维码扫描和 OCR 文字识别

## ✨ 特性

- 🖥️ **全屏识别** - 快速识别整个屏幕上的所有二维码
- ✂️ **区域识别** - 精确框选区域进行二维码识别
- 📝 **文字识别** - OCR 识别屏幕上的文字内容
- ⚡ **极速响应** - 识别速度 < 300ms
- 💾 **轻量体积** - 单文件便携版
- 🔒 **纯本地运行** - 无网络请求，保护隐私
- 🌓 **深色模式** - 支持亮色/暗色主题切换
- 📋 **一键复制** - 识别结果轻松复制
- 🔧 **系统托盘** - 最小化到托盘后台运行

## 🎮 快捷键

| 快捷键 | 功能 |
|--------|------|
| `Ctrl + Shift + S` | 全屏识别二维码 |
| `Ctrl + Shift + A` | 区域识别二维码 |
| `Ctrl + Shift + D` | OCR 文字识别 |
| `ESC` | 取消区域选择 |

## 🚀 快速开始

### 开发环境要求

- Windows 10/11
- [Rust](https://rustup.rs/) (1.92+)
- [Node.js](https://nodejs.org/) (18+)
- [pnpm](https://pnpm.io/) (推荐)

### 安装依赖

```bash
pnpm install
```

### 开发模式

```bash
# 使用脚本
dev.bat

# 或使用命令
pnpm dev
```

### 构建发布版本

```bash
# 使用脚本
build.bat

# 或使用命令
pnpm build
```

构建产物位于 `dist-portable/灵视Glimpse.exe`

## 📁 项目结构

```
Glimpse/
├── src/                      # 前端代码
│   ├── index.html           # 主页面
│   ├── main.js              # 主逻辑
│   ├── styles.css           # 样式表
│   └── assets/              # 静态资源
├── src-tauri/               # 后端代码
│   ├── src/
│   │   ├── main.rs         # 程序入口
│   │   ├── lib.rs          # 主模块
│   │   ├── screenshot.rs   # 截图模块
│   │   ├── qr_scanner.rs   # 二维码识别
│   │   ├── ocr.rs          # OCR 文字识别
│   │   ├── selector_window.rs # 区域选择窗口
│   │   └── tray.rs         # 系统托盘
│   ├── models/             # OCR 模型文件
│   ├── icons/              # 应用图标
│   ├── Cargo.toml          # Rust 依赖
│   └── tauri.conf.json     # Tauri 配置
├── dist-portable/          # 便携版输出目录
├── build.bat               # 构建脚本
├── dev.bat                 # 开发脚本
└── README.md
```

## 🛠️ 技术栈

- **框架**: [Tauri 2.0](https://tauri.app/)
- **语言**: Rust + JavaScript
- **UI**: Vanilla JS + CSS
- **二维码**: [rxing](https://crates.io/crates/rxing)
- **OCR**: PaddleOCR v4 (ONNX Runtime)
- **截图**: [screenshots](https://crates.io/crates/screenshots)
- **图像处理**: [image](https://crates.io/crates/image)

## 📊 性能指标

| 指标 | 说明 |
|------|------|
| 单文件便携版 | 无需安装 |
| 纯本地运行 | 无网络依赖 |
| 快速识别 | 毫秒级响应 |

## 🔐 安全性

- ✅ 纯本地运行
- ✅ 无网络请求
- ✅ 无数据收集
- ✅ 无外部依赖
- ✅ 开源透明

## 🎨 界面预览

**主界面**
- 简洁的双按钮设计
- 清晰的快捷键提示

**扫描结果**
- 卡片式结果展示
- 类型标签（链接/文本/邮箱/电话）
- 一键复制功能

**区域选择**
- 实时尺寸显示
- 半透明遮罩
- ESC 取消操作

## 📝 开发日志

### v0.1.0

- ✅ 初始化 Tauri 2.0 项目
- ✅ 实现全屏识别功能
- ✅ 实现区域识别功能
- ✅ 添加 OCR 文字识别功能
- ✅ 添加系统托盘支持
- ✅ 注册全局快捷键
- ✅ 精美 UI 设计
- ✅ 深色模式支持
- ✅ 便携版单文件输出

## 📄 许可证

MIT License

## 🙏 致谢

- [Tauri](https://tauri.app/) - 跨平台桌面应用框架
- [rxing](https://github.com/rxing-core/rxing) - Rust 二维码识别库
- [PaddleOCR](https://github.com/PaddlePaddle/PaddleOCR) - OCR 模型
- [screenshots](https://github.com/nashaofu/screenshots-rs) - Rust 截图库

---

## ⭐ Star History

[![Star History Chart](https://api.star-history.com/svg?repos=IGNiT3-R/Glimpse&type=Date)](https://star-history.com/#IGNiT3-R/Glimpse&Date)

---

**Made with ❤️ by IgniteRan using Rust & Tauri**
