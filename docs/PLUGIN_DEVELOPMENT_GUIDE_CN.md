# TuiSerial 插件开发指南

TuiSerial 插件系统允许通过 JavaScript / TypeScript 扩展串口数据的收发处理逻辑。插件运行在基于 [Boa](https://crates.io/crates/boa_engine) 的纯 Rust JS 运行时中，无需安装 Node.js。

## 目录

- [快速开始](#快速开始)
- [插件目录结构](#插件目录结构)
- [插件元数据](#插件元数据)
- [Hook 函数](#hook-函数)
- [tuiserial 全局 API](#tuiserial-全局-api)
- [多文件插件](#多文件插件)
- [插件生命周期](#插件生命周期)
- [数据管线](#数据管线)
- [TypeScript 支持](#typescript-支持)
- [插件安装与管理](#插件安装与管理)
- [路径解析与安全](#路径解析与安全)
- [完整示例](#完整示例)
- [发布插件到注册表](#发布插件到注册表)
- [调试技巧](#调试技巧)
- [API 速查表](#api-速查表)

## 快速开始

一个最小的插件只需要一个文件。在 `~/.config/tuiserial/plugins/` 下创建目录和入口文件：

```
~/.config/tuiserial/plugins/
└── my-plugin/
    └── plugin.ts
```

`plugin.ts` 内容：

```typescript
function onLoad() {
    tuiserial.log.info("MyPlugin loaded");
}

function onRx(data: number[]): number[] | null {
    // 打印收到的字节数
    tuiserial.log.info(`Received ${data.length} bytes`);
    return null; // null = 原样放行
}
```

启动 TuiSerial，打开 **Plugins → Reload Plugins** 即可加载。**Plugins → Plugin List** 可查看插件状态。

## 插件目录结构

```
~/.config/tuiserial/plugins/
├── my-plugin/               # 插件名 = 目录名
│   ├── plugin.ts            # 入口文件（plugin.js 也可）
│   ├── plugin.json          # 元数据（可选，GitHub 管理必需）
│   └── lib/                 # 子模块目录
│       └── utils.ts         # 可通过 require 加载
│
├── another-plugin/
│   └── plugin.js
│
└── disabled/                # 此目录下的插件不会被加载
    └── old-plugin/
        └── plugin.ts
```

规则：

- 每个**直接子目录**就是一个插件，目录名即插件名
- 目录下必须有 `plugin.ts` 或 `plugin.js`（优先 .ts）
- 放在 `disabled/` 子目录下的插件会被跳过
- 插件按名称字母排序加载，顺序可预测

## 插件元数据

在插件目录下放置 `plugin.json` 文件，用于 GitHub 更新管理：

```json
{
  "name": "hex-logger",
  "version": "1.0.0",
  "repo": "https://github.com/user/tuiserial-hex-logger",
  "description": "以 HEX 格式记录所有收发的串口数据",
  "author": "your-name"
}
```

| 字段 | 必填 | 说明 |
|------|------|------|
| `name` | 是 | 插件名称 |
| `version` | 否 | 语义化版本号 |
| `repo` | 否 | GitHub 仓库地址，用于检查更新和安装 |
| `description` | 否 | 简要描述 |
| `author` | 否 | 作者名 |

如果插件是通过 `git clone` 安装的面没有 `plugin.json`，系统会从 `git remote get-url origin` 自动推断 repo 地址。

## Hook 函数

插件通过定义全局函数来注册 Hook。所有 Hook 都是**可选的** — 只定义你需要的即可。

### 生命周期 Hook

#### `onLoad()`

```typescript
function onLoad() {
    tuiserial.log.success("插件初始化完成");
}
```

插件被加载时调用（等价于程序启动或手动 Reload）。适合做一次性初始化。

#### `onUnload()`

```typescript
function onUnload() {
    tuiserial.log.info("插件卸载，清理资源");
}
```

插件卸载时调用（程序退出或 Reload 前）。适合清理状态。

#### `onConnect()`

```typescript
function onConnect() {
    const cfg = tuiserial.config.get();
    tuiserial.log.info(`已连接到 ${cfg.port} @ ${cfg.baudRate}`);
}
```

串口连接建立时调用。此时可通过 `tuiserial.config.get()` 获取当前串口配置。

#### `onDisconnect()`

```typescript
function onDisconnect() {
    tuiserial.log.info("串口已断开");
}
```

串口断开时调用。

### 数据 Hook

#### `onRx(data)`

```typescript
function onRx(data: number[]): number[] | null {
    // data 是接收到的原始字节数组
    // 返回值决定数据处理方式
    return null;
}
```

**每次收到串口数据时调用。** `data` 是字节数组（每个元素 0–255）。

| 返回值 | 行为 |
|--------|------|
| `null` 或 `undefined` | **原样放行**，不做修改 |
| `[]`（空数组） | **丢弃数据**，不显示也不传给后续插件 |
| `[byte, byte, ...]`（非空数组） | **替换数据**，用返回的字节数组替代原始数据 |
| 抛出异常 | 插件被标记为错误，跳过后续调用 |

#### `onTx(data)`

```typescript
function onTx(data: number[]): number[] | null {
    // data 是即将发送的字节数组
    return null;
}
```

**每次发送数据前调用。** 签名和语义与 `onRx` 完全一致。可用于加密、协议包装、日志记录等。

## tuiserial 全局 API

插件中可以访问全局对象 `tuiserial`，提供以下功能。

### tuiserial.log — 日志输出

日志会显示在 TuiSerial 的通知栏中。

```typescript
tuiserial.log.info("普通消息");
tuiserial.log.warn("警告信息");
tuiserial.log.error("错误信息");
tuiserial.log.success("成功信息");
```

### tuiserial.config — 读取串口配置

```typescript
const cfg = tuiserial.config.get();
// cfg.port        → 串口路径，如 "/dev/ttyUSB0"
// cfg.baudRate    → 波特率，如 115200
// cfg.dataBits    → 数据位，如 8
// cfg.parity      → 校验位: "none" | "even" | "odd"
// cfg.stopBits    → 停止位，如 1
// cfg.flowControl → 流控: "none" | "hardware" | "software"
```

注意：配置是只读的快照 — 修改返回的对象不会影响实际串口配置。

### tuiserial.require — 加载子模块

```typescript
const utils = tuiserial.require("./lib/utils.ts");
utils.doSomething();
```

- 路径相对于**当前插件目录**
- 支持 `.ts` 和 `.js` 文件
- 模块会被**缓存** — 多次 require 同一路径返回同一个 exports 对象
- 被加载的文件也能使用 `tuiserial` 全局对象

子模块写法：

```typescript
// lib/utils.ts — 子模块
var exports = exports || {};

function doSomething() {
    tuiserial.log.info("来自子模块");
}

exports.doSomething = doSomething;
```

### tuiserial.fs — 读取文件

```typescript
// 读取文本文件（UTF-8）
const configJson = tuiserial.fs.read("config.json");
const settings = JSON.parse(configJson);

// 读取二进制文件 → number[]
const firmware = tuiserial.fs.readBinary("firmware.bin");
tuiserial.log.info(`固件大小: ${firmware.length} bytes`);
```

路径解析规则同 `require`，限制在插件目录内。

## 多文件插件

推荐拆分复杂插件为多个文件：

```
my-plugin/
├── plugin.ts          # 入口：定义 hooks
├── lib/
│   ├── parser.ts      # 解析逻辑
│   └── encoder.ts     # 编码逻辑
└── config.json        # 配置文件
```

入口文件示例：

```typescript
const parser = tuiserial.require("./lib/parser.ts");
const encoder = tuiserial.require("./lib/encoder.ts");

function onLoad() {
    const cfg = JSON.parse(tuiserial.fs.read("config.json"));
    parser.init(cfg);
    tuiserial.log.success("插件已加载");
}

function onRx(data: number[]): number[] | null {
    const decoded = parser.decode(data);
    tuiserial.log.info(`解析结果: ${JSON.stringify(decoded)}`);
    return null;
}

function onTx(data: number[]): number[] | null {
    const encoded = encoder.encode(data);
    return encoded;
}
```

## 插件生命周期

```
程序启动 / Reload
  │
  ▼
扫描插件目录 → 发现 plugin.ts/js → 创建运行时
  │
  ▼
加载 JS → 注册 tuiserial 全局对象 → 检测 Hook → 调用 onLoad()
  │
  ▼
┌──────────────────────────────────────────────────┐
│                                                    │
│   [串口连接] → onConnect()                          │
│       │                                             │
│   [数据收发] → onRx() / onTx() 管线                │
│       │                                             │
│   [串口断开] → onDisconnect()                       │
│       │                                             │
│   [重新连接] → onConnect() ...                      │
│                                                    │
└──────────────────────────────────────────────────┘
  │
  ▼
程序退出 / Reload → onUnload() → 释放运行时
```

关键点：

- `onLoad` 和 `onUnload` 与**插件生命周期**绑定，不与串口连接绑定
- `onConnect` / `onDisconnect` 在每次串口连接/断开时调用
- Reload 会先 `onUnload` 所有插件，再重新扫描并 `onLoad`

## 数据管线

多个插件的数据处理按**加载顺序**（名称排序）依次执行：

```
原始 RX 数据
  │
  ▼
插件 A.onRx() → PassThrough → 原数据不变
  │
  ▼
插件 B.onRx() → Modified([...]) → 数据被修改
  │
  ▼
插件 C.onRx() → Suppressed(return []) → 丢弃！
  │
  ▼
(不再显示)
```

管线规则：

| 场景 | 行为 |
|------|------|
| 插件 A → `PassThrough` | 数据原样传给插件 B |
| 插件 A → `Modified(bytes)` | 修改后的数据传给插件 B |
| 插件 A → `Suppressed` | **立即终止管线**，数据被丢弃 |
| 插件 A → 异常 | 插件 A 被标记为 Error，跳过，数据继续传给 B |

**TX 管线流程相同**，只是 Hook 名改为 `onTx`，在数据发送到串口之前执行。

**注意事项**：

- 管线中某插件报错后，该插件被**永久跳过**直到 Reload
- 数据的修改和丢弃对所有后续插件生效
- 如果没有任何插件返回 Suppressed，最终数据按正常流程显示/发送

## TypeScript 支持

插件入口和子模块都可以是 `.ts` 文件。TuiSerial 内置了一个 **TS 标注剥离器**（而非完整编译器），在运行前自动处理：

✅ **支持的特性**：
- `export function` / `export const` — `export` 关键字会被移除
- 类型标注 `x: number`, `data: number[]` — 标注部分被移除
- 泛型 `Array<number>`, `Promise<string>`
- `as Type` 类型断言
- `interface Name { ... }` 会被整个移除
- `type Alias = ...;` 会被整个移除

❌ **不支持的特性**：
- `enum` 声明
- `namespace` / `module`
- 装饰器
- `import` / `export {}` 模块语法（请用 `tuiserial.require`）
- 箭头函数中的泛型 `<T>(x: T) => ...`

**建议**：TypeScript 用于编写时类型检查（在 IDE 中），运行依赖剥离器。复杂的 TS 特性应在外部编译为 JS 后放入插件目录。

```typescript
// ✅ 可以
function onRx(data: number[]): number[] | null {
    const result: number[] = [];
    for (let i = 0; i < data.length; i++) {
        result.push(data[i] as number);
    }
    return result;
}

// ❌ 不支持
enum State { IDLE, ACTIVE }
import { helper } from "./utils";
```

## 插件安装与管理

### 通过 GitHub 安装

菜单操作：**Plugins → Install from Registry**

此操作会从内置注册表读取可用插件列表，并 `git clone` 到插件目录。安装后需 **Plugins → Reload Plugins** 激活。

### 手动安装

```bash
cd ~/.config/tuiserial/plugins/
git clone https://github.com/user/some-plugin.git
```

启动 TuiSerial 后 **Plugins → Reload Plugins**。

### 检查更新

**Plugins → Check for Updates** — 遍历所有 git 管理的插件，执行 `git fetch` 比对本地与远程的 commit hash。结果在通知栏显示，格式为：

```
my-plugin 有更新: a1b2c3d → e4f5g6h
所有插件已是最新
```

### 更新插件

**Plugins → Update All** — 对所有有更新的插件执行 `git pull --ff-only`（仅快进合并，安全不会产生冲突）。

### 手动管理

```bash
# 禁用插件
mv ~/.config/tuiserial/plugins/my-plugin ~/.config/tuiserial/plugins/disabled/

# 启用插件
mv ~/.config/tuiserial/plugins/disabled/my-plugin ~/.config/tuiserial/plugins/

# 删除插件
rm -rf ~/.config/tuiserial/plugins/my-plugin
```

## 路径解析与安全

`tuiserial.require()` 和 `tuiserial.fs` 的路径解析遵循以下规则：

```
当前插件目录: ~/.config/tuiserial/plugins/my-plugin/

"./lib/utils.ts"       → ~/.config/tuiserial/plugins/my-plugin/lib/utils.ts
"config.json"           → ~/.config/tuiserial/plugins/my-plugin/config.json
"../other/plugin.ts"    → ~/.config/tuiserial/plugins/other/plugin.ts
"../../../etc/passwd"   → 被阻止 (越界)
"/absolute/path"        → 被阻止 (绝对路径)
""                      → 被阻止 (空路径)
```

**允许范围**：插件目录及其父目录（即 plugins 根目录）。这意味着插件可以读取同级其他插件目录下的文件（用于插件间协作）。但不能访问 plugins 目录之外的任何文件。

## 完整示例

### 示例 1：HEX 日志记录器

将收发数据以 HEX 格式记录到通知栏。

```typescript
// plugin.ts
function bytesToHex(data: number[]): string {
    const hex: string[] = [];
    for (let i = 0; i < data.length; i++) {
        hex.push(data[i].toString(16).padStart(2, "0"));
    }
    return hex.join(" ");
}

function onRx(data: number[]): null {
    tuiserial.log.info(`RX [${data.length}B] ${bytesToHex(data)}`);
    return null;
}

function onTx(data: number[]): null {
    tuiserial.log.info(`TX [${data.length}B] ${bytesToHex(data)}`);
    return null;
}
```

### 示例 2：协议帧解析

从二进制流中解析以 `0xAA 0xBB` 为帧头、`0x0D 0x0A` 为帧尾的协议。

```typescript
// plugin.ts
var buffer: number[] = [];
const FRAME_HEAD = [0xAA, 0xBB];
const FRAME_TAIL = [0x0D, 0x0A];

function startsWith(data: number[], prefix: number[], offset: number): boolean {
    for (let i = 0; i < prefix.length; i++) {
        if (data[offset + i] !== prefix[i]) return false;
    }
    return true;
}

function onRx(data: number[]): number[] | null {
    // 追加到缓冲区
    for (let i = 0; i < data.length; i++) {
        buffer.push(data[i]);
    }

    // 搜索完整帧
    let result: number[] = [];
    let i = 0;
    while (i <= buffer.length - FRAME_TAIL.length) {
        // 查找帧尾
        if (startsWith(buffer, FRAME_TAIL, i)) {
            const frame = buffer.slice(0, i + FRAME_TAIL.length);
            // 找到了，移除 frame 从 buffer
            buffer = buffer.slice(i + FRAME_TAIL.length);
            // 检查帧头
            if (frame.length >= FRAME_HEAD.length && startsWith(frame, FRAME_HEAD, 0)) {
                const payload = frame.slice(FRAME_HEAD.length, frame.length - FRAME_TAIL.length);
                tuiserial.log.success(`帧数据: ${payload.length} bytes`);
                // 将帧数据加入输出
                for (let j = 0; j < frame.length; j++) {
                    result.push(frame[j]);
                }
            }
            i = 0;
            continue;
        }
        i++;
    }

    // 如果提取到了帧，输出它们；否则返回原始数据
    return result.length > 0 ? result : null;
}
```

### 示例 3：带配置的协议转换

使用 `plugin.json` 和 `config.json` 的完整插件。

```
protocol-converter/
├── plugin.ts
├── plugin.json
├── config.json
└── lib/
    └── modbus.ts
```

```json
// plugin.json
{
  "name": "protocol-converter",
  "version": "1.0.0",
  "repo": "https://github.com/user/tuiserial-protocol-converter",
  "description": "Modbus-RTU 协议解析",
  "author": "developer"
}
```

```json
// config.json
{
  "slaveId": 1,
  "timeout": 1000
}
```

```typescript
// lib/modbus.ts
var exports = exports || {};

interface ModbusConfig {
    slaveId: number;
    timeout: number;
}

var config: ModbusConfig = { slaveId: 1, timeout: 1000 };

function init(cfg: ModbusConfig) {
    config = cfg;
}

function parseRequest(data: number[]): object | null {
    if (data.length < 8) return null;
    if (data[0] !== config.slaveId) return null; // 不是发给本从机的
    const funcCode = data[1];
    const result = {
        slaveId: data[0],
        functionCode: funcCode,
        data: data.slice(2, data.length - 2),
        crc: data[data.length - 2] | (data[data.length - 1] << 8)
    };
    return result;
}

exports.init = init;
exports.parseRequest = parseRequest;
```

```typescript
// plugin.ts
const modbus = tuiserial.require("./lib/modbus.ts");

function onLoad() {
    const cfg = JSON.parse(tuiserial.fs.read("config.json"));
    modbus.init(cfg);
    tuiserial.log.success("Modbus 解析器已就绪");
}

function onRx(data: number[]): number[] | null {
    const request = modbus.parseRequest(data);
    if (request) {
        tuiserial.log.info(
            `Modbus 请求: 从机=${request.slaveId} 功能码=${request.functionCode}`
        );
    }
    return null;
}
```

## 发布插件到注册表

将你的插件分享给社区：

1. 在 GitHub 上创建公开仓库，按[插件目录结构](#插件目录结构)组织代码
2. 在根目录添加 `plugin.json`，填写 `repo` 字段
3. （可选）添加 `README.md` 说明插件的用途和用法
4. 向 [tuiserial 主仓库](https://github.com/horldsence/tuiserial) 提交 PR，在 `crates/tuiserial-plugin/src/registry.rs` 的 `builtin_registry()` 中添加条目：

```rust
RegistryEntry {
    name: "your-plugin-name".into(),
    repo: "https://github.com/your-username/your-plugin-repo".into(),
    description: Some("简要描述插件功能".into()),
    author: Some("your-name".into()),
},
```

合入后，所有用户即可通过 **Plugins → Install from Registry** 一键安装。

## 调试技巧

### 查看 Hook 检测结果

**Plugins → Plugin List** 显示每个插件注册了哪些 Hook：

```
✓ my-plugin (rx:true, tx:true)
✓ hex-logger (rx:true, tx:false)
```

### 利用日志

在 Hook 中大量使用 `tuiserial.log.*` 输出调试信息。日志会显示在 TuiSerial 底部通知栏。

```typescript
function onRx(data: number[]): null {
    tuiserial.log.info(`入参类型: ${typeof data}, 长度: ${data.length}`);
    tuiserial.log.info(`前5字节: ${data.slice(0, 5)}`);
    return null;
}
```

### 错误排查

- 插件加载失败时，通知栏会显示具体错误信息
- 数据 Hook 抛出异常后，插件被标记为 Error（⚠），不再参与管线
- 修复后需 **Plugins → Reload** 重新加载
- 检查 `plugin.ts` 是否在插件目录的**根层级**（不能在子目录中）

### 常见错误

| 现象 | 可能原因 |
|------|---------|
| 插件未在列表中 | 文件不在根目录、文件名不是 `plugin.ts`/`plugin.js`、放在 `disabled/` 下了 |
| Hook 显示全 false | 函数名拼写错误（区分大小写）、TS 标注剥离异常 |
| onRx 返回数据无效 | 返回的数组包含非数字元素 |
| require 失败 | 路径越界、文件不存在、子模块 JS 语法错误 |

## API 速查表

### Hook 函数

| 函数 | 触发时机 | 参数 | 返回值 |
|------|---------|------|--------|
| `onLoad()` | 插件加载 | 无 | void |
| `onUnload()` | 插件卸载 | 无 | void |
| `onConnect()` | 串口连接 | 无 | void |
| `onDisconnect()` | 串口断开 | 无 | void |
| `onRx(data)` | 收到数据 | `number[]` | `number[] \| null \| undefined` |
| `onTx(data)` | 发送数据前 | `number[]` | `number[] \| null \| undefined` |

### tuiserial 全局对象

| 路径 | 类型 | 说明 |
|------|------|------|
| `tuiserial.log.info(msg)` | 函数 | 信息日志 |
| `tuiserial.log.warn(msg)` | 函数 | 警告日志 |
| `tuiserial.log.error(msg)` | 函数 | 错误日志 |
| `tuiserial.log.success(msg)` | 函数 | 成功日志 |
| `tuiserial.config.get()` | 函数 | 获取串口配置对象 |
| `tuiserial.require(path)` | 函数 | 加载子模块（相对路径，有缓存） |
| `tuiserial.fs.read(path)` | 函数 | 读取文本文件，返回 string |
| `tuiserial.fs.readBinary(path)` | 函数 | 读取二进制文件，返回 number[] |

### 数据 Hook 返回值语义

| 返回值 | 管线行为 |
|--------|---------|
| `null` / `undefined` / 不返回 | 数据原样传递给下一插件 |
| `[]` (空数组) | 立即终止管线，数据被丢弃 |
| `[0-255, ...]` (非空) | 数据被替换为返回的字节数组 |
