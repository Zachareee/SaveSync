import { listen } from "./backend";
import { createWindow } from "./window";

export const conflicting_listener = () =>
  listen("conflicting_files", ([tag, folder]) => createWindow(`/conflicting/${folder}/${tag}`, { title: "Outdated folder", parent: "main" }))
