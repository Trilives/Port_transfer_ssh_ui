import { type ReactNode } from "react";
import { CheckCircle2 } from "lucide-react";
import { Card, CardDescription, CardHeader, CardTitle } from "../components/ui/card";
import type { Language } from "../types";

export function GuidePage(props: { language: Language }) {
  const isZh = props.language === "zh-CN";

  const quickSteps = isZh
    ? [
        ["1", "新建主机", "在「配置」页点击「新建主机」，只填 SSH 主机、用户、端口和密钥文件。"],
        ["2", "新建端口转发", "展开主机卡片，点击「新建端口转发」，填写监听与目标参数。"],
        ["3", "连接", "在转发上点击「连接」，程序自动判断免密或弹密码框，运行状态在「主页」查看。"],
      ]
    : [
        ["1", "New Host", "On Config, click \"New Host\" and fill only SSH host, user, port, and key file."],
        ["2", "New Forward", "Expand the host card and click \"New Forward\" to fill bind and target parameters."],
        ["3", "Connect", "Click Connect on a forward; the app auto-detects key auth or prompts for a password. Watch status on Home."],
      ];

  const exampleRows = isZh
    ? [
        ["目标", "远程服务器上的 127.0.0.1:8000，本地浏览器也能打开"],
        ["模式", "local"],
        ["监听地址", "127.0.0.1"],
        ["监听端口", "8000"],
        ["目标地址", "127.0.0.1"],
        ["目标端口", "8000"],
        ["打开方式", "连接成功后访问 http://127.0.0.1:8000"],
      ]
    : [
        ["Goal", "Open 127.0.0.1:8000 on the remote server from your local browser"],
        ["Mode", "local"],
        ["Bind Host", "127.0.0.1"],
        ["Bind Port", "8000"],
        ["Target Host", "127.0.0.1"],
        ["Target Port", "8000"],
        ["Open", "Visit http://127.0.0.1:8000 after connecting"],
      ];

  const pageCards = isZh
    ? [
        ["主页", "监控板：完整展示当前运行中的转发，以及连接/断开/错误等关键事件。只读，不在这里改配置。"],
        ["配置", "主机（一级）与端口转发（二级）的管理中心。新建/编辑/删除、连接/断开、发送指令、终端、上传密钥都在这里。"],
        ["日志", "完整运行日志，按设置中的等级过滤。排查连接失败时把等级调到 debug。"],
        ["设置", "主题（浅色/深色）、语言、日志等级。选项会随界面语言切换，默认浅色。"],
      ]
    : [
        ["Home", "Dashboard: shows all running forwards plus key connect/disconnect/error events. Read-only."],
        ["Config", "The hub for hosts (level 1) and forwards (level 2): create/edit/delete, connect, send command, terminal, upload key."],
        ["Logs", "Full runtime logs filtered by the level in Settings. Use debug to troubleshoot connection failures."],
        ["Settings", "Theme (light/dark), language, log level. Options follow the UI language; default is light."],
      ];

  const modeCards = isZh
    ? [
        ["Local", "本地端口 → SSH 服务器能访问到的目标服务。最常用，适合远程数据库、内网 Web 服务。"],
        ["Remote", "远程端口 → 本机服务。适合临时把本地开发服务暴露给服务器侧访问。"],
        ["Dynamic", "创建 SOCKS 代理。通常只需要监听地址和监听端口。"],
      ]
    : [
        ["Local", "Local port → a service reachable from the SSH server. The most common mode."],
        ["Remote", "Remote port → a local service. Useful when the server side needs to reach your dev service."],
        ["Dynamic", "Create a SOCKS proxy. Usually only bind host and bind port are required."],
      ];

  const hostOps = isZh
    ? [
        ["发送指令", "通过 SSH 在主机上执行一条指令并把输出显示在弹窗里。依赖已配置的免密登录。"],
        ["打开终端", "弹出一个外部 PowerShell 窗口，已经用 ssh 连上服务器，支持 Tab 自动补全。"],
        ["上传密钥", "把本机公钥写入远端 authorized_keys，配置免密登录；上传前会先检测是否已可免密。"],
      ]
    : [
        ["Send Command", "Run a single command on the host over SSH and show the output in a dialog. Requires passwordless login."],
        ["Open Terminal", "Pop up an external PowerShell window already connected via ssh, with Tab completion."],
        ["Upload Key", "Append your public key to remote authorized_keys; it first checks whether passwordless login already works."],
      ];

  const keyFileNote = isZh
    ? "私钥文件：留空则使用默认 %USERPROFILE%\\.ssh\\id_ed25519；也可以填绝对路径（如 C:\\Users\\you\\.ssh\\id_rsa）或 ~/.ssh/id_ed25519。请指向私钥本身，不要填 .pub 公钥文件。如果该路径下没有私钥，「上传密钥」会自动生成一对 ed25519 密钥。"
    : "Key file: leave empty to use the default %USERPROFILE%\\.ssh\\id_ed25519; or enter an absolute path (e.g. C:\\Users\\you\\.ssh\\id_rsa) or ~/.ssh/id_ed25519. Point to the private key itself, not the .pub file. If no key exists there, Upload Key will generate an ed25519 pair.";

  const tips = isZh
    ? [
        "连接时若本机端口已被占用，会自动顺延（+1）到第一个空闲端口并以它监听；主页与「网页打开」显示实际端口。",
        "连接出现关键错误（退出码 255）时会弹窗提示一次，并停止该转发的自动重连。",
        "首次连接新主机使用 OpenSSH accept-new 策略；指纹变化时连接被拒绝并弹窗，核对后再信任。",
        "关闭应用时，程序会自动清理由它启动的 SSH 转发进程；外部终端窗口不受影响。",
      ]
    : [
        "On connect, if the local port is already in use it auto-increments to the first free port and listens there; the Home page and \"Open in browser\" show the actual port.",
        "On a critical error (exit code 255) a dialog shows once and automatic reconnect for that forward stops.",
        "First-time hosts use OpenSSH accept-new; if a host key changes the connection is refused and a dialog asks you to verify it.",
        "When the app exits, SSH forwards it started are cleaned up; external terminal windows are unaffected.",
      ];

  return (
    <div className="grid gap-5">
      <Card>
        <CardHeader>
          <CardTitle>{isZh ? "使用说明" : "Guide"}</CardTitle>
          <CardDescription>
            {isZh
              ? "按实际使用顺序整理：先建主机，再在主机下建端口转发，最后连接并在主页与日志中观察。"
              : "Organized by the real workflow: create a host, add forwards under it, then connect and watch Home and Logs."}
          </CardDescription>
        </CardHeader>
        <div className="grid grid-cols-3 gap-4">
          {quickSteps.map(([step, title, body]) => (
            <div key={step} className="rounded-2xl border border-slate-200 bg-slate-50 p-4 dark:border-slate-800 dark:bg-slate-900">
              <div className="flex h-8 w-8 items-center justify-center rounded-full bg-blue-600 text-sm font-semibold text-white">{step}</div>
              <div className="mt-3 text-sm font-semibold text-slate-950 dark:text-slate-50">{title}</div>
              <p className="mt-2 text-sm leading-6 text-slate-500 dark:text-slate-400">{body}</p>
            </div>
          ))}
        </div>
      </Card>

      <GuideTable
        title={isZh ? "常用样例：本地打开远程 127.0.0.1:8000" : "Common Example: Open Remote 127.0.0.1:8000 Locally"}
        description={
          isZh
            ? "这是 Local 转发的典型场景。远程服务只监听服务器自己的 127.0.0.1，但你可以通过 SSH 隧道在本地浏览器访问。"
            : "The typical Local forwarding case. The remote service only listens on the server's own 127.0.0.1, but your local browser can reach it through the SSH tunnel."
        }
        rows={exampleRows}
      />

      <div className="grid grid-cols-2 gap-5">
        <GuidePanel title={isZh ? "每个页面怎么用" : "Pages"}>
          {pageCards.map(([title, body]) => (
            <GuideLine key={title} title={title} body={body} />
          ))}
        </GuidePanel>
        <GuidePanel title={isZh ? "转发模式怎么选" : "Forwarding Modes"}>
          {modeCards.map(([title, body]) => (
            <GuideLine key={title} title={title} body={body} />
          ))}
        </GuidePanel>
      </div>

      <div className="grid grid-cols-2 gap-5">
        <GuidePanel title={isZh ? "主机操作" : "Host Actions"}>
          {hostOps.map(([title, body]) => (
            <GuideLine key={title} title={title} body={body} />
          ))}
        </GuidePanel>
        <GuidePanel title={isZh ? "密钥文件怎么填" : "How to Fill the Key File"}>
          <div className="rounded-2xl bg-slate-50 p-4 text-sm leading-6 text-slate-600 dark:bg-slate-900 dark:text-slate-300">
            {keyFileNote}
          </div>
        </GuidePanel>
      </div>

      <GuidePanel title={isZh ? "排错与注意事项" : "Troubleshooting"}>
        <div className="grid gap-3">
          {tips.map((tip) => (
            <div key={tip} className="flex gap-3 rounded-2xl bg-slate-50 p-4 text-sm leading-6 text-slate-600 dark:bg-slate-900 dark:text-slate-300">
              <CheckCircle2 className="mt-0.5 shrink-0 text-blue-600" size={17} />
              <span>{tip}</span>
            </div>
          ))}
        </div>
      </GuidePanel>
    </div>
  );
}

