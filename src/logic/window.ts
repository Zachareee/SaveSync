import { WebviewWindow } from "@tauri-apps/api/webviewWindow";

export function createWindow(url: string, opts: ConstructorParameters<typeof WebviewWindow>[1]) {
  return new WebviewWindow(url, { ...opts, focus: true, url }).once("tauri://error", console.log)
}
