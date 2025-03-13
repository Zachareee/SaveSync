export type OptionalParameter<T> = undefined extends T ? [p?: T] : [p: T]

export namespace IPCtypes {
  /**
   *  key: command
   *  value`[`0`]`: input type
   *  value`[`1`]`: output type
   */
  export type InvokeTypes = {
    get_plugins: [undefined, Info[]]
    saved_plugin: [undefined, boolean]
    get_mapping: [undefined, { mapping: FolderMapping, required: RequiredList }]
    get_envpaths: [undefined, EnvMapping]
    set_mapping: [{ map: FolderMapping }, undefined]
    get_watched_folders: [undefined, [string, OsString][]]
  };

  /**
   *  key: command
   *  value: input type
   */
  export type EmitTypes = {
    init: string
    refresh: undefined
    abort: string
    sync: { tag: string, foldername: OsString }
    unload: undefined
    saved_plugin: undefined
    filetree: undefined
    conflict_resolve: [string, string, string]
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
    sync_result: [string, OsString, boolean]
    filetree_result: Record<string, OsString[]>
    conflicting_files: [string, string]
  }
}

export type OsString = { Windows: number[] }
export type Info = Record<"name" | "description" | "author" | "filename", string> & Partial<
  Record<"icon_url", string>
>

export type FileTree = Record<string, Record<string, boolean>>
export type FolderMapping = Record<string, [string, OsString]>
export type EnvMapping = Record<string, OsString>
export type RequiredList = string[]
