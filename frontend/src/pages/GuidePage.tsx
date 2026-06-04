import { CheckCircle2 } from "lucide-react";
import { Card, CardDescription, CardHeader, CardTitle } from "../components/ui/card";
import type { Language } from "../types";
import type { ReactNode } from "react";

export function GuidePage(props: { language: Language }) {
  const isZh = props.language === "zh-CN";
  const quickSteps = isZh
    ? [
        ["1", "\u65b0\u5efa\u8fde\u63a5", "\u8fdb\u5165\u8fde\u63a5\u7ba1\u7406\u6216\u4e3b\u9875\u9762\uff0c\u70b9\u51fb\u65b0\u5efa\uff0c\u6253\u5f00\u914d\u7f6e\u5f39\u7a97\u3002"],
        ["2", "\u586b\u5199\u53c2\u6570", "\u9009\u62e9 Local/Remote/Dynamic\uff0c\u586b SSH \u4e3b\u673a\u3001\u7528\u6237\u3001\u76d1\u542c\u7aef\u53e3\u548c\u76ee\u6807\u5730\u5740\u3002"],
        ["3", "\u5f00\u59cb\u8fde\u63a5", "\u70b9\u51fb\u8fde\u63a5\u540e\u4f1a\u4fdd\u5b58\u914d\u7f6e\u3001\u5173\u95ed\u5f39\u7a97\uff0c\u5e76\u56de\u5230\u4e3b\u9875\u9762\u67e5\u770b\u72b6\u6001\u548c\u65e5\u5fd7\u3002"],
      ]
    : [
        ["1", "Create", "Open Connection Management or Main, then click New to open the profile popup."],
        ["2", "Fill", "Choose Local/Remote/Dynamic, then fill SSH host, user, bind port, and target."],
        ["3", "Connect", "Connect saves the profile, closes the popup, and returns to Main for status and logs."],
      ];

  const remoteLocalRows = isZh
    ? [
        ["\u76ee\u6807", "\u8fdc\u7a0b\u670d\u52a1\u5668\u4e0a\u7684 127.0.0.1:8000\uff0c\u672c\u5730\u6d4f\u89c8\u5668\u4e5f\u80fd\u6253\u5f00"],
        ["\u6a21\u5f0f", "local"],
        ["SSH \u4e3b\u673a", "\u670d\u52a1\u5668 IP \u6216\u57df\u540d\uff0c\u4f8b\u5982 203.0.113.10"],
        ["SSH \u7aef\u53e3", "22"],
        ["SSH \u7528\u6237", "root / ubuntu / deploy"],
        ["\u76d1\u542c\u5730\u5740", "127.0.0.1"],
        ["\u76d1\u542c\u7aef\u53e3", "8000"],
        ["\u76ee\u6807\u5730\u5740", "127.0.0.1"],
        ["\u76ee\u6807\u7aef\u53e3", "8000"],
        ["\u6253\u5f00\u65b9\u5f0f", "\u8fde\u63a5\u6210\u529f\u540e\u8bbf\u95ee http://127.0.0.1:8000"],
      ]
    : [
        ["Goal", "Open 127.0.0.1:8000 on the remote server from your local browser"],
        ["Mode", "local"],
        ["SSH Host", "Server IP or domain, for example 203.0.113.10"],
        ["SSH Port", "22"],
        ["SSH User", "root / ubuntu / deploy"],
        ["Bind Host", "127.0.0.1"],
        ["Bind Port", "8000"],
        ["Target Host", "127.0.0.1"],
        ["Target Port", "8000"],
        ["Open", "Visit http://127.0.0.1:8000 after connecting"],
      ];

  const pageCards = isZh
    ? [
        ["\u4e3b\u9875\u9762", "\u67e5\u770b\u5f53\u524d\u8fde\u63a5\u3001\u8fd0\u884c\u72b6\u6001\u548c\u65e5\u5fd7\u3002\u70b9\u51fb\u4e00\u6761\u8bb0\u5f55\u53ef\u9009\u4e2d\uff0c\u518d\u70b9\u4e00\u6b21\u53ef\u53d6\u6d88\u9009\u4e2d\u3002"],
        ["\u8fde\u63a5\u7ba1\u7406", "\u7a0b\u5e8f\u9ed8\u8ba4\u6253\u5f00\u6b64\u9875\u3002\u65b0\u5efa\u3001\u914d\u7f6e\u3001\u8fde\u63a5\u3001\u4e0a\u4f20\u5bc6\u94a5\u548c\u5220\u9664\u90fd\u5728\u8fd9\u91cc\u3002"],
        ["\u914d\u7f6e\u5f39\u7a97", "\u586b\u5199\u6216\u4fee\u6539 SSH \u4e0e\u7aef\u53e3\u8f6c\u53d1\u53c2\u6570\u3002\u70b9\u51fb\u8fde\u63a5\u4f1a\u81ea\u52a8\u4fdd\u5b58\u5e76\u56de\u5230\u4e3b\u9875\u9762\u3002"],
        ["\u8bbe\u7f6e", "\u5207\u6362 dark/light \u4e3b\u9898\u3001\u754c\u9762\u8bed\u8a00\u548c\u65e5\u5fd7\u7b49\u7ea7\u3002Debug \u9002\u5408\u6392\u67e5\u8fde\u63a5\u5931\u8d25\u3002"],
      ]
    : [
        ["Main", "Inspect current connections, status, and logs. Click a row to select it, then click again to clear selection."],
        ["Connection Management", "The app opens here by default. Create, configure, connect, upload keys, and delete saved profiles here."],
        ["Profile Popup", "Fill or edit SSH and forwarding parameters. Connect saves the profile and returns to Main."],
        ["Settings", "Switch dark/light theme, language, and log level. Debug is useful for connection troubleshooting."],
      ];

  const modeCards = isZh
    ? [
        ["Local", "\u672c\u5730\u7aef\u53e3 -> SSH \u670d\u52a1\u5668\u80fd\u8bbf\u95ee\u5230\u7684\u76ee\u6807\u670d\u52a1\u3002\u6700\u5e38\u7528\u3002"],
        ["Remote", "\u8fdc\u7a0b\u7aef\u53e3 -> \u672c\u673a\u670d\u52a1\u3002\u9002\u5408\u4e34\u65f6\u628a\u672c\u5730\u5f00\u53d1\u670d\u52a1\u66b4\u9732\u7ed9\u670d\u52a1\u5668\u4fa7\u8bbf\u95ee\u3002"],
        ["Dynamic", "\u521b\u5efa SOCKS \u4ee3\u7406\u3002\u901a\u5e38\u53ea\u9700\u8981\u76d1\u542c\u5730\u5740\u548c\u76d1\u542c\u7aef\u53e3\u3002"],
      ]
    : [
        ["Local", "Local port -> a service reachable from the SSH server. This is the most common mode."],
        ["Remote", "Remote port -> a local service. Useful when the server side needs to reach your development service."],
        ["Dynamic", "Create a SOCKS proxy. Usually only bind host and bind port are required."],
      ];

  const tips = isZh
    ? [
        "\u9996\u6b21\u8fde\u63a5\u4e3b\u673a\u65f6\uff0c\u7a0b\u5e8f\u4f7f\u7528 OpenSSH \u7684 accept-new \u7b56\u7565\uff1a\u9996\u6b21\u6307\u7eb9\u4f1a\u81ea\u52a8\u52a0\u5165 known_hosts\uff0c\u4f46\u6307\u7eb9\u53d8\u5316\u4f1a\u88ab\u62d2\u7edd\u3002",
        "\u5982\u679c\u9700\u8981\u5148\u914d\u7f6e\u5bc6\u94a5\u767b\u5f55\uff0c\u70b9\u51fb\u4e0a\u4f20\u5bc6\u94a5\uff0c\u8f93\u5165\u4e00\u6b21 SSH \u5bc6\u7801\u5373\u53ef\u5199\u5165\u8fdc\u7aef authorized_keys\u3002",
        "\u8fde\u63a5\u51fa\u73b0\u5173\u952e\u9519\u8bef\u65f6\u4f1a\u5f39\u7a97\u63d0\u793a\u4e00\u6b21\uff0c\u8be5\u8fde\u63a5\u4f1a\u505c\u6b62\u81ea\u52a8\u91cd\u8bd5\u3002",
        "\u5173\u95ed\u5e94\u7528\u65f6\uff0c\u7a0b\u5e8f\u4f1a\u81ea\u52a8\u6e05\u7406\u7531\u5b83\u542f\u52a8\u7684 SSH \u8f6c\u53d1\u8fdb\u7a0b\u3002",
      ]
    : [
        "For first-time hosts, the app uses OpenSSH accept-new: new host keys are added automatically, but changed host keys are rejected.",
        "Use Upload Key with a one-time SSH password to append your public key to remote authorized_keys.",
        "Critical connection errors show one dismissible dialog and stop automatic retries for that tunnel.",
        "When the app exits, SSH forwarding processes started by the app are cleaned up automatically.",
      ];

  return (
    <div className="grid gap-5">
      <Card>
        <CardHeader>
          <CardTitle>{isZh ? "\u4f7f\u7528\u8bf4\u660e" : "Guide"}</CardTitle>
          <CardDescription>
            {isZh
              ? "\u6309\u5b9e\u9645\u4f7f\u7528\u987a\u5e8f\u6574\u7406\uff1a\u5148\u521b\u5efa\u8fde\u63a5\uff0c\u518d\u53c2\u8003\u6837\u4f8b\u586b\u5199\uff0c\u6700\u540e\u7528\u65e5\u5fd7\u548c\u5f39\u7a97\u6392\u67e5\u95ee\u9898\u3002"
              : "Organized by the real workflow: create a profile, fill it from an example, then use logs and dialogs for troubleshooting."}
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
        title={isZh ? "\u5e38\u7528\u6837\u4f8b\uff1a\u672c\u5730\u6253\u5f00\u8fdc\u7a0b 127.0.0.1:8000" : "Common Example: Open Remote 127.0.0.1:8000 Locally"}
        description={
          isZh
            ? "\u8fd9\u662f Local \u8f6c\u53d1\u7684\u5178\u578b\u573a\u666f\u3002\u8fdc\u7a0b\u670d\u52a1\u53ea\u76d1\u542c\u670d\u52a1\u5668\u81ea\u5df1\u7684 127.0.0.1\uff0c\u4f46\u4f60\u53ef\u4ee5\u901a\u8fc7 SSH \u96a7\u9053\u5728\u672c\u5730\u6d4f\u89c8\u5668\u8bbf\u95ee\u3002"
            : "This is the typical Local forwarding case. The remote service only listens on the server's own 127.0.0.1, but your local browser can reach it through the SSH tunnel."
        }
        rows={remoteLocalRows}
      />

      <div className="grid grid-cols-2 gap-5">
        <GuidePanel title={isZh ? "\u6bcf\u4e2a\u9875\u9762\u600e\u4e48\u7528" : "Pages"}>
          {pageCards.map(([title, body]) => <GuideLine key={title} title={title} body={body} />)}
        </GuidePanel>
        <GuidePanel title={isZh ? "\u8f6c\u53d1\u6a21\u5f0f\u600e\u4e48\u9009" : "Forwarding Modes"}>
          {modeCards.map(([title, body]) => <GuideLine key={title} title={title} body={body} />)}
        </GuidePanel>
      </div>

      <GuidePanel title={isZh ? "\u6392\u9519\u4e0e\u6ce8\u610f\u4e8b\u9879" : "Troubleshooting"}>
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
    <div className="rounded-2xl border border-blue-100 bg-white p-5 text-slate-900 shadow-sm transition duration-200 hover:-translate-y-0.5 hover:shadow-soft dark:border-blue-950/60">
      <div className="text-base font-semibold">{props.title}</div>
      <p className="mt-2 text-sm leading-6 text-slate-500">{props.description}</p>
      <div className="mt-4 overflow-hidden rounded-xl border border-slate-200">
        {props.rows.map(([label, value]) => (
          <div key={`${label}-${value}`} className="grid grid-cols-[120px_1fr] border-b border-slate-100 last:border-b-0">
            <div className="bg-blue-50 px-3 py-2 text-sm font-medium text-blue-700">{label}</div>
            <div className="px-3 py-2 text-sm text-slate-600">{value}</div>
          </div>
        ))}
      </div>
    </div>
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
