export type OsString = { Windows: number[] }
export type Info = Record<"name" | "description" | "author" | "filename", string> & Partial<
  Record<"icon_url", string>
>

export type FolderMapping = Record<string, OsString[]>
