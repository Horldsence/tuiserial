# TuiSerial 项目架构说明

## 项目概述

TuiSerial 是一个基于终端的串口通信工具，采用 Rust 编写，使用 Ratatui 构建用户界面，支持完整的鼠标和键盘交互。

## Workspace 结构

项目采用 Cargo Workspace 管理，分为以下几个独立的 crate：

```
tuiserial/
├── Cargo.toml                 # Workspace 配置
├── crates/
│   ├── tuiserial-core/        # 核心数据模型
│   ├── tuiserial-serial/      # 串口通信库
│   ├── tuiserial-ui/          # UI 渲染组件
│   └── tuiserial-cli/         # 命令行主程序
└── target/                    # 编译输出
```

### 1. tuiserial-core (核心库)

**职责**：定义核心数据结构和应用状态

**主要组件**：
- `AppState` - 应用程序全局状态
- `SerialConfig` - 串口配置信息
- `MessageLog` - 消息日志管理
- `Notification` - 通知消息系统
- 枚举类型：`DisplayMode`, `TxMode`, `Parity`, `FlowControl`, `FocusedField`

**依赖**：
- chrono - 时间戳处理
- ratatui - UI 状态（ListState）
- serde/serde_json - 配置序列化

### 2. tuiserial-serial (串口通信库)

**职责**：封装串口操作和数据转换

**主要功能**：
- `list_ports()` - 枚举系统可用串口
- `open_port()` - 打开串口连接
- `read_data()` - 读取串口数据
- `write_data()` - 写入串口数据
- `hex_to_bytes()` - HEX 字符串转字节数组
- `bytes_to_hex()` - 字节数组转 HEX 字符串
- `bytes_to_string()` - 字节数组转可读字符串

**依赖**：
- serialport - 跨平台串口库
- tuiserial-core - 核心数据类型

### 3. tuiserial-ui (UI 组件库)

**职责**：渲染用户界面和提供鼠标交互支持

**主要组件**：
- `draw()` - 主渲染函数
- `UiAreas` - UI 区域矩形信息（用于鼠标交互）
- `get_clicked_field()` - 根据鼠标坐标判断点击区域
- `is_inside()` - 判断点是否在矩形内

**UI 布局**：
```
┌─────────────────────────────────────────────────────────┐
│ ┌──────────────┐ ┌───────────────────────────────────┐ │
│ │  配置面板    │ │        数据日志区域             │ │
│ │              │ │                                   │ │
│ │  - 串口      │ │  [时间戳] ◄ RX: 数据...         │ │
│ │  - 波特率    │ │  [时间戳] ► TX: 数据...         │ │
│ │  - 数据位    │ │                                   │ │
│ │  - 校验位    │ │                                   │ │
│ │  - 停止位    │ │                                   │ │
│ │  - 流控制    │ ├───────────────────────────────────┤
│ │              │ │        发送数据区域             │ │
│ │  连接状态    │ │  输入框...                       │ │
│ │              │ │  提示信息                        │ │
│ │              │ ├───────────────────────────────────┤
│ └──────────────┘ │        统计信息                  │ │
│                  └───────────────────────────────────┘ │
├─────────────────────────────────────────────────────────┤
│                   消息提示栏                            │
└─────────────────────────────────────────────────────────┘
```

**依赖**：
- ratatui - TUI 框架
- crossterm - 终端控制
- tuiserial-core - 应用状态
- tuiserial-serial - 数据转换

### 4. tuiserial-cli (命令行程序)

**职责**：程序入口和事件处理

**主要组件**：
- `main()` - 程序入口，初始化终端
- `run_app()` - 主事件循环
- `handle_key_event()` - 键盘事件处理
- `handle_mouse_event()` - 鼠标事件处理
- `SerialHandler` - 串口连接管理器

**依赖**：所有其他 crate

## 交互设计

### 键盘快捷键

#### 全局快捷键
- `q` / `Esc` - 退出程序
- `Tab` - 切换焦点到下一个字段
- `Shift+Tab` - 切换焦点到上一个字段
- `o` - 打开/关闭串口连接
- `r` - 刷新串口列表

#### 配置面板
- `↑` / `k` - 上一项
- `↓` / `j` - 下一项
- `←` / `h` - 减小值（波特率）
- `→` / `l` - 增大值（波特率）
- `p` - 切换校验位
- `f` - 切换流控制

