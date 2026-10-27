import { folders } from "@/logic/folder_state"
import PageRoot from "@/PageRoot"
import { Component, createSignal, For, JSX } from "solid-js"
import { useNavigate, useParams } from "@solidjs/router"

import { closestCenter, createDraggable, DragDropProvider, DragDropSensors, DragEventHandler } from "@thisbeyond/solid-dnd"
import { mapValues } from "lodash"
import { Portal } from "solid-js/web"

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
  return <div use:draggable>{props.children}</div>
}

const onDragEnd: DragEventHandler = ({ droppable }) => {
  console.log(droppable?.id)
}

export default function Profiles() {
  const { TAGNAME: tag } = mapValues(useParams<Record<string, string>>(), decodeURIComponent)
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
  const navigate = useNavigate()

  const items = Object.keys(folders[tag])
  const profiles = ["allen", "bob", "charles"]

  return <PageRoot>
    <div>
      <input value={pattern()} onInput={(e) => setPattern(e.target.value)} />
      <DragDropProvider onDragEnd={onDragEnd} collisionDetector={closestCenter}>
        <DragDropSensors />
        <div>
          <div class="inline-block">
            <For each={items.filter(file => (file.match(pattern())?.length ?? 0) > 0)}>
              {
                item => <p>{item}</p>
              }
            </For>
          </div>
          <div class="inline-block">
            <For each={profiles}>
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
