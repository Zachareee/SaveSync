import { WebviewWindow } from "@tauri-apps/api/webviewWindow";

export function createWindow(title: string, opts: ConstructorParameters<typeof WebviewWindow>[1]) {
  return new WebviewWindow(title.replace(/[^a-zA-Z-/:_]/g, ""), { ...opts, focus: true }).once("tauri://error", console.log)
}
