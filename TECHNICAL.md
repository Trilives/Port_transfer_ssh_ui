# SSH Port Forwarder 技术文档

本文档面向开发者，记录当前项目架构、开发环境、构建方式和发布流程。
完整架构与数据模型见 [ARCHITECTURE.md](ARCHITECTURE.md)。

## 技术栈

- 桌面应用壳与本机后端：Tauri 2 + Rust
- 前端界面：React + TypeScript + Tailwind CSS
- UI 风格：本地 shadcn/ui 风格组件
- 转发执行：Rust 调用系统 `ssh.exe`
- 外部终端：Rust 起一个 PowerShell 窗口运行交互式 `ssh`
- 打包目标：Windows exe 和 NSIS 安装包

## 项目结构

v0.2.0 把单文件 `lib.rs` 拆为多个模块，并把前端拆为页面 / 组件 / 弹窗。

```text
src-tauri/src/
  main.rs            入口 + askpass helper
  lib.rs             run()、Tauri Builder、命令注册
  model.rs           Host / Forward / 枚举 / AppSettings / LogEntry / View
  store.rs           read_json / write_json
  state.rs           AppState、ManagedTunnel、视图与查找方法、日志写入
  validate.rs        validate_host / validate_forward
  portcheck.rs       端口占用检测（netstat + tasklist）
  terminal.rs        open_terminal：外部 PowerShell 终端
  ssh/               command / process / probe / keys
  commands/          hosts / forwards / exec / keys / settings（Tauri 命令）

frontend/src/
  App.tsx            壳：导航、页面路由、弹窗编排、主题应用
  types.ts api.ts i18n.ts
  pages/             DashboardPage / ConfigPage / LogsPage / SettingsPage / GuidePage
  components/        HostCard / ForwardRow / LogTable / StatusBadge / dialogs.tsx
  components/ui/     button / card / input / select
```

旧的 Python/Tkinter 原型和 uv 环境已经移除。

## 数据模型

- **Host（一级）**：SSH 连接参数（主机、端口、用户、私钥、额外参数）+ `forwards` 列表。
- **Forward（二级）**：转发参数（模式、监听地址/端口、目标地址/端口、保持连接）。
- 持久化文件 `hosts.json`（嵌套 forwards）。v0.1.x 的 `profiles.json` 不迁移，启动时备份为 `profiles.json.v0.1.bak`。

## Tauri 命令

主机：`list_hosts` / `save_host` / `delete_host`。
转发：`save_forward` / `delete_forward` / `connect_forward` / `connect_forward_with_password` /
`disconnect_forward` / `disconnect_host` / `disconnect_all`。
指令与终端：`send_command` / `open_terminal`。
密钥与探测：`upload_public_key` / `probe_connection` / `get_host_fingerprint` / `remove_known_host`。
设置与日志：`get_settings` / `save_settings_cmd` / `list_logs`。
系统环境：`check_ssh`（检测 `ssh.exe`）/ `install_openssh`（提权安装 OpenSSH 客户端）。

事件：`log-entry`（单条日志）、`critical-error`（退出码 255，载荷含 hostId/forwardId）。

## 后端行为

Rust 后端负责：

- 读取和保存主机与转发（`hosts.json`）、主题/语言/日志等级设置。
- 调用 `ssh.exe` 启动 Local / Remote / Dynamic 转发，复用父主机连接参数。
- 管理 SSH 子进程的连接、断开、整机断开、全部断开和自动重连。
- 新建/连接转发前用 `netstat -ano` + `tasklist` 检测本机监听端口是否被占用（local / dynamic）。
- 保存主机/转发前校验参数（主机/端口/转发地址等）。
- 连接前用 `BatchMode=yes` 探测（`probe_connection`），区分免密直连 / 需要密码 / 主机指纹变化 / 不可达。
- 指纹变化时用 `get_host_fingerprint` 展示新指纹，`remove_known_host` 在用户确认后移除旧记录再重试。
- `send_command` 通过 SSH 在主机上执行一条指令并返回输出（依赖免密登录）。
- `open_terminal` 起一个外部 PowerShell 窗口运行交互式 `ssh`（`CREATE_NEW_CONSOLE`）。
- 一键把本地公钥上传到远程主机的 `authorized_keys`，配置免密登录。
- 通过 `SSH_ASKPASS` 实现自动弹出的一次性密码连接。
- 在 Windows 下使用 `CREATE_NO_WINDOW` 启动后台 SSH 子进程，避免弹出终端窗口。
- 在应用退出时清理所有由本程序启动的 SSH 转发进程。

