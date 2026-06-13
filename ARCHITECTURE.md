# SSH Port Forwarder 架构文档

本文档描述 v0.2.0 的整体架构、数据模型、模块划分与关键流程，供开发与维护参考。
功能与上手见 [README.md](README.md)，开发环境与构建见 [TECHNICAL.md](TECHNICAL.md)，
何时/如何拆分模块见 [MODULARITY.md](MODULARITY.md)。

## 1. 概览

v0.2.0 把核心对象从「一条扁平的端口转发」改为「以主机为中心的两级结构」：

- **一级目录 = 主机（Host）**：一台 SSH 服务器，保存连接参数（主机、端口、用户、密钥）。
- **二级目录 = 端口转发（Forward）**：挂在某个主机下的一条转发，只保存转发参数（模式、监听、目标）。

主机是 UI 与数据上的「对象」，并不存在一条被复用的常驻 SSH 主连接（Windows 自带
OpenSSH 不支持 ControlMaster 多路复用）。真正常驻的是每条转发各自的 `ssh.exe` 进程；
「发送指令」「打开终端」「上传密钥」都是按需各起一次 `ssh` 调用，复用主机的连接参数。

```mermaid
flowchart TB
  subgraph FE[React 前端]
    Dash[主页/监控板] --> Cur[当前连接 + 关键事件]
    Cfg[配置页] --> HostCard --> ForwardRow
    HostCard --> Dlg[弹窗: Host/Forward/SendCmd/Password/KeyUpload/HostKey]
    LogsPage[日志页]
    SetPage[设置页]
    GuidePage[使用说明]
    api[api.ts] -. invoke / listen .- FE
  end
  subgraph BE[Tauri 2 + Rust]
    cmds[commands/*] --> model[model + store: hosts.json]
    cmds --> ssh[ssh: command/process/probe/keys]
    cmds --> port[portcheck]
    cmds --> term[terminal: 外部 PowerShell]
    ssh --> sshexe[(ssh.exe 转发子进程)]
    term --> ps[(powershell -NoExit ssh ...)]
  end
  api == Tauri IPC ==> cmds
  cmds -. emit log-entry / critical-error .-> api
```

## 2. 数据模型

```text
Host  (一级目录 = SSH 服务器)
├─ id: String
├─ name: String
├─ sshHost: String
├─ sshPort: String          // 默认 22
├─ sshUser: String
├─ identityFile: String     // 私钥文件路径，连接级；留空用默认 id_ed25519
├─ extraOptions: String     // 作用于该主机所有 ssh 操作的额外参数
├─ proxyJump: String        // 跳板机（ProxyJump），可空；连接时加 -J
├─ pinned: bool             // 置顶；列表排序时优先
├─ updatedAt: i64           // 最后修改时间（Unix 毫秒），列表按其从新到旧排序
└─ forwards: Vec<Forward>   // 二级目录，嵌套存储

Forward (二级目录 = 一条端口转发)
├─ id: String
├─ name: String
├─ mode: local | remote | dynamic
├─ bindHost: String         // 默认 127.0.0.1
├─ bindPort: String         // 监听端口
├─ targetHost: String       // 默认 127.0.0.1；dynamic 不需要
├─ targetPort: String       // dynamic 不需要
└─ keepConnected: bool      // 默认 true，断开后自动重连
```

- 持久化文件：`hosts.json`（嵌套 forwards），位于系统应用数据目录。
- v0.1.x 的 `profiles.json` **不迁移**；若存在则启动时备份为 `profiles.json.v0.1.bak`。
- 运行态：
  - `hosts: Mutex<Vec<Host>>`
  - `tunnels: Mutex<HashMap<ForwardId, ManagedTunnel>>`（每条运行中的转发一个子进程，
    记录父主机快照用于自动重连）
  - `settings`、`logs` 同前。

## 3. 模块划分

### 后端 `src-tauri/src/`

```text
main.rs            入口 + askpass helper
lib.rs             run()、Tauri Builder、命令注册、AppState 定义与装配
model.rs           Host / Forward / 枚举 / AppSettings / LogEntry / View / Default
store.rs           read_json / write_json、数据目录路径、AppState 持久化与日志写盘
validate.rs        validate_host / validate_forward
portcheck.rs       本机端口占用检测 + 占用进程名/PID（netstat + tasklist）
sshconfig.rs       解析/写入 ~/.ssh/config（导入主机、导出到托管区块、按 IP 查别名/追加条目）
vscode.rs          VS Code Remote-SSH：检测安装、读 storage.json 历史连接、按 IP 匹配、打开 folder-uri
ssh/
  mod.rs           子模块再导出
  command.rs       build_ssh / build_probe / build_key_upload / build_send 命令构造
  diagnose.rs      classify_ssh_failure：按 stderr 归类不可达/认证等失败并给本地化原因
  process.rs       start/stop/watch/cleanup tunnel、CREATE_NO_WINDOW、askpass helper
  probe.rs         probe_connection / get_host_fingerprint / remove_known_host
  keys.rs          ensure_public_key / resolve_identity_file / upload_key_to_remote
terminal.rs        open_terminal：起外部 PowerShell 窗口运行 ssh
commands/
  mod.rs
  hosts.rs         list_hosts / save_host / delete_host
  forwards.rs      save_forward / delete_forward / connect_* / disconnect_*
  exec.rs          send_command / open_terminal
  keys.rs          upload_public_key / probe_connection / get_host_fingerprint / remove_known_host
  settings.rs      get_settings / save_settings_cmd / list_logs
  transfer.rs      导入/导出主机（文件 + ~/.ssh/config）；rfd 原生文件对话框
  vscode.rs        vscode_status / vscode_ssh_history / vscode_open / vscode_open_direct / vscode_open_path
```

