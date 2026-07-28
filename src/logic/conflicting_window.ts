import { WebviewWindow } from "@tauri-apps/api/webviewWindow";
import { listen, osStringToString } from "./backend";

export const conflicting_listener = () =>
  listen("conflicting_files", ([tag, folder, [local, cloud]]) => {
    return new WebviewWindow("conflicting", {
      url: `/conflicting?${new URLSearchParams({
        tag,
        folder: osStringToString(folder),
        local: local.secs_since_epoch.toString(),
        cloud: cloud.secs_since_epoch.toString()
      })}`,
      title: "Outdated folder",
      parent: "main"
    }).once("tauri://error", console.log)
  })
