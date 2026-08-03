import { OsString } from "./rust";

export type Info = Record<"name" | "description" | "author", string>
  & Record<"filename", OsString>
  & Partial<
    Record<"icon_url", string>
  >

export type FileTree = Record<string, Record<string, Record<"folder" | "loading" | "synced", boolean>>>
export type FolderMapping = Record<string, OsString>
export type RequiredList = string[]
