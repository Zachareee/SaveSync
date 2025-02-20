import { invoke as old_invoke } from "@tauri-apps/api/core";
import { listen as old_listen, emit as old_emit, EventCallback, Options, EventName } from "@tauri-apps/api/event";
import { EmitTypes, InvokeTypes, ListenTypes, OsString } from "@/types.ts";

export function osStringToString(osString: OsString) {
  return String.fromCharCode(...osString.Windows)
}

export function invoke<T extends keyof InvokeTypes>(
  command: T, payload?: InvokeTypes[T][0]
): Promise<InvokeTypes[T][1]> {
  return old_invoke(command, payload);
}

export function emit<T extends keyof EmitTypes>(eventName: T, payload?: EmitTypes[T]) {
  return old_emit(eventName, payload)
}

export function listen<T extends keyof ListenTypes>(eventName: T | EventName, handler: EventCallback<ListenTypes[T]>, option?: Options) {
  return old_listen(eventName, handler, option)
}
