import { InvokeTypes, OsString } from "@/types";
import { invoke as old_invoke } from "@tauri-apps/api/core";

export function invoke<T extends keyof InvokeTypes>(
  command: T, payload?: InvokeTypes[T][0]
): Promise<InvokeTypes[T][1]> {
  return old_invoke(command, payload);
}

export function osStringToString(osString: OsString) {
  return String.fromCharCode(...osString.Windows)
}
