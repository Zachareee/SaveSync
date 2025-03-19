import { listen, osStringToString } from "./backend";
import { createWindow } from "./window";

export const conflicting_listener = () =>
  listen("conflicting_files", ([tag, folder, [local, cloud]]) => {
    createWindow(`/conflicting/${osStringToString(folder)}/${local.secs_since_epoch}/${cloud.secs_since_epoch}/${tag}`, { title: "Outdated folder", parent: "main" })
  })
