# SSH Port Forwarder 技术文档

本文档面向开发者，记录当前项目架构、开发环境、构建方式和发布流程。

## 技术栈

- 桌面应用壳与本机后端：Tauri 2 + Rust
- 前端界面：React + TypeScript + Tailwind CSS
- UI 风格：本地 shadcn/ui 风格组件
- 转发执行：Rust 调用系统 `ssh.exe`
- 打包目标：Windows exe 和 NSIS 安装包

## 项目结构

```text
frontend/                 React + TypeScript + Tailwind UI
frontend/src/App.tsx      主界面、连接管理、设置、说明页和弹窗
frontend/src/api.ts       Tauri command 调用封装
frontend/src/components   本地 UI 基础组件
frontend/src/i18n.ts      中文/英文界面文案
src-tauri/                Tauri + Rust 后端
src-tauri/src/lib.rs      配置、日志、SSH 进程、Tauri commands
src-tauri/src/main.rs     程序入口和 askpass helper 入口
```

旧的 Python/Tkinter 原型和 uv 环境已经移除。

## 后端行为

Rust 后端负责：

- 读取和保存历史连接。
- 读取和保存主题、语言、日志等级等设置。
- 调用系统 `ssh.exe` 启动 Local / Remote / Dynamic 转发。
- 管理 SSH 子进程的连接、断开、全部断开和自动重连。
- 通过 `SSH_ASKPASS` 实现一次性密码连接。
- 在 Windows 下使用 `CREATE_NO_WINDOW` 启动 SSH 子进程，避免连接时弹出终端窗口。
- 在应用退出时清理所有由本程序启动的 SSH 转发进程。

## 数据存储

后端使用 `directories::ProjectDirs` 获取系统应用数据目录。历史连接、设置和日志写入用户应用数据目录，而不是源码目录。

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
cd src-tauri
cargo check
```

## 启动开发版

```powershell
npm.cmd run tauri:dev
```

## 打包

```powershell
$env:PATH="$env:USERPROFILE\.cargo\bin;$env:PATH"
npm.cmd run tauri:build
```

成功后生成：

```text
src-tauri\target\release\ssh-port-forwarder.exe
src-tauri\target\release\bundle\nsis\SSH Port Forwarder_0.1.1_x64-setup.exe
```

## 发布 v0.1.1

建议发布内容：

- Git tag：`v0.1.1`
- Release title：`SSH Port Forwarder v0.1.1`
- Release asset：
  - `ssh-port-forwarder.exe`
  - `SSH Port Forwarder_0.1.1_x64-setup.exe`

发布说明可参考：

```text
首次发布：
- 可视化管理 SSH 端口转发。
- 支持历史连接、连接管理、配置弹窗和主页面快捷操作。
- 支持 Local / Remote / Dynamic 转发模式。
- 支持一次性密码连接、保持连接、日志等级、主题和语言设置。
- Windows 下隐藏 SSH 子进程终端窗口。
- 应用退出时自动清理当前转发连接。
```

## 仓库

目标仓库：

```text
https://github.com/Trilives/Port_transfer_ssh_ui.git
```
