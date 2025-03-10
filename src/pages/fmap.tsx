import { useFolderContext } from "@/App"
import { emit, invoke, osStringToString } from "@/logic/backend"
import { useNavigate } from "@solidjs/router"
import { createSignal, Index, Show } from "solid-js"
import { menuStatus } from "@/logic/menu"
import toast from "solid-toast"
import { FileTree } from "@/types"

const sync_folder = (data: Record<string, string>) => {
  emit("sync", data)
}

export default function Fmap() {
  menuStatus(true)
  const [currentFolder, setCurrentFolder] = createSignal("FAKE")

  const navigate = useNavigate()
  const { folders, setFolders } = useFolderContext()!

  invoke("get_filetree").then(setFolders)

  invoke("get_mapping").then(({ ignored }) => {
    if (ignored.length)
      toast.error("Some folders were not synced, please check File -> Mappings")
  })

  return <main class="container">
    <Show when={currentFolder()}
      fallback={<TagList folders={folders} setCurrentFolder={setCurrentFolder} />}>
      <FolderList folders={folders} currentFolder={currentFolder()} setCurrentFolder={setCurrentFolder} />
    </Show>
    <div>
      <button onclick={() => { emit("unload"); navigate("/") }}>Back to plugin select</button>
    </div>
  </main>
}

function TagList(props: { folders: FileTree, setCurrentFolder: CurrentFolderSetter }) {
  return <Index each={Object.entries(props.folders)}>
    {elem => <div class="border-white m-4" onclick={[props.setCurrentFolder, elem()[0]]}>
      <p>{elem()[0]}</p>
    </div>}
  </Index>
}

function FolderList(props: { folders: FileTree, currentFolder: string, setCurrentFolder: CurrentFolderSetter }) {
  return <div class="flex justify-center">
    <div class="w-min">
      <Index each={props.folders[props.currentFolder]}>
        {
          foldername => <button class="w-full text-nowrap" onclick={[sync_folder, { tag: props.currentFolder, foldername: foldername() }]}>{osStringToString(foldername())}</button>
        }
      </Index>
    </div>
  </div>
}

type CurrentFolderSetter = ReturnType<typeof createSignal<string>>[1]
