import { emit, invoke, listen, osStringToString, stringToOsString, unlisten } from "@/logic/backend"
import { useNavigate } from "@solidjs/router"
import { createSignal, For, Index, Show } from "solid-js"
import { menuStatus } from "@/logic/menu"
import toast from "solid-toast"
import lo from "lodash"
import { FileTree, OsString } from "@/types"
import { Portal } from "solid-js/web"
import DivButton from "@/components/DivButton"
import { createStore, reconcile } from "solid-js/store"

const sync_folder = (data: { tag: string, foldername: OsString }) => {
  emit("sync", data)
}

export default function Fmap() {
  menuStatus(true)
  const [currentFolder, setCurrentFolder] = createSignal("")
  const [folders, setFolders] = createStore<FileTree>()

  unlisten([
    listen("sync_result", ({ payload: [tag, folder, bool] }) => {
      setFolders(tag, osStringToString(folder), bool)
    }),
    listen("filetree_result", ({ payload }) => {
      setCurrentFolder("")
      invoke("get_watched_folders").then(watched => {
        setFolders(reconcile(Object.fromEntries(
          Object.entries(payload).map(
            ([k, v]) => [k, Object.fromEntries(v.map(e =>
              [osStringToString(e), watched.some(
                tagpath => lo.isEqual(tagpath, [k, e])
              )]
            ))]
          )
        )))
      })
    })
  ])()

  emit("filetree")

  invoke("get_mapping").then(({ mapping, required }) => {
    if (!lo.isEqual(required, Object.entries(mapping).map(([key]) => key)))
      toast.error("Some folders were not synced, please check File -> Mappings")
  })

  return <main class="w-full">
    <Show when={currentFolder()}
      fallback={<TagList folders={folders} setCurrentFolder={setCurrentFolder} />}>
      <FolderList folders={folders} currentFolder={currentFolder()} setCurrentFolder={setCurrentFolder} />
    </Show>
  </main>
}

function TagList(props: { folders: FileTree, setCurrentFolder: CurrentFolderSetter }) {
  const navigate = useNavigate()

  return <div class="flex justify-center">
    <For each={Object.entries(props.folders)}>
      {elem => <div class="border-white m-4" onclick={[props.setCurrentFolder, elem[0]]}>
        <p>{elem[0]}</p>
      </div>}
    </For>
    <Portal>
      <div class="fixed right-0 bottom-0 m-4">
        <button onclick={() => { emit("unload"); navigate("/") }}>Back to plugin select</button>
      </div>
    </Portal>
  </div>
}

function FolderList(props: { folders: FileTree, currentFolder: string, setCurrentFolder: CurrentFolderSetter }) {
  return <div class="flex justify-center">
    <div class="w-min">
      <Index each={Object.entries(props.folders[props.currentFolder])}>
        {
          foldername => <DivButton onclick={[sync_folder, { tag: props.currentFolder, foldername: stringToOsString(foldername()[0]) }]}>
            <input type="checkbox" class="mr-4 rounded-2xl" checked={foldername()[1]} onclick={(e) => e.preventDefault()} />
            {foldername()[0]}
          </DivButton>
        }
      </Index>
    </div>
    <Portal>
      <div class="fixed right-0 bottom-0 m-4">
        <button onclick={[props.setCurrentFolder, ""]}>Back to tags</button>
      </div>
    </Portal>
  </div>
}

type CurrentFolderSetter = ReturnType<typeof createSignal<string>>[1]
