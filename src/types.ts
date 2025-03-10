export type OptionalParameter<T> = undefined extends T ? [p?: T] : [p: T]

export namespace IPCtypes {
  /** 
   *  key: command
   *  value`[`0`]`: input type
   *  value`[`1`]`: output type
   */
  export type InvokeTypes = {
    get_plugins: [undefined, Info[]]
    get_filetree: [undefined, FileTree]
    saved_plugin: [undefined, boolean]
    get_mapping: [undefined, { mapping: FolderMapping, ignored: IgnoreList }]
    get_envpaths: [undefined, EnvMapping]
    set_mapping: [{ map: FolderMapping }, undefined]
  };

  /** 
   *  key: command
   *  value: input type
   */
  export type EmitTypes = {
    init: string
    refresh: undefined
    abort: string
    sync: Record<"tag" | "foldername", string>
    unload: undefined
    saved_plugin: undefined
  };

  /** 
   *  key: command
   *  value: output type
   */
  export type ListenTypes = {
    plugins: Info[]
    init_result: boolean
    abort_result: string
    plugin_error: [string, string]
    saved_result: undefined
  }
}

export type OsString = { Windows: number[] }
export type Info = Record<"name" | "description" | "author" | "filename", string> & Partial<
  Record<"icon_url", string>
>

export type FileTree = Record<string, OsString[]>
export type FolderMapping = Record<string, [string, OsString]>
export type EnvMapping = Record<string, OsString>
export type IgnoreList = string[]
