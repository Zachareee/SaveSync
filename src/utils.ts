import { invoke as old_invoke } from "@tauri-apps/api/core";
import { Info } from "./types.ts";
export function invoke<T extends keyof InvokeTypes>(
  command: T,
): Promise<InvokeTypes[T]> {
  return old_invoke(command);
}
type InvokeTypes = {
  get_plugins: Info[];
};