### 前端 `frontend/src/`

```text
App.tsx              壳：导航、页面路由、数据加载/刷新、所有动作与弹窗编排、主题应用
types.ts  api.ts  i18n.ts
pages/
  DashboardPage.tsx  主页监控板：完整当前连接 + 关键事件摘要（连接/断开/错误，限量）
  ConfigPage.tsx     配置：主机列表(一级)展开 + 转发(二级) + 全部增删改与操作入口
  LogsPage.tsx       完整日志表，按等级过滤
  SettingsPage.tsx   主题/语言/日志等级；选项随语言切换，默认 light
  GuidePage.tsx      使用说明；暗色适配；含密钥字段说明
components/
  HostCard.tsx       一级：点击展开/收起，状态，按钮[发送指令|打开终端▾(含 VS Code 打开)|上传密钥|新建端口转发]
  ForwardRow.tsx     二级：状态 + [连接|断开|编辑|删除]
  StatusBadge.tsx    运行/停止状态徽标
  LogTable.tsx       日志表
  dialogs.tsx        Host/Forward/SendCommand/Password/KeyUpload/HostKeyChanged/ConnectionError/CriticalError/SelectHosts/ImportConflict/VscodeHistory/VscodeMissing 弹窗
  ui/ button card input select
```

数据加载、刷新轮询、连接/上传/指纹等流程与所有弹窗状态集中在 `App.tsx` 编排，
页面与组件保持纯展示（通过 props 接收数据与回调）。

## 4. Tauri 命令

| 域 | 命令 | 说明 |
| --- | --- | --- |
| 主机 | `list_hosts -> [HostView]` | 含每条转发的运行状态；置顶优先、其余按修改时间从新到旧排序 |
| | `save_host(host) -> HostView` | 不校验参数；可用性留到连接运行时判断；保存即刷新修改时间 |
| | `set_host_pinned(id, pinned) -> HostView` | 置顶/取消置顶；不改修改时间 |
| | `delete_host(id)` | 先断开其下所有运行中的转发 |
| 导入导出 | `read_import_file -> [Host]` / `read_import_ssh_config -> [Host]` | 选文件/读 ~/.ssh/config 解析为主机供前端勾选 |
| | `import_hosts(hosts, strategy) -> ImportResult` | 按 sshHost 去重；strategy=""探测冲突、overwrite、skip |
| | `export_hosts_to_file(hostIds) -> bool` | 选文件导出完整主机（含转发）|
| | `export_hosts_to_ssh_config(hostIds)` | 写 ~/.ssh/config 托管区块（仅 ssh 可解析字段）|
| 转发 | `save_forward(hostId, forward) -> HostView` | 不校验参数、不查端口占用；均留到连接时判断 |
| | `delete_forward(hostId, forwardId)` | |
| | `connect_forward(hostId, forwardId)` | 复用父主机参数；先 probe |
| | `connect_forward_with_password(hostId, forwardId, password)` | |
| | `disconnect_forward(hostId, forwardId)` / `disconnect_host(hostId)` / `disconnect_all()` | |
| 指令/终端 | `send_command(hostId, command) -> String` | `ssh … user@host "command"`，返回输出 |
| | `open_terminal(hostId)` | 起外部 PowerShell 窗口运行交互式 `ssh` |
| 密钥/探测 | `upload_public_key(hostId, password)` | 主机级；上传前已在前端 probe |
| | `probe_connection(hostId) -> ready｜password_required｜host_key_changed` | 不可达/IP/端口/网络等错误归为带原因的 Err，不当作需要密码 |
| | `get_host_fingerprint(hostId)` / `remove_known_host(hostId)` | |
| 设置/日志 | `get_settings` / `save_settings_cmd(settings)` / `list_logs(level)` | 默认主题 light |
| 系统环境 | `check_ssh -> bool` | 检测 `ssh.exe` 是否可用 |
| | `install_openssh()` | 提权 PowerShell 运行 `Add-WindowsCapability` 安装 OpenSSH 客户端 |
| VS Code | `vscode_status -> {installed, remoteSsh}` | 检测 Code.exe 与 Remote-SSH 扩展 |
| | `vscode_ssh_history(hostId) -> [{uri, path}]` | 按主机 IP 匹配 storage.json 里的远端历史文件夹 |
| | `vscode_open(uri)` | `code --folder-uri <uri>` 重开历史文件夹 |
| | `vscode_open_direct(hostId) -> {addedToConfig, alias}` | 直连：必要时写入 ~/.ssh/config，`code --remote ssh-remote+<别名>` 打开不带文件夹的已连接窗口 |
| | `vscode_open_path(hostId, path) -> {addedToConfig, alias}` | 打开指定远端目录；绝对路径原样，`~`/相对路径探测 `$HOME` 后解析 |