#### 日志区域
- `x` - 切换 HEX/TEXT 显示模式
- `c` - 清空日志
- `a` - 切换自动滚动
- `PgUp` - 向上翻页
- `PgDn` - 向下翻页
- `Home` - 跳到开头
- `End` - 跳到结尾

#### 发送区域（当焦点在发送框时）
- `字符键` - 输入字符
- `Backspace` - 删除前一个字符
- `Delete` - 删除后一个字符
- `←` / `→` - 移动光标
- `Home` / `End` - 光标移到开头/结尾
- `↑` / `↓` - 切换 HEX/ASCII 模式
- `Enter` - 发送数据
- `Esc` - 清空输入

### 鼠标交互

#### 左键点击
- **配置面板** - 切换焦点并选择项目
  - 点击串口列表项 - 选择该串口
  - 点击波特率列表项 - 选择该波特率
  - 点击其他配置项 - 选择该配置
- **日志区域** - 切换焦点到日志区域
- **发送区域** - 切换焦点并定位光标

#### 右键点击
- **日志区域** - 切换 HEX/TEXT 显示模式
- **发送区域** - 切换 HEX/ASCII 发送模式
- **统计信息区** - 切换自动滚动开关

#### 中键点击
- **日志区域** - 清空日志
- **发送区域** - 清空输入

#### 滚轮滚动
- **日志区域** - 向上/向下滚动日志
- **配置列表** - 在列表中向上/向下选择

## 数据流

```
用户输入
   │
   ├─ 键盘事件 ────┐
   │               │
   └─ 鼠标事件 ────┤
                   │
                   ▼
          handle_*_event()
                   │
                   ├─ 修改 AppState
                   │
                   ├─ 调用 SerialHandler
                   │       │
                   │       └─ 串口读写操作
                   │              │
                   │              ▼
                   │       MessageLog 更新
                   │
                   ▼
              UI 重绘 (draw())
```

## 编译和运行

### 开发模式
```bash
cd tuiserial
cargo build
cargo run --bin tuiserial
```

### 发布模式
```bash
cargo build --release
./target/release/tuiserial
```

### 运行特定 crate 的测试
```bash
cargo test -p tuiserial-core
cargo test -p tuiserial-serial
cargo test -p tuiserial-ui
cargo test -p tuiserial-cli
```

## 扩展和维护

### 添加新的配置选项
1. 在 `tuiserial-core` 的 `SerialConfig` 中添加字段
2. 在 `AppState` 中添加对应的 UI 状态
3. 在 `tuiserial-ui` 中创建渲染函数
4. 在 `tuiserial-cli` 的事件处理中添加交互逻辑
5. 在 `tuiserial-serial` 中实现串口配置

### 添加新的 UI 组件
1. 在 `tuiserial-ui` 中创建新的绘制函数
2. 更新 `UiAreas` 结构体以包含新区域
3. 在 `draw()` 函数中调用新组件
4. 在 `get_clicked_field()` 中添加鼠标交互支持

### 优化建议
1. 使用 `Arc<Mutex<>>` 实现多线程安全的状态共享
2. 添加配置文件持久化（JSON/TOML）
3. 实现日志导出功能
4. 添加宏录制和回放功能
5. 支持多串口同时监控
6. 实现协议解析器插件系统

## 性能考虑

- **日志限制**：`MAX_LOG_LINES = 10000`，超出后自动删除旧记录
- **UI 刷新**：100ms 轮询间隔，平衡响应性和 CPU 使用
- **内存管理**：使用 `VecDeque` 实现高效的 FIFO 日志队列
- **异步 I/O**：串口读写使用超时机制，避免阻塞主线程

## 平台兼容性

- ✅ Linux (已测试)
- ✅ macOS (已测试)
- ✅ Windows (理论支持，需测试)
- ✅ BSD 系统 (理论支持)

## 许可证

MIT License

## 贡献指南

1. Fork 项目
2. 创建特性分支 (`git checkout -b feature/AmazingFeature`)
3. 提交更改 (`git commit -m 'Add some AmazingFeature'`)
4. 推送到分支 (`git push origin feature/AmazingFeature`)
5. 开启 Pull Request

---

**维护者**: Your Name <your.email@example.com>
**最后更新**: 2024-12-24