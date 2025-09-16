import { FolderMapping, RequiredList, EnvMapping, Info } from "./data";
import { OsString, SystemTime } from "./rust";

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
  conflict_resolve: [string, OsString, string]
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
  conflicting_files: [string, OsString, [SystemTime, SystemTime]]
}