事件：

- `log-entry`：单条日志推送。
- `critical-error`：转发退出码 255 时推送，载荷含 `hostId / forwardId / name / message`，前端弹窗一次并停止自动重连。

## 5. 关键流程

### 端口冲突检测（local / dynamic 监听本机端口时）

1. 应用内：bindPort 是否已被本应用另一条运行中的转发占用 → 报错指出是哪条转发。
2. 操作系统：`netstat -ano -p tcp` 找 `LISTENING` 且端口匹配的 PID，
   再用 `tasklist /FI "PID eq <pid>" /FO CSV /NH` 取进程名（均 `CREATE_NO_WINDOW`）。

报错示例：`端口 8000 已被进程 chrome.exe (PID 1234) 占用`。
remote 模式监听在远端，本机不检查。

### 转发连接 / 自动重连

`connect_forward` 取父主机参数 + 转发参数构造 `ssh -N -T -L/-R/-D …`，以 `CREATE_NO_WINDOW`
后台启动，登记进 `tunnels`。后台线程 `watch_tunnel` 轮询子进程：

- 正常退出且开启 keepConnected → 延时重连。
- 退出码 255（致命错误）→ 记 error 日志 + 发 `critical-error`，停止重连。
- 用户主动断开 / 关闭 keepConnected → 不重连。

应用退出（`RunEvent::ExitRequested`）时清理所有由本程序启动的转发子进程。

编辑配置时的自动重连（`restart_forward`：复用暂存密码，先停后起）：
- `save_host`：主机 IP 或用户变化 → 重启该主机下所有运行中的转发。
- `save_forward`：某条转发的模式/监听/目标 ip 或端口变化且正在运行 → 仅重连该条。

### 一次性密码连接

前端先 `probe_connection`：

- `ready` → 直接 `connect_forward`。
- `password_required` → 弹密码框 → `connect_forward_with_password`，通过 `SSH_ASKPASS`
  + askpass helper 注入一次性密码；密码不写盘，仅在开启 keepConnected 时留内存用于本次会话重连。
- `host_key_changed` → 取指纹弹窗，用户确认后 `remove_known_host` 再重试。

### 外部终端

`open_terminal` 起一个可见的 PowerShell 窗口运行 `ssh [opts] user@host`，提供完整交互式会话
（Tab 补全由远端 shell 完成）。该窗口独立于应用，不随应用退出而被清理。

### 通过 VS Code 打开（Remote-SSH）

「打开终端」按钮右侧下拉提供「通过 VS Code 打开」。流程：

1. `vscode_status`：找不到 Code.exe → 提示未安装；找到但无 Remote-SSH 扩展 → 提示装扩展。
2. `vscode_ssh_history(hostId)`：读 `%APPDATA%\Code\User\globalStorage\storage.json` 的
   `profileAssociations.workspaces`，key 为 `vscode-remote://ssh-remote+<authority>/<path>`。
   authority 为裸 IP，或 hex 的 `{"hostName":"别名"}`；别名经 `~/.ssh/config` 的 `HostName`
   解析回 IP，与主机 IP 比对，列出命中的远端文件夹。
3. 弹窗第一项「直连」→ `vscode_open_direct`：在 config 找该 IP 的别名，没有就以主机名（冲突时退回
   IP）追加一条 `Host` 写入 config 并提示；再 `code --remote ssh-remote+<别名>` 用 VS Code 默认方式
   打开不带文件夹的已连接窗口。历史项 → `vscode_open` 原样重开该历史 URI。弹窗底部「指定目录打开」
   → `vscode_open_path`：绝对路径原样打开，`~`/相对路径用一次性免密 SSH 探测 `$HOME` 后解析。
   无历史时仅显示「直连」与「指定目录」。

## 6. 数据存储

后端用 `directories::ProjectDirs` 取系统应用数据目录，写入 `hosts.json`、`settings.json`
和 `logs/` 日志文件，不写源码目录或程序所在目录。
