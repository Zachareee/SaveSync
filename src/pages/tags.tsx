import { emit, invoke, listen, osStringToString, unlisten } from "@/logic/backend"
import { useNavigate } from "@solidjs/router"
import { For } from "solid-js"
import toast from "solid-toast"
import isEqual from "lodash/isEqual"
import { Portal } from "solid-js/web"
import { reconcile } from "solid-js/store"
import { conflicting_listener } from "@/logic/conflicting_window"

import { folders, setFolders } from "@/App"
import { silenceMissingMappings } from "./settings"

export default function Folders() {
  const navigate = useNavigate()

  unlisten([
    listen("sync_result", ([tag, folder, bool]) => {
      setFolders(tag, osStringToString(folder), { loading: false, synced: bool })
    }),
    conflicting_listener()
  ])()

  invoke("filetree").then(payload => {
    invoke("get_watched_folders").then(watched => {
      setFolders(reconcile(Object.fromEntries(
        Object.entries(payload).map(
          ([k, v]) => [k, Object.fromEntries(v.map(([filename, isFolder]) =>
            [osStringToString(filename), {
              folder: isFolder,
              synced: watched.some(
                tagpath => isEqual(tagpath, [k, filename])
              ),
              loading: false
            }]
          ))]
        )
      )))
    })
  })

  invoke("get_mapping").then(({ mapping, required }) => {
    let current = Object.entries(mapping).map(([key]) => key)
    if (required.some(tag => !current.includes(tag)) && !silenceMissingMappings())
      toast.error(
        (t) =>
          <p onclick={() => {
            toast.dismiss(t.id)
            navigate("/mapping")
          }}>Some folders were not synced, please check Tag Mapping</p>
      )
  })

  return <div class="flex justify-center overflow-y-auto">
    <div class="text-center">
      <For each={Object.keys(folders).toSorted()}>
        {elem =>
          <div class="border-white border-3 rounded-full m-4 px-4 py-2 bg-indigo-600 cursor-pointer hover:bg-indigo-800" onclick={[navigate, `/tags/${elem}`]}>
            {elem}
          </div>
        }
      </For>
      <Portal>
        <div class="fixed right-0 bottom-0 m-4">
          <button onclick={() => { emit("unload"); navigate("/") }}>Back to plugin select</button>
        </div>
      </Portal>
    </div>
  </div>
}
