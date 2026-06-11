# TuiSerial - 终端串口调试助手

一个用 Rust + Ratatui 构建的现代化 TUI 串口调试工具，内置 JS/TS 插件系统，支持完整的键盘和鼠标交互。

> **v0.2.0** — 插件系统现在通过 feature 开关控制。不加 `--features plugin` 编译可获得更小巧的二进制文件，启用后获得完整插件体验。

中文文档 | [English](README.md)

![](image/image.png)

## 📦 安装

### 从 crates.io 安装

```bash
# 基础版（不含插件支持）
cargo install tuiserial

# 完整版（包含插件支持）
cargo install tuiserial --features plugin
```
运行：
```bash
tuiserial
```

### 从源码构建

1. 克隆仓库：`git clone https://github.com/horldsence/tuiserial.git`
2. 进入目录：`cd tuiserial`
3. 构建（不含插件支持）：`cargo build --release`
4. 或构建包含完整插件支持：`cargo build --release --features plugin`
5. 运行：`./target/release/tuiserial`

### 使用预编译二进制文件

1. 下载二进制文件：[下载链接](https://github.com/horldsence/tuiserial/releases)
2. 解压文件
3. 运行：`./tuiserial`

## ✨ 功能特性

### 核心功能
- **完整串口配置**：端口选择、波特率、数据位、校验位、停止位、流控制
- **配置持久化**：自动保存/加载配置到 `~/.config/tuiserial/config.json` 💾
- **配置锁定机制**：连接后自动锁定配置，防止误操作，断开后解锁 🔒
- **智能状态显示**：实时显示连接状态和完整配置信息（8-N-1 格式）
- **国际化支持**：支持中英文切换，默认英文 🌍
- **菜单栏导航**：标准菜单栏（文件/会话/视图/设置/插件/帮助），支持键盘和鼠标操作
- **双模式显示**：HEX 和 TEXT 两种显示模式，实时切换
- **简洁消息格式**：`[时间] ◄ RX (字节数) 数据` - 清晰直观
- **双向数据传输**：支持 HEX/ASCII 两种发送模式
- **灵活追加选项**：可选择追加 `\n`、`\r`、`\r\n`、`\n\r` 或无追加
- **实时数据接收**：高效的环形缓冲区，支持最多 10000 行日志
- **自动/手动滚动**：智能自动跟踪或手动浏览历史数据
- **快捷操作**：快速切换配置和显示模式

### 插件系统 🧩（feature 可选：`--features plugin`）
- **JS/TS 插件运行时**：基于 [boa_engine](https://crates.io/crates/boa_engine) 纯 Rust 实现的 JavaScript 引擎，用 JS/TypeScript 编写插件
- **插件钩子**：`onLoad`、`onUnload`、`onConnect`、`onDisconnect`、`onRx(data)`、`onTx(data)` — 拦截和转换串口数据
- **插件管理器界面**：内置模态窗口，查看已安装插件、运行状态、在线重新加载
- **插件市场**：浏览、搜索、通过 Git 从在线注册表安装插件
- **Git 更新机制**：「检查更新」/「更新全部」保持插件最新
- **插件 API**：`tuiserial.log.*`、`tuiserial.config.get()`、`tuiserial.require()`、`tuiserial.fs.read()`
- **插件发现**：将插件放入 `~/.config/tuiserial/plugins/<名称>/`，启动时自动加载
- **安全沙箱**：路径遍历被阻止，插件在沙箱化的 JS 运行时中执行

### 交互特性
- **完整键盘控制**：vim 风格快捷键 + 标准导航键 + F10 菜单
- **全面鼠标支持**：点击、右键、中键、滚轮全支持，菜单栏点击
- **剪贴板粘贴**：支持直接粘贴 HEX 或 ASCII 数据到输入框
- **实时统计**：Tx/Rx 字节数统计和连接状态
- **通知系统**：操作反馈和错误提示，支持多语言

### UI 优化
- **状态面板**：
  - 连接状态：`✓ 已连接` / `✗ 未连接`
  - 配置状态：`🔓 可修改` / `🔒 已锁定`
  - 完整配置信息：串口、波特率、配置格式（8-N-1）
  - 插件数量指示器
- **消息日志**：
  - 简洁标题：`消息 - HEX | 123 条 [x 切换 | c 清空]`
  - 统一格式：`[时间] ◄ RX (字节数) 数据`
  - 智能提示：空日志时显示连接状态和快捷键
- **配置锁定提示**：连接后配置面板显示 `[已锁定]` 标记，边框变灰
- **追加选项选择器**：右侧独立面板快速选择换行符类型
- **高亮提示**：焦点字段黄色高亮，选中项加粗显示，锁定字段灰色显示
- **快捷键帮助**：按 `F1` 或 `?` 查看完整键盘快捷键

## 📦 项目结构

采用 Cargo Workspace 管理的模块化架构：

```
tuiserial/
├── Cargo.toml                 # Workspace 配置
├── crates/
│   ├── tuiserial-core/        # 核心数据模型、状态管理、国际化
│   ├── tuiserial-serial/      # 串口通信库（封装 serialport）
│   ├── tuiserial-ui/          # UI 渲染组件（基于 ratatui）
│   ├── tuiserial-tabs/        # 多标签和分屏布局管理
│   ├── tuiserial-plugin/      # JS/TS 插件运行时（基于 boa_engine）
│   └── tuiserial-cli/         # 主二进制包（发布为 "tuiserial"）
├── docs/
│   └── PLUGIN_DEVELOPMENT_GUIDE_CN.md  # 插件开发指南
├── ARCHITECTURE.md            # 详细架构文档
├── README.md                  # 英文文档
└── README-CN.md               # 本文档（中文）
```

**注意**：目录名为 `tuiserial-cli`，但包在 crates.io 上发布为 `tuiserial`。用户应使用 `cargo install tuiserial` 安装。

## 🚀 快速开始

### 编译

```bash
cd tuiserial

# 基础版 — 不含插件支持（二进制更小，编译更快）
cargo build --release

# 完整版 — 包含 JS/TS 插件系统
cargo build --release --features plugin
```

### 运行

```bash
./target/release/tuiserial
```

或直接运行：

```bash
cargo run --release --bin tuiserial
```

> **注意**：插件系统通过 feature 开关控制。使用不含 `--features plugin` 的构建时，插件管理器界面仍然可访问，但插件操作会提示启用该功能。

## ⌨️ 键盘快捷键

### 全局控制
| 快捷键 | 功能 |
|--------|------|
| `Ctrl+C` / `Ctrl+Q` / `q` / `Esc` | 退出程序 |
| `F10` | 打开/关闭菜单栏 |
| `F1` / `?` | 切换快捷键帮助面板 |
| `Tab` | 切换焦点到下一个字段 |
| `Shift+Tab` | 切换焦点到上一个字段 |
| `o` | 打开/关闭串口连接（连接后锁定配置） |
| `r` | 刷新串口列表 |
| `p` | 打开/关闭插件管理器 |
| `Ctrl+S` | 保存配置 |
| `Ctrl+O` | 加载配置 |

### 菜单栏导航（F10 激活）
| 快捷键 | 功能 |
|--------|------|
| `←` / `→` | 切换菜单项 |
| `↑` / `↓` | 在下拉菜单中选择 |
| `Enter` | 执行选中的菜单项 |
| `Esc` | 关闭菜单/返回上级 |

### 配置面板导航（⚠️ 连接后自动锁定）
| 快捷键 | 功能 |
|--------|------|
| `↑` / `k` | 列表向上/减小值 |
| `↓` / `j` | 列表向下/增大值 |
| `←` / `h` | 减小波特率 |
| `→` / `l` | 增大波特率 |
| `p` | 切换校验位（None → Even → Odd） |
| `f` | 切换流控制（None → Hardware → Software） |

**注意**：连接串口后，所有配置参数自动锁定，无法修改。必须先断开连接才能调整配置。

### 日志区域
| 快捷键 | 功能 |
|--------|------|
| `x` | 切换 HEX/TEXT 显示模式 |
| `c` | 清空日志 |
| `a` | 切换自动滚动 |
| `PgUp` | 向上翻页（10行） |
| `PgDn` | 向下翻页（10行） |
| `Home` | 跳到日志开头 |
| `End` | 跳到日志末尾（并开启自动滚动） |

### 发送区域（焦点在发送框时）
| 快捷键 | 功能 |
|--------|------|
| `字符键` | 输入字符 |
| `粘贴` | 粘贴 HEX 或 ASCII 数据 |
| `Backspace` | 删除前一个字符 |
| `Delete` | 删除后一个字符 |
| `←` / `→` | 移动光标 |
| `Home` / `End` | 光标移到开头/结尾 |
| `↑` / `↓` | 切换 HEX/ASCII 模式 |
| `n` | 循环切换追加选项 |
| `Enter` | 发送数据 |
| `Esc` | 清空输入 |

### 插件管理器
| 快捷键 | 功能 |
|--------|------|
| `p` | 打开/关闭插件管理器（本地视图） |
| `↑` / `↓` | 导航插件列表 |
| `Enter` | 安装选中的插件（市场视图） |
| `r` | 重新加载所有插件 |
| `Esc` / `q` / `p` | 关闭插件管理器 |

## 🖱️ 鼠标交互

### 左键点击
- **菜单栏** → 打开菜单下拉列表
- **菜单项** → 执行对应功能
- **配置面板** → 切换焦点并直接选择列表项
- **日志区域** → 切换焦点到日志区域
- **发送框** → 切换焦点并定位光标位置
- **追加选项** → 直接选择追加模式

### 右键点击
- **日志区域** → 快速切换 HEX/TEXT 显示模式
- **发送框** → 快速切换 HEX/ASCII 发送模式
- **追加选项** → 循环切换追加模式
- **统计信息区** → 切换自动滚动开关

### 中键点击
- **日志区域** → 快速清空日志
- **发送框** → 快速清空输入

### 滚轮滚动
- **日志区域** → 向上/向下滚动日志（3行）
- **配置列表** → 在列表中向上/向下选择
- **追加选项** → 循环切换追加模式
- **插件管理器** → 导航插件列表

## 📊 数据格式

### 接收显示格式
```
[14:32:45.123] ◄ RX (   5 B) 48 65 6C 6C 6F
[14:32:45.456] ◄ RX (   5 B) Hello
```

**格式说明**：
- 时间戳精确到毫秒
- `◄ RX` 接收方向（青色加粗）
- `► TX` 发送方向（绿色加粗）
- 字节数右对齐，便于查看

### 发送模式
1. **ASCII 模式**：直接输入文本，如 `Hello`
2. **HEX 模式**：输入十六进制，空格分隔，如 `48 65 6C 6C 6F`

### 追加选项
- **无追加**：不添加任何字符
- **\n**：添加换行符（LF，0x0A）
- **\r**：添加回车符（CR，0x0D）
- **\r\n**：添加回车换行（CRLF，0x0D 0x0A）
- **\n\r**：添加换行回车（LFCR，0x0A 0x0D）

## 🛠️ 技术栈

- **Ratatui 0.29**：现代的 Rust TUI 框架
- **Crossterm 0.28**：跨平台终端控制
- **Serialport 4.3+**：跨平台串口访问
- **Boa Engine 0.21**：纯 Rust 实现的 JavaScript 运行时，用于插件系统
- **Tokio 1.40**：异步运行时
- **Chrono 0.4**：时间戳处理
- **Color-eyre 0.6**：错误处理
- **PHF 0.11**：编译时完美哈希表，用于国际化

## 📈 开发状态

### ✅ 已实现
- ✅ 串口配置管理（所有常用参数）
- ✅ **配置持久化**（自动保存/加载配置文件）
- ✅ **菜单栏系统**（文件/会话/视图/设置/插件/帮助，支持键盘和鼠标）
- ✅ **国际化支持**（中英文切换，编译时零开销）
- ✅ **插件系统**（JS/TS 运行时、插件管理器、注册表、Git 更新）
- ✅ **配置锁定机制**（连接后自动锁定，防止误操作）
- ✅ **智能状态显示**（连接状态、配置状态、完整配置信息）
- ✅ 数据接收显示（HEX/TEXT 模式）
- ✅ 数据发送功能（HEX/ASCII 模式）
- ✅ 追加选项（\n, \r, \r\n, \n\r, 无）
- ✅ 剪贴板粘贴支持
- ✅ 完整键盘控制（含 F10 菜单、F1/? 快捷键帮助）
- ✅ 完整鼠标交互（点击、右键、中键、滚轮、菜单栏）
- ✅ 优化消息格式（简洁直观的日志显示）
- ✅ 自动/手动滚动
- ✅ 实时统计和通知系统
- ✅ 模块化架构（6 个 crate 组成的 Workspace）

### 🔄 路线图
- 🔄 插件生态扩展 — 更多注册表插件，插件 SDK 改进
- 🔄 多会话与标签管理（session crate 已就绪，UI 接入中）
- 🔄 分屏布局（单视图、水平分割、垂直分割、2×2 网格）
- 🔄 命令预设和快速发送
- 🔄 日志导出（TXT/CSV/JSON）
- 🔄 日志搜索和过滤功能
- 🔄 数据分析和实时图表
- 🔄 多串口同时监控
- 🔄 宏录制和回放
- 🔄 更多语言支持（日语、韩语等）

## 📚 文档

- [ARCHITECTURE.md](ARCHITECTURE.md) - 详细架构设计文档
- [docs/PLUGIN_DEVELOPMENT_GUIDE_CN.md](docs/PLUGIN_DEVELOPMENT_GUIDE_CN.md) - 插件开发指南
- [Cargo Workspace](https://doc.rust-lang.org/book/ch14-03-cargo-workspaces.html) - Cargo Workspace 官方文档

## 🧩 插件系统概览

> **Feature 开关**：插件系统为可选功能。编译时启用：`cargo build --release --features plugin`。禁用时，插件管理器界面仍然可访问，点击插件操作会引导你启用该功能。

TuiSerial 内置基于 [boa_engine](https://crates.io/crates/boa_engine)（纯 Rust 实现的 JavaScript 引擎）的 JS/TS 插件系统。插件可以实时拦截和转换串口数据。

### 快速上手

1. 创建插件目录：`~/.config/tuiserial/plugins/my-plugin/`
2. 创建 `plugin.ts`（或 `plugin.js`）作为入口文件：

```js
// 串口连接时调用
function onConnect() {
    tuiserial.log.success("已连接！我的插件已就绪。");
}

// 拦截接收数据
function onRx(data) {
    // data 是 number[]（字节数组）
    tuiserial.log.info("接收: " + data.length + " 字节");
    return data; // 原样返回，或返回修改后的数据
}

function onTx(data) {
    return data; // 返回 null 可阻止发送
}
```

3. 按 `p` 打开插件管理器，或按 `r` 重新加载插件。

### 插件 API

| API | 说明 |
|-----|------|
| `tuiserial.log.info(msg)` | 记录信息日志 |
| `tuiserial.log.warn(msg)` | 记录警告日志 |
| `tuiserial.log.error(msg)` | 记录错误日志 |
| `tuiserial.log.success(msg)` | 记录成功日志 |
| `tuiserial.config.get()` | 获取当前串口配置 |
| `tuiserial.require(path)` | 加载插件目录中的另一个 JS/TS 文件 |
| `tuiserial.fs.read(path)` | 以 UTF-8 字符串读取文件 |
| `tuiserial.fs.readBinary(path)` | 以 `number[]` 读取二进制文件 |

详细的插件开发文档请参阅 [docs/PLUGIN_DEVELOPMENT_GUIDE_CN.md](docs/PLUGIN_DEVELOPMENT_GUIDE_CN.md)。

## 🔐 核心特性：配置锁定机制

**为什么需要配置锁定？**
在串口通信中，连接建立后修改参数可能导致：
- 通信中断或数据损坏
- 设备响应异常
- 调试信息混乱

**我们的解决方案：**
1. ✅ **连接时自动锁定** - 按 `o` 连接后，所有配置参数立即锁定
2. ✅ **视觉反馈** - 配置面板显示 `[已锁定]` 标记，边框变灰
3. ✅ **操作拦截** - 任何修改尝试都会显示警告："配置已锁定，请先断开连接"
4. ✅ **断开解锁** - 再次按 `o` 断开后，配置恢复可修改状态
5. ✅ **状态同步** - 状态面板实时显示当前配置和锁定状态

**实际效果：**
```
未连接时：
  状态：✗ 未连接
  配置：🔓 可修改
  → 可以自由调整所有参数

连接后：
  状态：✓ 已连接
  配置：🔒 已锁定
  串口：/dev/ttyUSB0
  波特：115200
  配置：8-N-1
  → 参数锁定，无法修改

断开后：
  状态：✗ 未连接
  配置：🔓 可修改
  → 恢复可修改状态
```

## 🤝 贡献指南

欢迎提交 Issue 和 Pull Request！

1. Fork 本项目
2. 创建特性分支 (`git checkout -b feature/AmazingFeature`)
3. 提交更改 (`git commit -m 'Add some AmazingFeature'`)
4. 推送到分支 (`git push origin feature/AmazingFeature`)
5. 开启 Pull Request

## 📝 许可证

MIT License

## 👨‍💻 作者

pengheng <m18511047688@163.com>

## 🌍 国际化

目前支持：
- **English** (默认)
- **中文**

切换语言：
- 按 `F10` 打开菜单
- 选择 `设置` → `切换语言`
- 或直接点击菜单栏

技术实现：
- 使用 `phf` 实现编译时静态 HashMap
- 零运行时开销，所有翻译在编译时嵌入
- Fallback 机制：找不到翻译时返回 key 本身
- 简单直接，无复杂框架依赖

## 💾 配置文件

配置自动保存到：
- **Linux/macOS**: `~/.config/tuiserial/config.json`
- **Windows**: `%APPDATA%\tuiserial\config.json`

配置内容：
```json
{
  "port": "/dev/ttyUSB0",
  "baud_rate": 115200,
  "data_bits": 8,
  "parity": "None",
  "stop_bits": 1,
  "flow_control": "None"
}
```

操作：
- **保存配置**：菜单 → 文件 → 保存配置（或 `Ctrl+S`）
- **加载配置**：菜单 → 文件 → 加载配置（启动时自动加载，或 `Ctrl+O`）
- 配置文件损坏时自动使用默认配置，不会崩溃

---

**⭐ 如果这个项目对你有帮助，请给一个 Star！**
