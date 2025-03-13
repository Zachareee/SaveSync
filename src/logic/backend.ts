import { IPCtypes, OptionalParameter, OsString } from "@/types";
import { invoke as old_invoke } from "@tauri-apps/api/core";
import { listen as old_listen, emit as old_emit, Options, EventName, UnlistenFn } from "@tauri-apps/api/event";
import { onCleanup } from "solid-js";


export function invoke<T extends keyof IPCtypes.InvokeTypes>(
  command: T, ...payload: OptionalParameter<IPCtypes.InvokeTypes[T][0]>
): Promise<IPCtypes.InvokeTypes[T][1]> {
  return old_invoke(command, ...payload);
}

export function emit<T extends keyof IPCtypes.EmitTypes>(eventName: T, ...payload: OptionalParameter<IPCtypes.EmitTypes[T]>) {
  return old_emit(eventName, ...payload)
}

export function listen<T extends keyof IPCtypes.ListenTypes>(eventName: T | EventName, handler: (param: IPCtypes.ListenTypes[T]) => void, option?: Options) {
  return old_listen<IPCtypes.ListenTypes[T]>(eventName, ({ payload }) => handler(payload), option)
}

export function osStringToString(osString?: OsString) {
  return osString ? String.fromCharCode(...osString.Windows) : ""
}

export function stringToOsString(str: string): OsString {
  return { Windows: str.split('').map(s => s.charCodeAt(0)) }
}

export function unlisten(unlistens: Promise<UnlistenFn>[]) {
  return () => onCleanup(async () => await Promise.all(unlistens.map(async f => (await f)())))
}
