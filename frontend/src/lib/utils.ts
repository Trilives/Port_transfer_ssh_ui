import { type ClassValue, clsx } from "clsx";
import { twMerge } from "tailwind-merge";

export function cn(...inputs: ClassValue[]) {
  return twMerge(clsx(inputs));
}

/** 端口转发监听端点对应的浏览器地址；监听全地址（0.0.0.0/::）按 127.0.0.1 处理。 */
export function forwardWebUrl(forward: { bindHost?: string; bindPort: string }): string {
  let host = (forward.bindHost ?? "").trim();
  if (host === "" || host === "0.0.0.0" || host === "::" || host === "[::]") host = "127.0.0.1";
  return `http://${host}:${forward.bindPort.trim()}`;
}
