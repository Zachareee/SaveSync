import { InvokeTypes, OsString } from "@/types";
import { invoke as old_invoke } from "@tauri-apps/api/core";

export function invoke<T extends keyof InvokeTypes>(
  command: T, payload?: InvokeTypes[T][0]
): Promise<InvokeTypes[T][1]> {
  return old_invoke(command, payload);
}

export function osStringToString(osString?: OsString) {
  return osString ? String.fromCharCode(...osString.Windows) : ""
}

export function stringToOsString(str: string): OsString {
  return { Windows: str.split('').map(s => s.charCodeAt(0)) }
}
