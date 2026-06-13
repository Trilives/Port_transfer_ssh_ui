import type { Language } from "./types";

export const languageLabels: Record<Language, string> = {
  "zh-CN": "中文（简体）",
  "en-US": "English",
};

const text = {
  "zh-CN": {
    title: "SSH 端口转发",
    subtitle: "以主机为中心，管理 SSH 连接、端口转发、终端与日志。",

    // 导航
    dashboard: "主页",
    config: "历史链接",
    logs: "日志",
    settings: "设置",
    guide: "使用说明",

    // 主页 / 监控板
    dashboardTitle: "监控板",
    dashboardDesc: "只显示当前连接与关键事件。详细配置在「配置」页，完整日志在「日志」页。",
    currentConnections: "当前连接",
    noConnections: "暂无运行中的转发。",
    keyEvents: "关键事件",
    noEvents: "暂无连接、断开或错误事件。",
    stopAll: "全部断开",
    new: "新建",
    host: "主机",

    // 配置页
    configTitle: "主机与端口转发",
    configDesc: "每台主机是一个一级目录，点击展开查看其下的端口转发（二级目录）。",
    newHost: "新建主机",
    editHost: "编辑主机",
    deleteHost: "删除主机",
    pin: "置顶",
    unpin: "取消置顶",
    pinned: "已置顶",

    // 导入 / 导出
    import: "导入",
    export: "导出",
    importFromFile: "从文件导入",
    importFromConfig: "从本机 SSH 配置导入",
    exportToFile: "导出到文件",
    exportToConfig: "导出到本机 SSH 配置",
    selectHostsToImport: "选择要导入的主机",
    selectHostsToExport: "选择要导出的主机",
    selectAll: "全选",
    clearAll: "全不选",
    confirmImport: "导入",
    confirmExport: "导出",
    noHostsToImport: "没有可导入的主机。",
    importConflictTitle: "存在重复的主机 IP",
    importConflictDesc: "以下主机的 IP 与现有主机重复，请选择处理方式：",
    exportConfigConflictDesc: "以下主机的 IP 与本机 SSH 配置中已有的条目重复，请选择处理方式：",
    overwriteAll: "全部覆盖",
    skipDuplicates: "仅导入不重复的",
    importDone: "导入完成：新增 {added}，覆盖 {overwritten}，跳过 {skipped}。",
    exportDone: "已导出到文件。",
    exportConfigDone: "已导出到本机 SSH 配置（~/.ssh/config）。",
    sendCommand: "发送指令",
    openTerminal: "打开终端",
    more: "更多",
    openInVscode: "通过 VS Code 打开",
    uploadKey: "上传密钥",

    // 通过 VS Code 打开
    vscodeHistoryTitle: "通过 VS Code 打开",
    vscodeHistoryDesc: "选择一个历史远端文件夹，或「直连」用 VS Code 默认方式连接，也可指定目录打开。",
    vscodeDirect: "直连（VS Code 默认连接，不打开文件夹）",
    vscodeNoHistory: "未找到该主机的 VS Code 历史连接。点击下方「直连」将用 VS Code 默认方式连接（不打开文件夹），并在 ~/.ssh/config 中添加该主机（若尚未存在）。",
    vscodeOpenPathLabel: "指定目录打开",
    vscodeOpenPathPlaceholder: "远端目录，如 /home/user/project（~ 或相对路径按家目录解析）",
    vscodeOpenPathButton: "打开",
    vscodeAddedToConfig: "已将该主机以别名「{alias}」写入 ~/.ssh/config。",
    vscodeMissingTitle: "未检测到 VS Code",
    vscodeMissingDesc: "未找到 VS Code（code 可执行文件）。请先安装 VS Code 后再使用此功能。",
    vscodeRemoteSshMissingTitle: "未安装 Remote-SSH 扩展",
    vscodeRemoteSshMissingDesc: "已检测到 VS Code，但未安装 Remote-SSH 扩展（ms-vscode-remote.remote-ssh）。请在 VS Code 中安装后再试。",
    newForward: "新建端口转发",
    noHosts: "还没有主机。点击「新建主机」开始。",
    noForwards: "该主机下还没有端口转发。",
    forwardsCount: "条转发",
    runningCount: "运行中",

    // 转发行
    connect: "连接",
    disconnect: "断开",
    openWeb: "在浏览器打开",
    edit: "编辑",
    delete: "删除",
    mode: "模式",
    bind: "监听",
    target: "目标",
    status: "状态",
    running: "运行中",
    stopped: "已停止",

    // 主机字段
    name: "名称",
    sshHost: "SSH 主机",
    sshPort: "SSH 端口",
    sshUser: "SSH 用户",
    identityFile: "私钥文件",
    identityFileHint:
      "私钥文件路径。留空则使用默认 %USERPROFILE%\\.ssh\\id_ed25519；可填绝对路径（如 C:\\Users\\you\\.ssh\\id_rsa）或 ~/.ssh/id_ed25519。请指向私钥本身，不要填 .pub 公钥文件。",
    extraOptions: "额外 SSH 参数",
    proxyJump: "跳板机（ProxyJump）",
    proxyJumpHint: "可不填。通过另一台主机跳转连接，形如 user@jump-host 或 user@jump-host:port；多级跳转用逗号分隔。",

    // 转发字段
    bindHost: "监听地址",
    bindPort: "本机访问端口",
    targetHost: "目标地址",
    targetPort: "远程待映射端口",
    keepConnected: "保持连接并自动重连",

    // 主机弹窗
    hostDialogTitle: "主机连接",
    hostDialogDesc: "只填 SSH 连接参数。端口转发在主机展开后单独新建。",
    // 转发弹窗
    forwardDialogTitle: "端口转发",
    forwardDialogDesc: "Local 访问远端内网服务，Remote 反向暴露本地服务，Dynamic 创建 SOCKS 代理。",
    save: "保存",
    cancel: "取消",

    // 发送指令弹窗
    sendCommandTitle: "发送指令",
    sendCommandDesc: "通过 SSH 在该主机上执行一条指令（依赖已配置的免密登录）。",
    commandPlaceholder: "例如：uptime",
    run: "执行",
    sendWithPassword: "使用密码发送",
    running2: "执行中…",
    output: "输出",

    // 密码弹窗
    passwordDialogTitle: "一次性密码连接",
    passwordDialogDescription: "请输入本次连接使用的 SSH 密码",
    passwordPlaceholder: "SSH 密码",
    passwordOnceNote: "密码不会保存到配置文件。若开启保持连接，密码只会在本次应用运行期间用于自动重连。",

    // 上传密钥弹窗
    keyUploadTitle: "上传 SSH 公钥",
    keyUploadDesc: "输入该主机的 SSH 密码，程序会把本机公钥写入远端 authorized_keys。",
    keyUploadNote: "如果未指定私钥文件，程序会使用或生成 %USERPROFILE%\\.ssh\\id_ed25519。密码不会保存。",
    keyUploadNotNeeded: "该主机已可免密直连，无需上传公钥。",
    detectingConnection: "正在检测连接方式…",

    // 指纹变化弹窗
    hostKeyChangedTitle: "远程主机指纹已改变",
    hostKeyChangedWarn:
      "远程主机的密钥与本地记录不一致，连接已被拒绝。这可能是因为服务器被重装或更换，也可能是中间人攻击。请在确认指纹安全后再决定。",
    hostKeyFingerprintLabel: "当前远程主机密钥指纹：",
    hostKeyFetching: "正在获取指纹…",
    hostKeyUnavailable: "无法获取远程指纹，请谨慎操作。",
    hostKeyTrust: "信任新密钥并重试",

    // 致命错误弹窗
    criticalTitle: "连接出现关键错误",
    criticalDesc: "已停止自动重试。请修复错误后手动重新连接。",
    close: "关闭",

    // 连接失败弹窗（探测阶段：不可达 / IP / 端口 / 网络等）
    connectFailedTitle: "无法建立连接",
    connectFailedDesc: "请根据下方原因检查主机连接参数后重试。",

    // 未安装 SSH 弹窗
    sshMissingTitle: "未检测到 OpenSSH 客户端",
    sshMissingDesc: "本程序依赖 Windows 自带的 OpenSSH 客户端（ssh.exe）。是否现在安装？安装需要管理员权限，会从 Windows Update 下载。",
    install: "安装",
    sshInstallStarted: "已开始安装 OpenSSH 客户端。请在弹出的窗口中完成安装，然后重启本程序。",

    // 删除主机二次确认
    confirmDeleteHostTitle: "删除主机",
    confirmDeleteHostDesc: "该主机下的所有端口转发都会一并删除，运行中的连接会先断开。此操作不可撤销。",

    // 日志页
    logsTitle: "运行日志",
    logsDesc: "按等级过滤显示，同时写入本地日志目录。等级在「设置」中调整。",
    message: "消息",
    time: "时间",
    level: "等级",

    // 设置页
    settingsTitle: "设置",
    settingsDesc: "主题、语言和日志等级会保存在本地，下次启动自动恢复。",
    theme: "主题",
    language: "语言",
    logLevel: "日志等级",
    themeLight: "浅色",
    themeDark: "深色",
    logDebug: "调试",
    logInfo: "信息",
    logWarning: "警告",
    logError: "错误",
  },
  "en-US": {
    title: "SSH Port Forwarder",
    subtitle: "Host-centric management of SSH connections, port forwards, terminals, and logs.",

    dashboard: "Home",
    config: "History",
    logs: "Logs",
    settings: "Settings",
    guide: "Guide",

    dashboardTitle: "Dashboard",
    dashboardDesc: "Shows only current connections and key events. Full config is on Config, full logs on Logs.",
    currentConnections: "Current Connections",
    noConnections: "No running forwards.",
    keyEvents: "Key Events",
    noEvents: "No connect, disconnect, or error events yet.",
    stopAll: "Stop All",
    new: "New",
    host: "Host",

    configTitle: "Hosts & Port Forwards",
    configDesc: "Each host is a top-level item; click to expand the port forwards beneath it.",
    newHost: "New Host",
    editHost: "Edit Host",
    deleteHost: "Delete Host",
    pin: "Pin",
    unpin: "Unpin",
    pinned: "Pinned",

    import: "Import",
    export: "Export",
    importFromFile: "Import from file",
    importFromConfig: "Import from SSH config",
    exportToFile: "Export to file",
    exportToConfig: "Export to SSH config",
    selectHostsToImport: "Select hosts to import",
    selectHostsToExport: "Select hosts to export",
    selectAll: "Select all",
    clearAll: "Clear all",
    confirmImport: "Import",
    confirmExport: "Export",
    noHostsToImport: "No hosts to import.",
    importConflictTitle: "Duplicate Host IPs",
    importConflictDesc: "The following hosts share an IP with existing hosts. Choose how to handle them:",
    exportConfigConflictDesc: "The following hosts share an IP with entries already in your local SSH config. Choose how to handle them:",
    overwriteAll: "Overwrite all",
    skipDuplicates: "Import non-duplicates only",
    importDone: "Import done: {added} added, {overwritten} overwritten, {skipped} skipped.",
    exportDone: "Exported to file.",
    exportConfigDone: "Exported to local SSH config (~/.ssh/config).",
    sendCommand: "Send Command",
    openTerminal: "Open Terminal",
    more: "More",
    openInVscode: "Open in VS Code",
    uploadKey: "Upload Key",

    // Open in VS Code
    vscodeHistoryTitle: "Open in VS Code",
    vscodeHistoryDesc: "Pick a recent remote folder, use \"Direct\" for VS Code's default connect, or open a specific directory.",
    vscodeDirect: "Direct (VS Code default connect, no folder)",
    vscodeNoHistory: "No VS Code history found for this host. \"Direct\" below connects the VS Code default way (no folder) and adds the host to ~/.ssh/config if it isn't there yet.",
    vscodeOpenPathLabel: "Open a specific directory",
    vscodeOpenPathPlaceholder: "Remote path, e.g. /home/user/project (~ or relative resolved against home)",
    vscodeOpenPathButton: "Open",
    vscodeAddedToConfig: "Added this host to ~/.ssh/config as alias \"{alias}\".",
    vscodeMissingTitle: "VS Code Not Found",
    vscodeMissingDesc: "Could not find VS Code (the code executable). Install VS Code first to use this feature.",
    vscodeRemoteSshMissingTitle: "Remote-SSH Extension Missing",
    vscodeRemoteSshMissingDesc: "VS Code was found, but the Remote-SSH extension (ms-vscode-remote.remote-ssh) is not installed. Install it in VS Code, then try again.",
    newForward: "New Forward",
    noHosts: "No hosts yet. Click \"New Host\" to start.",
    noForwards: "No port forwards under this host.",
    forwardsCount: "forwards",
    runningCount: "running",

    connect: "Connect",
    disconnect: "Disconnect",
    openWeb: "Open in browser",
    edit: "Edit",
    delete: "Delete",
    mode: "Mode",
    bind: "Bind",
    target: "Target",
    status: "Status",
    running: "Running",
    stopped: "Stopped",

    name: "Name",
    sshHost: "SSH Host",
    sshPort: "SSH Port",
    sshUser: "SSH User",
    identityFile: "Key File",
    identityFileHint:
      "Path to the private key. Leave empty to use the default %USERPROFILE%\\.ssh\\id_ed25519; or enter an absolute path (e.g. C:\\Users\\you\\.ssh\\id_rsa) or ~/.ssh/id_ed25519. Point to the private key itself, not the .pub file.",
    extraOptions: "Extra SSH Options",
    proxyJump: "Jump Host (ProxyJump)",
    proxyJumpHint: "Optional. Connect via another host, e.g. user@jump-host or user@jump-host:port; chain multiple with commas.",

    bindHost: "Bind Host",
    bindPort: "Local Access Port",
    targetHost: "Target Host",
    targetPort: "Remote Port to Map",
    keepConnected: "Keep connected and reconnect automatically",

    hostDialogTitle: "Host Connection",
    hostDialogDesc: "Only SSH connection parameters. Port forwards are created after expanding the host.",
    forwardDialogTitle: "Port Forward",
    forwardDialogDesc: "Local reaches remote services, Remote exposes local services, Dynamic creates a SOCKS proxy.",
    save: "Save",
    cancel: "Cancel",

    sendCommandTitle: "Send Command",
    sendCommandDesc: "Run a single command on this host over SSH (requires passwordless login).",
    commandPlaceholder: "e.g. uptime",
    run: "Run",
    sendWithPassword: "Send with password",
    running2: "Running…",
    output: "Output",

    passwordDialogTitle: "One-time password connection",
    passwordDialogDescription: "Enter the SSH password for this connection",
    passwordPlaceholder: "SSH password",
    passwordOnceNote:
      "The password is not saved to the config file. If keep connected is enabled, it is kept only in memory for reconnects during this app session.",

    keyUploadTitle: "Upload SSH Public Key",
    keyUploadDesc: "Enter the host's SSH password; the app appends your public key to remote authorized_keys.",
    keyUploadNote:
      "If no key file is specified, the app uses or creates %USERPROFILE%\\.ssh\\id_ed25519. The password is not saved.",
    keyUploadNotNeeded: "This host already supports passwordless login; no upload needed.",
    detectingConnection: "Checking connection…",

    hostKeyChangedTitle: "Remote Host Key Changed",
    hostKeyChangedWarn:
      "The remote host key does not match the local record, so the connection was refused. This may happen if the server was rebuilt or replaced — but it can also indicate a man-in-the-middle attack. Verify the fingerprint before deciding.",
    hostKeyFingerprintLabel: "Current remote host key fingerprint:",
    hostKeyFetching: "Fetching fingerprint…",
    hostKeyUnavailable: "Could not fetch the remote fingerprint; proceed with caution.",
    hostKeyTrust: "Trust New Key & Retry",

    criticalTitle: "Critical Connection Error",
    criticalDesc: "Automatic retries stopped. Fix the error, then reconnect manually.",
    close: "Close",

    connectFailedTitle: "Cannot Connect",
    connectFailedDesc: "Check the host connection settings based on the reason below, then retry.",

    sshMissingTitle: "OpenSSH Client Not Found",
    sshMissingDesc: "This app requires the built-in Windows OpenSSH client (ssh.exe). Install it now? Administrator rights are required and it downloads from Windows Update.",
    install: "Install",
    sshInstallStarted: "OpenSSH client installation started. Finish it in the elevated window, then restart this app.",

    confirmDeleteHostTitle: "Delete Host",
    confirmDeleteHostDesc: "All its port forwards will be removed and any running connections disconnected. This cannot be undone.",

    logsTitle: "Runtime Logs",
    logsDesc: "Filtered by level and written to the local log folder. Change the level in Settings.",
    message: "Message",
    time: "Time",
    level: "Level",

    settingsTitle: "Settings",
    settingsDesc: "Theme, language, and log level are saved locally and restored on next launch.",
    theme: "Theme",
    language: "Language",
    logLevel: "Log Level",
    themeLight: "Light",
    themeDark: "Dark",
    logDebug: "Debug",
    logInfo: "Info",
    logWarning: "Warning",
    logError: "Error",
  },
};

export function t(language: Language, key: keyof typeof text["zh-CN"]) {
  return text[language][key] ?? text["zh-CN"][key];
}
