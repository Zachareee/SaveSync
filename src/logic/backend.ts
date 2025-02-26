import { EmitTypes, InvokeTypes, ListenTypes, OptionalParameter, OsString } from "@/types";
import { invoke as old_invoke } from "@tauri-apps/api/core";
import { listen as old_listen, emit as old_emit, EventCallback, Options, EventName } from "@tauri-apps/api/event";

export function invoke<T extends keyof InvokeTypes>(
  command: T, ...payload: OptionalParameter<InvokeTypes[T][0]>
): Promise<InvokeTypes[T][1]> {
  return old_invoke(command, ...payload);
}

export function emit<T extends keyof EmitTypes>(eventName: T, ...payload: OptionalParameter<EmitTypes[T]>) {
  return old_emit(eventName, ...payload)
}

export function listen<T extends keyof ListenTypes>(eventName: T | EventName, handler: EventCallback<ListenTypes[T]>, option?: Options) {
  return old_listen(eventName, handler, option)
}

export function osStringToString(osString?: OsString) {
  return osString ? String.fromCharCode(...osString.Windows) : ""
}

export function stringToOsString(str: string): OsString {
  return { Windows: str.split('').map(s => s.charCodeAt(0)) }
}
