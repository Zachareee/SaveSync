import { listen } from "./backend";
import { createWindow } from "./window";

export const conflicting_listener = () =>
  listen("conflicting_files", ([tag, folder, [local, cloud]]) => createWindow(`/conflicting/${folder}/${local}/${cloud}/${tag}`, { title: "Outdated folder", parent: "main" }))
