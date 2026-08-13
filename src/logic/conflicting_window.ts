import { WebviewWindow } from "@tauri-apps/api/webviewWindow";
import { listen, osStringToString } from "./backend";

export const conflicting_listener = () =>
  listen("conflicting_files", ([tag, folder, [local, cloud]]) => {
    const folderString = osStringToString(folder)
    return new WebviewWindow(`conflicting:${tag}:${folderString}`, {
      url: `/conflicting?${new URLSearchParams({
        tag,
        folder: folderString,
        local: local.secs_since_epoch.toString(),
        cloud: cloud.secs_since_epoch.toString()
      })}`,
      title: "Outdated folder",
      parent: "main"
    }).once("tauri://error", console.log)
  })
