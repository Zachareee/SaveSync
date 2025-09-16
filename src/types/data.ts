import { OsString } from "./rust";

export type Info = Record<"name" | "description" | "author" | "filename", string> & Partial<
  Record<"icon_url", string>
>

export type FileTree = Record<string, Record<string, boolean>>
export type FolderMapping = Record<string, [string, OsString]>
export type EnvMapping = Record<string, OsString>
export type RequiredList = string[]
