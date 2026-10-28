import { folders } from "@/logic/folder_state"
import PageRoot from "@/PageRoot"
import { Component, createSignal, For, Index, JSX, Show } from "solid-js"
import { useNavigate, useParams } from "@solidjs/router"

import { createDraggable, createDroppable, DragDropProvider, DragDropSensors, DragEventHandler } from "@thisbeyond/solid-dnd"
import { concat, flow, mapValues, partialRight, without } from "lodash"
import { Portal } from "solid-js/web"
import { createStore } from "solid-js/store"

declare module 'solid-js' {
  namespace JSX {
    interface Directives {
      draggable: boolean
      droppable: boolean
      sortable: boolean
    }
  }
}

const Draggable: Component<JSX.HTMLAttributes<HTMLDivElement> & { id: string }> = (props) => {
  const draggable = createDraggable(props.id)
  return <div use:draggable {...props}>{props.children}</div>
}

const Droppable: Component<JSX.HTMLAttributes<HTMLDivElement> & { id: string }> = (props) => {
  const droppable = createDroppable(props.id)
  return <div use:droppable {...props}>{props.children}</div>
}

export default function Profiles() {
  const { TAGNAME: tag } = mapValues(useParams<Record<string, string>>(), decodeURIComponent)
  const navigate = useNavigate()

  const [pattern, setPattern] = createSignal("", {
    equals(prev, next) {
      try {
        new RegExp(next)
      } catch {
        return true
      }

      return prev === next
    }
  })

  const items = Object.keys(folders[tag])
  const [profiles, setProfiles] = createSignal(["allen", "bob", "charles"])
  const [profileMapping, setProfileMapping] = createStore<Record<string, string>>(Object.fromEntries(items.map(k => [k, k])))

  const onDragEnd: DragEventHandler = ({ droppable, draggable }) => {
    if (draggable && droppable) {
      // emit(actual file, profile file)
      console.log(`draggable: ${draggable.id}, droppable: ${droppable.id}`)
      setProfiles(flow(
        partialRight(without, draggable.id.toString()),
        partialRight(concat, profileMapping[droppable.id.toString()])
      ))
      setProfileMapping(droppable.id.toString(), draggable.id.toString())
    }
  }

  return <PageRoot>
    <div>
      <input value={pattern()} onInput={(e) => setPattern(e.target.value)} />
      <DragDropProvider onDragEnd={onDragEnd}>
        <DragDropSensors />
        <div class="flex">
          <div class="flex-col place-items-center outline-2 h-full space-y-4 p-2 rounded-xl">
            <For each={items.filter(file => (file.match(pattern())?.length ?? 0) > 0)}>
              {
                item => <div class="flex space-x-4">
                  <h1>{item}</h1>
                  <Droppable id={item}>
                    <div class="p-2 outline-2 select-none rounded-xl">
                      {profileMapping[item]}
                    </div>
                  </Droppable>
                </div>
              }
            </For>
          </div>
          <div class="inline-block">
            <For each={profiles()}>
              {
                item => <Draggable id={item}>
                  <div class="overflow-x-auto text-nowrap bg-slate-900 rounded-2xl p-4 m-2 select-none cursor-pointer hover:outline-2">{item}</div>
                </Draggable>
              }
            </For>
          </div>
        </div>
      </DragDropProvider>
    </div>
    <Portal>
      <div class="fixed right-0 bottom-0 m-4">
        <button onclick={[navigate, "/profiles"]}>Back to profiles</button>
      </div>
    </Portal>
  </PageRoot>
}
