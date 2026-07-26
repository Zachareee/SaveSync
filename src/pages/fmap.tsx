import { emit, invoke, listen, osStringToString, stringToOsString, unlisten } from "@/logic/backend"
import { useNavigate } from "@solidjs/router"
import { createSignal, For, Index, Show } from "solid-js"
import toast from "solid-toast"
import lo from "lodash"
import { FileTree } from "@/types/data"
import { OsString } from "@/types/rust"
import { Portal } from "solid-js/web"
import DivButton from "@/components/DivButton"
import { createStore, reconcile } from "solid-js/store"
import { conflicting_listener } from "@/logic/conflicting_window"

import { Folder, InsertDriveFile, Loop } from "@suid/icons-material"

export default function Fmap() {
  const [currentFolder, setCurrentFolder] = createSignal("")
  const [folders, setFolders] = createStore<FileTree>()

  unlisten([
    listen("sync_result", ([tag, folder, bool]) => {
      setFolders(tag, osStringToString(folder), { loading: false, synced: bool })
    }),
    conflicting_listener()
  ])()

  invoke("filetree").then(payload => {
    setCurrentFolder("")
    invoke("get_watched_folders").then(watched => {
      setFolders(reconcile(Object.fromEntries(
        Object.entries(payload).map(
          ([k, v]) => [k, Object.fromEntries(v.map(([filename, isFolder]) =>
            [osStringToString(filename), {
              folder: isFolder,
              synced: watched.some(
                tagpath => lo.isEqual(tagpath, [k, filename])
              ),
              loading: false
            }]
          ))]
        )
      )))
    })
  })

  const sync_folder = (foldername: OsString) => {
    const tag = currentFolder()
    setFolders(tag, osStringToString(foldername), "loading", true)
    emit("sync", { tag, foldername })
  }


  invoke("get_mapping").then(({ mapping, required }) => {
    let current = Object.entries(mapping).map(([key]) => key)
    if (required.some(tag => !current.includes(tag)))
      toast.error("Some folders were not synced, please check File -> Mappings")
  })

  return <div class="flex justify-center items-center">
    <Show when={currentFolder()}
      fallback={<TagList folders={Object.keys(folders).toSorted()} setCurrentFolder={setCurrentFolder} />}>
      <FolderList folders={folders[currentFolder()]} sync_folder={sync_folder} back={() => setCurrentFolder("")} />
    </Show>
  </div>
}

function TagList(props: { folders: string[], setCurrentFolder: (s: string) => void }) {
  const navigate = useNavigate()

  return <>
    <For each={props.folders}>
      {elem => <div class="border-white m-4" onclick={[props.setCurrentFolder, elem]}>
        <p>{elem}</p>
      </div>}
    </For>
    <Portal>
      <div class="fixed right-0 bottom-0 m-4">
        <button onclick={() => { emit("unload"); navigate("/") }}>Back to plugin select</button>
      </div>
    </Portal>
  </>
}

function FolderList(props: { folders: FileTree[string], sync_folder: (arg: OsString) => void, back: () => void }) {
  return <>
    <div class="w-min">
      <Index each={Object.entries(props.folders)}>
        {
          foldername => <DivButton onclick={[props.sync_folder, stringToOsString(foldername()[0])]}>
            <input type="checkbox" class="mr-4 rounded-2xl" checked={foldername()[1].synced} onclick={(e) => e.preventDefault()} />
            {foldername()[1].folder ? <Folder /> : <InsertDriveFile />} {foldername()[0]}
            <Loop style={{"visibility": foldername()[1].loading ? "visible" : "hidden"}} class="ml-2"/>
          </DivButton>
        }
      </Index>
    </div>
    <Portal>
      <div class="fixed right-0 bottom-0 m-4">
        <button onclick={props.back}>Back to tags</button>
      </div>
    </Portal>
  </>
}
