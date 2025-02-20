import { listen as old_listen, emit as old_emit, EventCallback, Options, EventName } from "@tauri-apps/api/event";
import { EmitTypes, ListenTypes} from "@/types.ts";

export function emit<T extends keyof EmitTypes>(eventName: T, payload?: EmitTypes[T]) {
  return old_emit(eventName, payload)
}

export function listen<T extends keyof ListenTypes>(eventName: T | EventName, handler: EventCallback<ListenTypes[T]>, option?: Options) {
  return old_listen(eventName, handler, option)
}
