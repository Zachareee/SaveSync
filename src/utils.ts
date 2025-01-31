import { invoke as old_invoke } from "@tauri-apps/api/core";
import { emit as old_emit } from "@tauri-apps/api/event";
import { Info } from "./types.ts";

export function invoke<T extends keyof InvokeTypes>(
  command: T, payload?: InvokeTypes[T][0]
): Promise<InvokeTypes[T][1]> {
  return old_invoke(command, payload);
}

export function emit<T extends keyof EmitTypes>(eventName: string, payload?: EmitTypes[T]) {
  return old_emit(eventName, payload)
}

/** key: command
 *  value`[`0`]`: input type
 *  value`[`1`]`: output type
 */
type InvokeTypes = {
  get_plugins: [undefined, Info[]];
};

/** key: command
 *  value: input type
 */
type EmitTypes = {
  init: string;
};
