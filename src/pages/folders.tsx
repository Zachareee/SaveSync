import { useFolderContext } from "@/App"
import { emit, osStringToString } from "@/utils"
import { useParams } from "@solidjs/router"
import { Index } from "solid-js"

const sync_folder = (data: Record<string, string>) => {
  emit("sync", data)
}

export default function Folders() {
  const { TAG: tag } = useParams<{ TAG: string }>()
  const { folders } = useFolderContext()!
  return <>
    <Index each={folders[tag]}>
      {
        foldername => <button onclick={[sync_folder, { tag, foldername: foldername() }]}>{osStringToString(foldername())}</button>
      }
    </Index>
  </>
}
