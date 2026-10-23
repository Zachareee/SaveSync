import { FileTree } from "@/types/data";
import { createStore } from "solid-js/store";

export const [folders, setFolders] = createStore<FileTree>();

