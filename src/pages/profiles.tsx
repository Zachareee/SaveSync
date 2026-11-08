import { folders } from "@/logic/folder_state"
import PageRoot from "@/PageRoot"
import { Component, createSignal, For, JSX } from "solid-js"
import { useNavigate, useParams } from "@solidjs/router"

import { createDraggable, createDroppable, DragDropProvider, DragDropSensors, DragEventHandler } from "@thisbeyond/solid-dnd"
import _, { concat, flow, map, mapKeys, mapValues, partialRight, without } from "lodash"
import { Portal } from "solid-js/web"
import { createStore } from "solid-js/store"
import { invoke, osStringToString, stringToOsString } from "@/logic/backend"
import { OsString } from "@/types/rust"

declare module 'solid-js' {
  namespace JSX {
    interface Directives {
      draggable: boolean
      droppable: boolean
    }
  }
}

const Draggable: Component<JSX.HTMLAttributes<HTMLDivElement> & { id: string }> = (props) => {
  // @ts-ignore
  const draggable = createDraggable(props.id)
  return <div use:draggable {...props}>{props.children}</div>
}

const Droppable: Component<JSX.HTMLAttributes<HTMLDivElement> & { id: string }> = (props) => {
  // @ts-ignore
  const droppable = createDroppable(props.id)
  return <div use:droppable {...props}>{props.children}</div>
}

function createIdMap(names: string[]) {
  return Object.fromEntries(names.map(e => [crypto.randomUUID(), e]))
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

  const uuidFolderMapping = createIdMap(
    Object.keys(folders[tag]).filter(name => !name.endsWith(".savesyncprofile"))
  )
  const [idMapping, setIdMapping] = createStore<Record<string, string>>({})

  const [uuidProfileMapping, setUuidProfileMapping] = createStore<Record<string, string>>({})
  invoke("get_profiles", { tag })
    .then(dict => (Object.keys(dict) as unknown as OsString[]).map((k: OsString) => osStringToString(k)))
    .then(createIdMap)
    .then(newMapping => {
      setIdMapping({ ...uuidFolderMapping, ...newMapping })
      setUuidProfileMapping(newMapping)
    })

  const [inactiveProfiles, setInactiveProfiles] = createStore(Object.keys(uuidProfileMapping))
  const [activeProfileMapping, setActiveProfileMapping] = createStore(_(uuidFolderMapping).keys().map(k => [k, k]).fromPairs().value())

  const onDragEnd: DragEventHandler = ({ droppable, draggable }) => {
    if (draggable && droppable) {
      const draggableId = draggable.id.toString()
      const droppableId = droppable.id.toString()
      invoke("change_active_profile", {
        tag,
        filename: stringToOsString(idMapping[droppableId]),
        profile: stringToOsString(idMapping[draggableId])
      })

      setInactiveProfiles(flow(
        partialRight(without, draggableId),
        partialRight(concat, activeProfileMapping[droppableId]),
      ))
      setActiveProfileMapping(droppableId, draggableId)
    }
  }

  return <PageRoot>
    <div>
      <input value={pattern()} onInput={({ target: { value } }) => setPattern(value)} />
      <DragDropProvider onDragEnd={onDragEnd}>
        <DragDropSensors />
        <div class="flex">
          <div class="flex-col place-items-center outline-2 h-full space-y-4 p-2 rounded-xl">
            <For each={
              _(uuidFolderMapping)
                .toPairs()
                .filter(([_, file]) => (file.match(pattern())?.length ?? 0) > 0)
                .map(0)
                .value()
            }>
              {
                id => <div class="flex space-x-4">
                  <h1>{uuidFolderMapping[id]}</h1>
                  <Droppable id={id}>
                    <div class="p-2 outline-2 select-none rounded-xl">
                      {idMapping[activeProfileMapping[id]]}
                    </div>
                  </Droppable>
                </div>
              }
            </For>
          </div>
          <div class="inline-block">
            <h1>Profiles</h1>
            <For each={inactiveProfiles}>
              {
                id =>
                  <Draggable id={id}>
                    <div class="overflow-x-auto text-nowrap bg-slate-900 rounded-2xl m-2 select-none cursor-pointer hover:outline-2">
                      <input
                        value={idMapping[id]}
                        class="max-w-[16ch]"
                        onInput={({ target: { value } }) => setIdMapping(id, value)} />
                    </div>
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
