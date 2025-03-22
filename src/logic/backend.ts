import { OsString, SystemTime } from "@/types/rust";
import { InvokeTypes, EmitTypes, ListenTypes } from "@/types/IPC";
import { invoke as old_invoke } from "@tauri-apps/api/core";
import { listen as old_listen, emit as old_emit, Options, EventName, UnlistenFn } from "@tauri-apps/api/event";
import { onCleanup } from "solid-js";

const intl = new Intl.DateTimeFormat("en-UK", { dateStyle: "medium", timeStyle: "short" })

export function invoke<T extends keyof InvokeTypes>(
  command: T, ...payload: OptionalParameter<InvokeTypes[T][0]>
): Promise<InvokeTypes[T][1]> {
  return old_invoke(command, ...payload);
}

export function emit<T extends keyof EmitTypes>(eventName: T, ...payload: OptionalParameter<EmitTypes[T]>) {
  return old_emit(eventName, ...payload)
}

export function listen<T extends keyof ListenTypes>(eventName: T | EventName, handler: (param: ListenTypes[T]) => void, option?: Options) {
  return old_listen<ListenTypes[T]>(eventName, ({ payload }) => handler(payload), option)
}

export function osStringToString(osString?: OsString) {
  return osString ? String.fromCharCode(...osString.Windows) : ""
}

export function stringToOsString(str: string): OsString {
  return { Windows: str.split('').map(s => s.charCodeAt(0)) }
}

export function systemTimeToNumber(time: SystemTime) {
  return new Date(time.secs_since_epoch * 1000)
}

export function dateToLocaleString(date: Date) {
  return intl.format(date)
}

export function unlisten(unlistens: Promise<UnlistenFn>[]) {
  return () => onCleanup(async () => await Promise.all(unlistens.map(async f => (await f)())))
}

type OptionalParameter<T> = undefined extends T ? [p?: T] : [p: T]
