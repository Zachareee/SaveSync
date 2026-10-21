import { folders } from "@/App"
import PageRoot from "@/PageRoot"
import { Component, createSignal, For } from "solid-js"
import { useParams } from "@solidjs/router"

import { createDraggable, DragDropProvider, useDragDropContext } from "@thisbeyond/solid-dnd"

export default function Profiles() {
  const params = useParams()
  const { TAGNAME } = Object.fromEntries(
    Object.entries(params).map(([k, v]) => [k, decodeURIComponent(v)])
  )
  const [pattern, setPattern] = createSignal("")

  const items = Object.entries(folders[TAGNAME]).filter(([_, { synced }]) => synced).map(([file]) => file)

  const Draggable: Component<{ id: string }> = (props) => {
    const draggable = createDraggable(props.id)
    return <div use:draggable>droppable</div>
  }

  const Sandbox = () => {
    const [, { onDragEnd }] = useDragDropContext()

    onDragEnd(({ draggable, droppable }) => {
      if (droppable) { }
    })
  }

  return <PageRoot>
    <div>
      <input value={pattern()} onInput={(e) => setPattern(e.target.value)} />
      <div>
        <For each={items.filter(file => {
          try {
            return file.match(new RegExp(pattern(), "i"))?.length ?? 0 > 1
          } catch {
            return true
          }
        })}>
          {
            item => <div class="overflow-x-auto text-nowrap bg-slate-900 rounded-2xl p-4 m-2 select-none cursor-pointer hover:outline-2">{item}</div>
          }
        </For>
      </div>
    </div>
  </PageRoot>
}
