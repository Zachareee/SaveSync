import { WebviewWindow } from "@tauri-apps/api/webviewWindow";
import { listen, osStringToString } from "./backend";

const labelRegex = /[^a-zA-Z0-9-\/:_]/g

export const conflicting_listener = () =>
  listen("conflicting_files", ([tag, folder, [local, cloud]]) => {
    const folderString = osStringToString(folder)
    return new WebviewWindow(`conflicting:${tag.replace(labelRegex, "")}:${folderString.replace(labelRegex, "")}`, {
      url: `/conflicting?${new URLSearchParams({
        tag,
        folder: folderString,
        local: local.secs_since_epoch.toString(),
        cloud: cloud.secs_since_epoch.toString()
      })}`,
      title: "Outdated folder",
      parent: "main"
    }).once("tauri://error", alert)
  })
