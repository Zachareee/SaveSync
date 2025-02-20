import { useFolderContext } from "@/App"
import { osStringToString } from "@/logic/all-backend"
import { createWindow } from "@/logic/window";
import { emit, listen } from "@/logic/main-backend"
import { useParams } from "@solidjs/router"
import { Index, onCleanup } from "solid-js"

const sync_folder = (data: Record<string, string>) => {
  emit("sync", data)
}

export default function Folders() {
  const { TAG: tag } = useParams<{ TAG: string }>()
  const { folders } = useFolderContext()!

  const unlisten = listen("plugin_error", ({ payload: [title, description] }) => createWindow(title, { url: `/error/${description}` }))
  onCleanup(() => unlisten.then(e => e()))
  return <>
    <Index each={folders[tag]}>
      {
        foldername => <button onclick={[sync_folder, { tag, foldername: foldername() }]}>{osStringToString(foldername())}</button>
      }
    </Index>
  </>
}