function GuideTable(props: { title: string; description: string; rows: string[][] }) {
  return (
    <Card>
      <div className="text-base font-semibold text-slate-950 dark:text-slate-50">{props.title}</div>
      <p className="mt-2 text-sm leading-6 text-slate-500 dark:text-slate-400">{props.description}</p>
      <div className="mt-4 overflow-hidden rounded-xl border border-slate-200 dark:border-slate-800">
        {props.rows.map(([label, value]) => (
          <div key={`${label}-${value}`} className="grid grid-cols-[120px_1fr] border-b border-slate-100 last:border-b-0 dark:border-slate-800">
            <div className="bg-blue-50 px-3 py-2 text-sm font-medium text-blue-700 dark:bg-blue-950/40 dark:text-blue-300">{label}</div>
            <div className="px-3 py-2 text-sm text-slate-600 dark:text-slate-300">{value}</div>
          </div>
        ))}
      </div>
    </Card>
  );
}

function GuidePanel(props: { title: string; children: ReactNode }) {
  return (
    <Card>
      <CardHeader>
        <CardTitle>{props.title}</CardTitle>
      </CardHeader>
      <div className="grid gap-3">{props.children}</div>
    </Card>
  );
}

function GuideLine(props: { title: string; body: string }) {
  return (
    <div className="rounded-2xl border border-slate-200 bg-white p-4 dark:border-slate-800 dark:bg-slate-950">
      <div className="text-sm font-semibold text-slate-950 dark:text-slate-50">{props.title}</div>
      <p className="mt-2 text-sm leading-6 text-slate-500 dark:text-slate-400">{props.body}</p>
    </div>
  );
}