## 数据存储

后端使用 `directories::ProjectDirs` 获取系统应用数据目录。主机配置、设置和日志写入用户应用数据目录，而不是源码目录。

## 开发环境

需要安装：

- Node.js
- npm
- Rust stable toolchain
- Windows OpenSSH Client

如果 PowerShell 无法识别 `cargo`、`rustc` 或 `rustup`，先确认用户 PATH 包含：

```text
C:\Users\<你的用户名>\.cargo\bin
```

当前 PowerShell 会话可临时刷新 PATH：

```powershell
$env:PATH="$env:USERPROFILE\.cargo\bin;$env:PATH"
```

## 安装依赖

在项目根目录运行：

```powershell
npm.cmd --prefix frontend install
```

## 前端构建

```powershell
npm.cmd run build
```

## Rust 检查

```powershell
cargo check --manifest-path src-tauri\Cargo.toml
```

## 启动开发版

```powershell
npm.cmd run tauri:dev
```

## 打包

打包需要在**项目根目录**运行，Tauri CLI 才能找到 `src-tauri/tauri.conf.json`：

```powershell
$env:PATH="$env:USERPROFILE\.cargo\bin;$env:PATH"
.\frontend\node_modules\.bin\tauri.cmd build
```

`beforeBuildCommand` 会自动先执行前端构建（`npm.cmd --prefix frontend run build`），无需手动构建前端。

成功后生成：

```text
src-tauri\target\release\ssh-port-forwarder.exe
src-tauri\target\release\bundle\nsis\SSH Port Forwarder_0.2.0_x64-setup.exe
```

## 发布 v0.2.0

建议发布内容：

- Git tag：`v0.2.0`
- Release title：`SSH Port Forwarder v0.2.0`
- Release asset：
  - `ssh-port-forwarder.exe`
  - `SSH Port Forwarder_0.2.0_x64-setup.exe`

发布说明可参考：

```text
v0.2.0 更新：
- 改为以主机为中心的两级结构：主机（连接参数）为一级目录，端口转发（监听/目标参数）为二级目录。
- 主机操作：发送指令、打开外部终端、上传密钥、新建端口转发。
- 新建/连接端口转发前检测端口占用，报错指出占用的转发或进程（含 PID）。
- 主页改为轻量监控板（当前连接 + 关键事件），完整日志移到独立页面。
- 默认改为浅色主题；设置项随界面语言切换；修复说明文档暗色显示问题；补充密钥文件填写说明。
- 建立连接时不再要求填写转发参数，转发参数只在新建端口转发界面填写。
- 后端拆分为 model/store/state/validate/portcheck/ssh/commands 等模块，前端拆分为页面与组件。
- 数据文件改为 hosts.json，旧 profiles.json 自动备份为 profiles.json.v0.1.bak。
- 启动时检测 OpenSSH 客户端，未安装时弹窗可一键安装（提权 Add-WindowsCapability）。

v0.1.2 更新：
- 连接时自动判断认证方式：能免密直连就直接连接，需要密码时自动弹窗输入。
- 检测到远程主机指纹变化时弹窗提示，由用户核对后决定是否信任新密钥并重试。

首次发布（v0.1.0）：
- 可视化管理 SSH 端口转发，支持 Local / Remote / Dynamic 模式与历史连接。
```

## 开源协议

本项目采用 MIT 协议，详见仓库根目录 [LICENSE](LICENSE)。

## 仓库

目标仓库：

```text
https://github.com/Trilives/Port_transfer_ssh_ui.git
```
